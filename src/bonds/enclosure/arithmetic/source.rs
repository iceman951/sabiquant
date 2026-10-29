//! Exact raw-source and affine staging in the producer's borrowed scalar Frame.
#![forbid(unsafe_code)]

use super::scalar::{E, Frame, R, Raw, Work};
use std::cmp::Ordering;

/// Small helper failures; all partially staged fields must be discarded on error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Error {
    Nonfinite,
    DenominatorZero,
    Capacity,
    WorkLimit,
}

type Result<T> = std::result::Result<T, Error>;

/// Exact staged value is signed Raw0.N/Raw0.D times 2^exponent.
/// Frame.E0 is also initialized to abs(exponent), ready for borrowed Normalize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) struct Seed {
    pub(in crate::bonds::enclosure) negative: bool,
    pub(in crate::bonds::enclosure) exponent: i32,
}

struct Parts {
    negative: bool,
    significand: u64,
    exponent: i32,
}

fn charge(work: &mut Work, count: u64) -> Result<()> {
    let next = work.used.checked_add(count).ok_or(Error::WorkLimit)?;
    if next > work.max {
        return Err(Error::WorkLimit);
    }
    work.used = next;
    Ok(())
}

fn parts(bits: u64, work: &mut Work) -> Result<Parts> {
    charge(work, 12)?;
    let encoded = (bits >> 52) & 2047;
    if encoded == 2047 {
        return Err(Error::Nonfinite);
    }
    let significand = (bits & ((1_u64 << 52) - 1)) | if encoded == 0 { 0 } else { 1_u64 << 52 };
    Ok(Parts {
        negative: bits >> 63 != 0 && significand != 0,
        significand,
        exponent: if significand == 0 {
            0
        } else if encoded == 0 {
            -1074
        } else {
            encoded as i32 - 1075
        },
    })
}

fn clear(raw: &mut Raw, work: &mut Work) -> Result<()> {
    charge(work, (2 * R + 2) as u64)?;
    raw.n.fill(0);
    raw.d.fill(0);
    raw.controls.fill(0);
    Ok(())
}

fn shifted(out: &mut [u64; R], value: u128, shift: u32, work: &mut Work) -> Result<()> {
    charge(work, (R + 24) as u64)?;
    out.fill(0);
    if value == 0 {
        return Ok(());
    }
    if shift >= (64 * R) as u32 {
        return Err(Error::Capacity);
    }
    let offset = shift as usize / 64;
    let bits = shift % 64;
    for lane in 0..2 {
        let word = (value >> (64 * lane)) as u64;
        if word == 0 {
            continue;
        }
        let index = offset + lane;
        let target = out.get_mut(index).ok_or(Error::Capacity)?;
        *target |= word << bits;
        if bits != 0 {
            let carry = word >> (64 - bits);
            if carry != 0 {
                let next = out.get_mut(index + 1).ok_or(Error::Capacity)?;
                *next |= carry;
            }
        }
    }
    Ok(())
}

fn compare(a: &[u64; R], b: &[u64; R], work: &mut Work) -> Result<Ordering> {
    charge(work, R as u64)?;
    for (x, y) in a.iter().zip(b).rev() {
        if x != y {
            return Ok(x.cmp(y));
        }
    }
    Ok(Ordering::Equal)
}

fn add(destination: &mut [u64; R], other: &[u64; R], work: &mut Work) -> Result<()> {
    charge(work, (4 * R + 2) as u64)?;
    let mut carry = 0_u128;
    for (a, b) in destination.iter_mut().zip(other) {
        let sum = u128::from(*a) + u128::from(*b) + carry;
        *a = sum as u64;
        carry = sum >> 64;
    }
    if carry != 0 {
        return Err(Error::Capacity);
    }
    Ok(())
}

/// Subtract in either direction without an owned working array or large swap.
fn difference(
    destination: &mut [u64; R],
    other: &[u64; R],
    reverse: bool,
    work: &mut Work,
) -> Result<()> {
    charge(work, (5 * R + 2) as u64)?;
    let mut borrow = false;
    for (a, b) in destination.iter_mut().zip(other) {
        let (left, right) = if reverse { (*b, *a) } else { (*a, *b) };
        let (first, b1) = left.overflowing_sub(right);
        let (second, b2) = first.overflowing_sub(u64::from(borrow));
        *a = second;
        borrow = b1 || b2;
    }
    if borrow {
        return Err(Error::Capacity);
    }
    Ok(())
}

fn finish(frame: &mut Frame, negative: bool, exponent: i32, work: &mut Work) -> Result<Seed> {
    charge(work, (R + E + 4) as u64)?;
    let zero = frame.raw0.n.iter().all(|&x| x == 0);
    let exponent = if zero { 0 } else { exponent };
    frame.e0.fill(0);
    frame.e0[0] = u64::from(exponent.unsigned_abs());
    Ok(Seed {
        negative: negative && !zero,
        exponent,
    })
}

fn stage_affine(
    magnitude: u128,
    negative: bool,
    exponent: i32,
    denominator: u32,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    charge(work, 8)?;
    if denominator == 0 {
        return Err(Error::DenominatorZero);
    }
    clear(&mut frame.raw0, work)?;
    clear(&mut frame.raw1, work)?;
    let left_shift = exponent.min(0).unsigned_abs();
    shifted(&mut frame.raw0.n, u128::from(denominator), left_shift, work)?;
    shifted(&mut frame.raw0.d, u128::from(denominator), left_shift, work)?;
    shifted(&mut frame.raw1.n, magnitude, exponent.max(0) as u32, work)?;
    let output_negative;
    if negative {
        let order = compare(&frame.raw0.n, &frame.raw1.n, work)?;
        output_negative = order == Ordering::Less;
        difference(&mut frame.raw0.n, &frame.raw1.n, output_negative, work)?;
    } else {
        output_negative = false;
        add(&mut frame.raw0.n, &frame.raw1.n, work)?;
    }
    finish(frame, output_negative, 0, work)
}

/// Decode the exact represented finite value; signed zero is canonicalized.
pub(in crate::bonds::enclosure) fn binary(
    bits: u64,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    let p = parts(bits, work)?;
    clear(&mut frame.raw0, work)?;
    shifted(&mut frame.raw0.n, u128::from(p.significand), 0, work)?;
    charge(work, 1)?;
    frame.raw0.d[0] = 1;
    finish(frame, p.negative, p.exponent, work)
}

/// Stage the actual Generic unit ratio100/face as one exact source operand.
/// A reciprocal interval followed by multiplication would unnecessarily widen
/// even100/100 and hide an otherwise complete final singleton.
pub(in crate::bonds::enclosure) fn per_100_face(
    bits: u64,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    let face = parts(bits, work)?;
    charge(work, 4)?;
    if face.significand == 0 {
        return Err(Error::DenominatorZero);
    }
    if face.negative {
        return Err(Error::Capacity);
    }
    source(&[100], &[face.significand], false, frame, work)?;
    finish(frame, false, -face.exponent, work)
}

/// Stage exact represented a*b without native multiplication of floating values.
pub(in crate::bonds::enclosure) fn product(
    a: u64,
    b: u64,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    let a = parts(a, work)?;
    let b = parts(b, work)?;
    charge(work, 4)?;
    let magnitude = u128::from(a.significand) * u128::from(b.significand);
    clear(&mut frame.raw0, work)?;
    shifted(&mut frame.raw0.n, magnitude, 0, work)?;
    charge(work, 1)?;
    frame.raw0.d[0] = 1;
    finish(
        frame,
        a.negative != b.negative,
        a.exponent + b.exponent,
        work,
    )
}

/// Exact Simple affine 1+y*t, including both negative and zero numerators.
pub(in crate::bonds::enclosure) fn simple(
    y: u64,
    t: u64,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    let y = parts(y, work)?;
    let t = parts(t, work)?;
    charge(work, 4)?;
    let magnitude = u128::from(y.significand) * u128::from(t.significand);
    stage_affine(
        magnitude,
        y.negative != t.negative,
        y.exponent + t.exponent,
        1,
        frame,
        work,
    )
}

/// Exact 1+y*n/d for explicit u32 source counts, d>0. Denominator validation first.
pub(in crate::bonds::enclosure) fn ratio_affine(
    y: u64,
    numerator: u32,
    denominator: u32,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    charge(work, 2)?;
    if denominator == 0 {
        return Err(Error::DenominatorZero);
    }
    let y = parts(y, work)?;
    charge(work, 2)?;
    let magnitude = u128::from(y.significand) * u128::from(numerator);
    stage_affine(magnitude, y.negative, y.exponent, denominator, frame, work)
}

/// Copy exact bounded source words; slices are borrowed and at most Raw34 each.
pub(in crate::bonds::enclosure) fn source(
    numerator: &[u64],
    denominator: &[u64],
    negative: bool,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<Seed> {
    charge(work, 2)?;
    if numerator.len() > R || denominator.len() > R {
        return Err(Error::Capacity);
    }
    charge(work, denominator.len() as u64)?;
    if denominator.iter().all(|&x| x == 0) {
        return Err(Error::DenominatorZero);
    }
    clear(&mut frame.raw0, work)?;
    charge(work, (numerator.len() + denominator.len()) as u64)?;
    for (a, b) in frame.raw0.n.iter_mut().zip(numerator) {
        *a = *b;
    }
    for (a, b) in frame.raw0.d.iter_mut().zip(denominator) {
        *a = *b;
    }
    finish(frame, negative, 0, work)
}

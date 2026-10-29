//! Bounded directed log/exp operations over the shared producer arena.
#![forbid(unsafe_code)]

use super::scalar::{Frame, Work};
use super::{Bound, EndpointSign, ExponentView, SlotArena, UNIT, copy_point, point_cmp};
use std::cmp::Ordering;

#[derive(Debug)]
pub(in crate::bonds::enclosure) enum KernelError {
    Arena(super::Error),
    LogDomain,
    InputCapacity,
    ExpRange,
    PointRequired,
    SlotProtocol,
    Invariant,
}
impl From<super::Error> for KernelError {
    fn from(value: super::Error) -> Self {
        Self::Arena(value)
    }
}
impl From<super::scalar::Error> for KernelError {
    fn from(value: super::scalar::Error) -> Self {
        Self::Arena(super::Error::Scalar(value))
    }
}
type Result<T> = std::result::Result<T, KernelError>;

const Z: usize = 8;
const Z2: usize = 9;
const TERM: usize = 10;
const SUM: usize = 11;
const COEFFICIENT: usize = 12;
const PRODUCT: usize = 13;
const SUM_NEXT: usize = 14;
const TERM_NEXT: usize = 15;
const HELD: usize = 16;

fn tick(work: &mut Work, count: usize) -> Result<()> {
    super::charge(
        work,
        u64::try_from(count).map_err(|_| KernelError::InputCapacity)?,
    )?;
    Ok(())
}
fn bits(words: &[u64], work: &mut Work) -> Result<usize> {
    tick(work, words.len())?;
    Ok(words
        .iter()
        .rposition(|&word| word != 0)
        .map_or(0, |index| {
            index * 64 + 64 - words[index].leading_zeros() as usize
        }))
}
fn compare(a: &[u64], b: &[u64], work: &mut Work) -> Result<Ordering> {
    tick(work, a.len().max(b.len()))?;
    for index in (0..a.len().max(b.len())).rev() {
        let order = a
            .get(index)
            .copied()
            .unwrap_or(0)
            .cmp(&b.get(index).copied().unwrap_or(0));
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(Ordering::Equal)
}
fn relative_left(
    destination: &mut [u64],
    source: &[u64],
    shift: usize,
    work: &mut Work,
) -> Result<()> {
    let significant = bits(source, work)?;
    if significant != 0
        && significant
            .checked_add(shift)
            .ok_or(KernelError::InputCapacity)?
            > destination.len() * 64
    {
        return Err(KernelError::InputCapacity);
    }
    tick(work, destination.len() + source.len() + 4)?;
    destination.fill(0);
    let words = shift / 64;
    let remainder = shift % 64;
    for (index, &word) in source.iter().enumerate() {
        if word == 0 {
            continue;
        }
        let target = index.checked_add(words).ok_or(KernelError::InputCapacity)?;
        *destination
            .get_mut(target)
            .ok_or(KernelError::InputCapacity)? |= word << remainder;
        if remainder != 0 {
            let high = word >> (64 - remainder);
            if let Some(next) = destination.get_mut(target + 1) {
                *next |= high;
            } else if high != 0 {
                return Err(KernelError::InputCapacity);
            }
        }
    }
    Ok(())
}
fn add_magnitudes(destination: &mut [u64], a: &[u64], b: &[u64], work: &mut Work) -> Result<()> {
    tick(work, destination.len())?;
    let mut carry = 0_u128;
    for (index, word) in destination.iter_mut().enumerate() {
        let sum = u128::from(a.get(index).copied().unwrap_or(0))
            + u128::from(b.get(index).copied().unwrap_or(0))
            + carry;
        *word = sum as u64;
        carry = sum >> 64;
    }
    if carry != 0 {
        Err(KernelError::InputCapacity)
    } else {
        Ok(())
    }
}
fn subtract_magnitudes(
    destination: &mut [u64],
    a: &[u64],
    b: &[u64],
    work: &mut Work,
) -> Result<()> {
    tick(work, destination.len())?;
    let mut borrow = false;
    for (index, word) in destination.iter_mut().enumerate() {
        let (value, b1) = a
            .get(index)
            .copied()
            .unwrap_or(0)
            .overflowing_sub(b.get(index).copied().unwrap_or(0));
        let (value, b2) = value.overflowing_sub(u64::from(borrow));
        *word = value;
        borrow = b1 || b2;
    }
    if borrow {
        Err(KernelError::Invariant)
    } else {
        Ok(())
    }
}

/// Adjust full E33 signed magnitude by a bounded small offset, in place.
fn adjust_small(words: &mut [u64], negative: bool, delta: i32, work: &mut Work) -> Result<bool> {
    tick(work, 4 * words.len() + 16)?;
    let magnitude = u64::from(delta.unsigned_abs());
    let delta_negative = delta < 0;
    if negative == delta_negative {
        let mut carry = magnitude;
        for word in words.iter_mut() {
            let (sum, overflow) = word.overflowing_add(carry);
            *word = sum;
            carry = u64::from(overflow);
        }
        if carry != 0 {
            return Err(KernelError::InputCapacity);
        }
        return Ok(negative && words.iter().any(|&word| word != 0));
    }
    let bigger = words.iter().skip(1).any(|&word| word != 0)
        || words.first().copied().unwrap_or(0) > magnitude;
    if bigger {
        let mut borrow = magnitude;
        for word in words.iter_mut() {
            let (value, overflow) = word.overflowing_sub(borrow);
            *word = value;
            borrow = u64::from(overflow);
        }
        if borrow != 0 {
            return Err(KernelError::Invariant);
        }
        Ok(negative)
    } else {
        let old = words.first().copied().unwrap_or(0);
        words.fill(0);
        *words.first_mut().ok_or(KernelError::InputCapacity)? = magnitude - old;
        Ok(delta_negative && magnitude != old)
    }
}

fn setup(p: usize, slots: &SlotArena, output: usize, work: &mut Work) -> Result<()> {
    tick(work, 8)?;
    if ![64, 128, 256, 512, 1024].contains(&p) {
        return Err(super::scalar::Error::Precision.into());
    }
    if output >= 8 || slots.unit_precision != Some(p) {
        return Err(KernelError::SlotProtocol);
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "one borrowed Frame and explicit bounded exact seed"
)]
fn exact_small(
    slots: &mut SlotArena,
    frame: &mut Frame,
    work: &mut Work,
    p: usize,
    output: usize,
    n: u64,
    d: u64,
    negative: bool,
    exponent: i32,
) -> Result<()> {
    tick(work, 2 * super::scalar::R + super::scalar::E + 16)?;
    frame.raw0.n.fill(0);
    frame.raw0.d.fill(0);
    frame.e0.fill(0);
    frame.raw0.n[0] = n;
    frame.raw0.d[0] = d;
    frame.e0[0] = u64::from(exponent.unsigned_abs());
    slots.normalize_staged(output, p, negative, exponent < 0, frame, work)?;
    Ok(())
}

/// Input Raw0 is released after exact b=2^k*m, z=(m-1)/(m+1).
fn prepare_log_z(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<i32> {
    let nb = bits(&frame.raw0.n, work)?;
    let db = bits(&frame.raw0.d, work)?;
    if nb == 0 || db == 0 {
        return Err(KernelError::LogDomain);
    }
    if nb > 1084 || db > 1084 {
        return Err(KernelError::InputCapacity);
    }
    let mut k = nb as i32 - db as i32;
    for attempt in 0..2 {
        tick(work, 1)?;
        relative_left(
            &mut frame.raw1.n,
            &frame.raw0.n,
            if k < 0 { k.unsigned_abs() as usize } else { 0 },
            work,
        )?;
        relative_left(
            &mut frame.raw1.d,
            &frame.raw0.d,
            if k > 0 { k as usize } else { 0 },
            work,
        )?;
        if compare(&frame.raw1.n, &frame.raw1.d, work)? != Ordering::Less {
            break;
        }
        if attempt != 0 {
            return Err(KernelError::Invariant);
        }
        k -= 1;
    }
    subtract_magnitudes(&mut frame.raw0.n, &frame.raw1.n, &frame.raw1.d, work)?;
    add_magnitudes(&mut frame.raw0.d, &frame.raw1.n, &frame.raw1.d, work)?;
    tick(work, super::scalar::E)?;
    frame.e0.fill(0);
    slots.normalize_staged(Z, p, false, false, frame, work)?;
    Ok(k)
}

/// Positive finite atanh series for the exact source z in[0,1/3], in L0..L7.
/// Output L0 contains2sum plus[0,2^(1-2p)]; exact z0 stays exact0.
fn atanh_z(p: usize, slots: &mut SlotArena, frame: &mut Frame, work: &mut Work) -> Result<usize> {
    let lower = slots.inspect_endpoint(p, Z, Bound::Lower, work)?;
    let upper = slots.inspect_endpoint(p, Z, Bound::Upper, work)?;
    if lower.sign == EndpointSign::Zero && upper.sign == EndpointSign::Zero {
        return Ok(Z);
    }
    if lower.sign != EndpointSign::Positive || upper.sign != EndpointSign::Positive {
        return Err(KernelError::Invariant);
    }
    slots.multiply(p, Z, Z, Z2, frame, work)?;
    slots.copy_interval(p, Z, TERM, work)?;
    exact_small(slots, frame, work, p, SUM, 0, 1, false, 0)?;
    let (mut term, mut term_next) = (TERM, TERM_NEXT);
    let (mut sum, mut sum_next) = (SUM, SUM_NEXT);
    for j in 0..p {
        tick(work, 1)?;
        exact_small(
            slots,
            frame,
            work,
            p,
            COEFFICIENT,
            1,
            (2 * j + 1) as u64,
            false,
            0,
        )?;
        slots.multiply(p, term, COEFFICIENT, PRODUCT, frame, work)?;
        slots.add(p, sum, PRODUCT, sum_next, frame, work)?;
        std::mem::swap(&mut sum, &mut sum_next);
        if j + 1 < p {
            slots.multiply(p, term, Z2, term_next, frame, work)?;
            std::mem::swap(&mut term, &mut term_next);
        }
    }
    exact_small(slots, frame, work, p, COEFFICIENT, 2, 1, false, 0)?;
    slots.multiply(p, sum, COEFFICIENT, PRODUCT, frame, work)?;
    exact_small(
        slots,
        frame,
        work,
        p,
        COEFFICIENT,
        1,
        1,
        false,
        1 - 2 * p as i32,
    )?;
    slots.add(p, PRODUCT, COEFFICIENT, sum_next, frame, work)?;
    slots.copy_endpoint(p, PRODUCT, Bound::Lower, Z, Bound::Lower, work)?;
    slots.copy_endpoint(p, sum_next, Bound::Upper, Z, Bound::Upper, work)?;
    Ok(Z)
}

/// Enclose ln(N/D) from positive exact staged Raw0 inputs of<=1084bits each.
pub(in crate::bonds::enclosure) fn logarithm(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    output: usize,
    work: &mut Work,
) -> Result<()> {
    setup(p, slots, output, work)?;
    let k = prepare_log_z(p, slots, frame, work)?;
    if k == 0 && slots.inspect_endpoint(p, Z, Bound::Upper, work)?.sign == EndpointSign::Zero {
        exact_small(slots, frame, work, p, output, 0, 1, false, 0)?;
        return Ok(());
    }
    let log_m = atanh_z(p, slots, frame, work)?;
    slots.copy_interval(p, log_m, HELD, work)?;
    exact_small(slots, frame, work, p, Z, 1, 3, false, 0)?;
    let ln2 = atanh_z(p, slots, frame, work)?;
    exact_small(
        slots,
        frame,
        work,
        p,
        17,
        u64::from(k.unsigned_abs()),
        1,
        k < 0,
        0,
    )?;
    slots.multiply(p, ln2, 17, 18, frame, work)?;
    slots.add(p, 18, HELD, 19, frame, work)?;
    slots.copy_interval(p, 19, output, work)?;
    Ok(())
}

fn validate_exp_endpoint(
    p: usize,
    slots: &SlotArena,
    input: usize,
    bound: Bound,
    work: &mut Work,
) -> Result<()> {
    let facts = slots.inspect_endpoint(p, input, bound, work)?;
    match facts.exponent {
        ExponentView::BeyondI32 { negative: false } => Err(KernelError::ExpRange),
        ExponentView::Small(k) if k > 2048 || (k == 2048 && !facts.power_of_two) => {
            Err(KernelError::ExpRange)
        }
        _ => Ok(()),
    }
}

fn prepare_exp_absolute(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    input: usize,
    bound: Bound,
    work: &mut Work,
) -> Result<(usize, bool)> {
    let facts = slots.inspect_endpoint(p, input, bound, work)?;
    let s = match facts.exponent {
        ExponentView::Small(k) if k >= -1 => {
            usize::try_from(k + 1 + i32::from(!facts.power_of_two))
                .map_err(|_| KernelError::Invariant)?
        }
        _ => 0,
    };
    if s > 2049 {
        return Err(KernelError::ExpRange);
    }
    tick(
        work,
        super::scalar::R * 2 + super::scalar::M + super::scalar::E + 16,
    )?;
    let point = super::endpoint(
        slots.slots.get(input).ok_or(KernelError::SlotProtocol)?,
        bound,
    );
    frame.raw0.n.fill(0);
    frame.raw0.d.fill(0);
    frame.raw0.n[..super::scalar::M].copy_from_slice(&point.m);
    frame.raw0.d[0] = 1;
    frame.e0.copy_from_slice(&point.k);
    let negative = adjust_small(
        &mut frame.e0,
        point.flags & 2 != 0,
        -(p as i32 - 1) - s as i32,
        work,
    )?;
    slots.normalize_staged(8, p, false, negative, frame, work)?;
    Ok((s, facts.sign == EndpointSign::Negative))
}

fn intersect_unit_two(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    output: usize,
    work: &mut Work,
) -> Result<()> {
    exact_small(slots, frame, work, p, 19, 2, 1, false, 0)?;
    let [answer, unit, two] = slots
        .slots
        .get_disjoint_mut([output, UNIT, 19])
        .map_err(|_| KernelError::SlotProtocol)?;
    if point_cmp(&answer.lower, &unit.lower, work)? == Ordering::Less {
        copy_point(&mut answer.lower, &unit.lower, work)?;
    }
    if point_cmp(&answer.upper, &two.upper, work)? == Ordering::Greater {
        copy_point(&mut answer.upper, &two.upper, work)?;
    }
    if point_cmp(&answer.lower, &answer.upper, work)? == Ordering::Greater {
        return Err(KernelError::Invariant);
    }
    Ok(())
}

/// One endpoint's whole rigorous exp interval in L0..L12; original input global
/// remains borrowed. No working Point/Frame is returned by value.
fn exp_endpoint(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    input: usize,
    bound: Bound,
    work: &mut Work,
) -> Result<usize> {
    if slots.inspect_endpoint(p, input, bound, work)?.sign == EndpointSign::Zero {
        slots.copy_interval(p, UNIT, HELD, work)?;
        return Ok(HELD);
    }
    let (s, negative) = prepare_exp_absolute(p, slots, frame, input, bound, work)?;
    let (mut term, mut term_next) = (9, 13);
    let (mut sum, mut sum_next) = (10, 14);
    slots.copy_interval(p, UNIT, term, work)?;
    slots.copy_interval(p, UNIT, sum, work)?;
    for j in 1..=2 * p {
        tick(work, 1)?;
        exact_small(slots, frame, work, p, 11, 1, j as u64, false, 0)?;
        slots.multiply(p, term, 8, 12, frame, work)?;
        slots.multiply(p, 12, 11, term_next, frame, work)?;
        slots.add(p, sum, term_next, sum_next, frame, work)?;
        std::mem::swap(&mut term, &mut term_next);
        std::mem::swap(&mut sum, &mut sum_next);
    }
    exact_small(slots, frame, work, p, 15, 1, 1, false, -2 * p as i32)?;
    slots.add(p, sum, 15, HELD, frame, work)?;
    slots.copy_endpoint(p, sum, Bound::Lower, HELD, Bound::Lower, work)?;
    intersect_unit_two(p, slots, frame, HELD, work)?;
    let (mut answer, mut next) = (HELD, 17);
    for _ in 0..s {
        tick(work, 1)?;
        slots.multiply(p, answer, answer, next, frame, work)?;
        std::mem::swap(&mut answer, &mut next);
    }
    if negative {
        slots.reciprocal(p, answer, 18, frame, work)?;
        answer = 18;
    }
    Ok(answer)
}

fn validate_exp_input(
    p: usize,
    slots: &SlotArena,
    input: usize,
    output: usize,
    work: &mut Work,
) -> Result<()> {
    setup(p, slots, output, work)?;
    if input >= 8 || input == output {
        return Err(KernelError::SlotProtocol);
    }
    validate_exp_endpoint(p, slots, input, Bound::Lower, work)?;
    validate_exp_endpoint(p, slots, input, Bound::Upper, work)?;
    let source = slots.slots.get(input).ok_or(KernelError::SlotProtocol)?;
    if point_cmp(&source.lower, &source.upper, work)? == Ordering::Greater {
        return Err(KernelError::Invariant);
    }
    Ok(())
}

/// Exact-point input: equal validated input endpoints, one Taylor/squaring route.
pub(in crate::bonds::enclosure) fn exponential_point(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    input: usize,
    output: usize,
    work: &mut Work,
) -> Result<()> {
    validate_exp_input(p, slots, input, output, work)?;
    let source = slots.slots.get(input).ok_or(KernelError::SlotProtocol)?;
    if point_cmp(&source.lower, &source.upper, work)? != Ordering::Equal {
        return Err(KernelError::PointRequired);
    }
    let result = exp_endpoint(p, slots, frame, input, Bound::Lower, work)?;
    slots.copy_interval(p, result, output, work)?;
    Ok(())
}

/// Interval input: both endpoint runs are charged/executed. The first completed
/// lower result stays in the distinct global output during the second run.
pub(in crate::bonds::enclosure) fn exponential_interval(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    input: usize,
    output: usize,
    work: &mut Work,
) -> Result<()> {
    validate_exp_input(p, slots, input, output, work)?;
    let lower = exp_endpoint(p, slots, frame, input, Bound::Lower, work)?;
    slots.copy_endpoint(p, lower, Bound::Lower, output, Bound::Lower, work)?;
    let upper = exp_endpoint(p, slots, frame, input, Bound::Upper, work)?;
    slots.copy_endpoint(p, upper, Bound::Upper, output, Bound::Upper, work)?;
    Ok(())
}

/// Input transport has staged p-bit m in Raw0.N, D1 and original K in E0.
/// Translate the exact normalized Point unit via full-width metadata, then call
/// actual Normalize; this is seed projection, not an exponential calculation.
pub(in crate::bonds::enclosure) fn normalize_exp_seed(
    p: usize,
    slots: &mut SlotArena,
    frame: &mut Frame,
    output: usize,
    value_negative: bool,
    k_negative: bool,
    work: &mut Work,
) -> Result<()> {
    setup(p, slots, output, work)?;
    let nbits = bits(&frame.raw0.n, work)?;
    if nbits != 0 && nbits != p {
        return Err(KernelError::InputCapacity);
    }
    let negative = adjust_small(&mut frame.e0, k_negative, -(p as i32 - 1), work)?;
    slots.normalize_staged(output, p, value_negative, negative, frame, work)?;
    Ok(())
}

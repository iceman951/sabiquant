//! Bounded limb arithmetic shared by exact folds and directed price intervals.
#![forbid(unsafe_code)]

use std::cmp::Ordering;

pub(in crate::bonds::enclosure) const M: usize = 16;
pub(in crate::bonds::enclosure) const E: usize = 33;
pub(in crate::bonds::enclosure) const R: usize = 34;
pub(in crate::bonds::enclosure) const W: usize = 52;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Error {
    Precision,
    DenominatorZero,
    Capacity,
    Malformed,
    WorkLimit,
}
type Result<T> = std::result::Result<T, Error>;

/// Abstract prepaid limb/scalar actions, not CPU cycles or final producer work.
pub(in crate::bonds::enclosure) struct Work {
    pub(in crate::bonds::enclosure) used: u64,
    pub(in crate::bonds::enclosure) max: u64,
}
impl Work {
    fn charge(&mut self, n: usize) -> Result<()> {
        let n = u64::try_from(n).map_err(|_| Error::Capacity)?;
        let next = self.used.checked_add(n).ok_or(Error::WorkLimit)?;
        if next > self.max {
            return Err(Error::WorkLimit);
        }
        self.used = next;
        Ok(())
    }
}

/// flags bit0=value negative; bit1=K negative. Zero is all-zero.
/// Intentionally neither Copy nor Clone; all numerical results use borrowed writes.
pub(in crate::bonds::enclosure) struct Point {
    pub(in crate::bonds::enclosure) m: [u64; M],
    pub(in crate::bonds::enclosure) k: [u64; E],
    pub(in crate::bonds::enclosure) flags: u64,
}
pub(in crate::bonds::enclosure) const ZERO_POINT: Point = Point {
    m: [0; M],
    k: [0; E],
    flags: 0,
};

pub(in crate::bonds::enclosure) struct Raw {
    pub(in crate::bonds::enclosure) n: [u64; R],
    pub(in crate::bonds::enclosure) d: [u64; R],
    pub(in crate::bonds::enclosure) controls: [u64; 2],
}
const ZERO_RAW: Raw = Raw {
    n: [0; R],
    d: [0; R],
    controls: [0; 2],
};

/// One explicit 531-word scalar frame. No helper constructs/returns another frame.
pub(in crate::bonds::enclosure) struct Frame {
    pub(in crate::bonds::enclosure) raw0: Raw,
    pub(in crate::bonds::enclosure) raw1: Raw,
    pub(in crate::bonds::enclosure) w0: [u64; W],
    pub(in crate::bonds::enclosure) w1: [u64; W],
    pub(in crate::bonds::enclosure) w2: [u64; W],
    pub(in crate::bonds::enclosure) w3: [u64; W],
    pub(in crate::bonds::enclosure) q: [u64; W],
    pub(in crate::bonds::enclosure) e0: [u64; E],
    pub(in crate::bonds::enclosure) e1: [u64; E],
    pub(in crate::bonds::enclosure) e2: [u64; E],
    pub(in crate::bonds::enclosure) controls: [u64; 32],
}
pub(in crate::bonds::enclosure) const ZERO_FRAME: Frame = Frame {
    raw0: ZERO_RAW,
    raw1: ZERO_RAW,
    w0: [0; W],
    w1: [0; W],
    w2: [0; W],
    w3: [0; W],
    q: [0; W],
    e0: [0; E],
    e1: [0; E],
    e2: [0; E],
    controls: [0; 32],
};

fn precision(p: usize) -> Result<()> {
    if [64, 128, 256, 512, 1024].contains(&p) {
        Ok(())
    } else {
        Err(Error::Precision)
    }
}
fn clear(a: &mut [u64], work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    a.fill(0);
    Ok(())
}
fn bits(a: &[u64], work: &mut Work) -> Result<usize> {
    work.charge(a.len())?;
    for i in (0..a.len()).rev() {
        if a[i] != 0 {
            return Ok(i * 64 + (64 - a[i].leading_zeros() as usize));
        }
    }
    Ok(0)
}
fn cmp(a: &[u64], b: &[u64], work: &mut Work) -> Result<Ordering> {
    work.charge(a.len().max(b.len()))?;
    for i in (0..a.len().max(b.len())).rev() {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return Ok(x.cmp(&y));
        }
    }
    Ok(Ordering::Equal)
}
fn copy(dst: &mut [u64], src: &[u64], work: &mut Work) -> Result<()> {
    if bits(src, work)? > dst.len() * 64 {
        return Err(Error::Capacity);
    }
    clear(dst, work)?;
    let n = dst.len().min(src.len());
    work.charge(n)?;
    dst[..n].copy_from_slice(&src[..n]);
    Ok(())
}
fn left(dst: &mut [u64], src: &[u64], n: usize, work: &mut Work) -> Result<()> {
    let b = bits(src, work)?;
    if b != 0 && b.checked_add(n).ok_or(Error::Capacity)? > dst.len() * 64 {
        return Err(Error::Capacity);
    }
    clear(dst, work)?;
    work.charge(src.len())?;
    if b == 0 {
        return Ok(());
    }
    let words = n / 64;
    let rem = n % 64;
    for (i, &v) in src.iter().enumerate() {
        if v == 0 {
            continue;
        }
        let j = i + words;
        dst[j] |= v << rem;
        if rem != 0 && j + 1 < dst.len() {
            dst[j + 1] |= v >> (64 - rem);
        }
    }
    Ok(())
}
fn right(a: &mut [u64], n: usize, work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    let words = n / 64;
    let rem = n % 64;
    for i in 0..a.len() {
        a[i] = if i + words < a.len() {
            let lo = a[i + words] >> rem;
            let hi = if rem != 0 && i + words + 1 < a.len() {
                a[i + words + 1] << (64 - rem)
            } else {
                0
            };
            lo | hi
        } else {
            0
        };
    }
    Ok(())
}
fn sticky(a: &[u64], n: usize, work: &mut Work) -> Result<bool> {
    work.charge(a.len())?;
    for (i, &v) in a.iter().enumerate() {
        let start = i * 64;
        if n >= start + 64 && v != 0 {
            return Ok(true);
        }
        if n > start && n < start + 64 && v & ((1_u64 << (n - start)) - 1) != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}
fn increment(a: &mut [u64], work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    for x in a {
        let (v, overflow) = x.overflowing_add(1);
        *x = v;
        if !overflow {
            return Ok(());
        }
    }
    Err(Error::Capacity)
}
fn sub_in_place(a: &mut [u64], b: &[u64], work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    let mut borrow = false;
    for (i, x) in a.iter_mut().enumerate() {
        let (v, b1) = x.overflowing_sub(b.get(i).copied().unwrap_or(0));
        let (v, b2) = v.overflowing_sub(u64::from(borrow));
        *x = v;
        borrow = b1 || b2;
    }
    if borrow { Err(Error::Capacity) } else { Ok(()) }
}
fn add_mag(dst: &mut [u64], a: &[u64], b: &[u64], work: &mut Work) -> Result<()> {
    work.charge(dst.len())?;
    let mut carry = 0_u128;
    for (i, x) in dst.iter_mut().enumerate() {
        let value = u128::from(a.get(i).copied().unwrap_or(0))
            + u128::from(b.get(i).copied().unwrap_or(0))
            + carry;
        *x = value as u64;
        carry = value >> 64;
    }
    if carry != 0 {
        Err(Error::Capacity)
    } else {
        Ok(())
    }
}
fn sub_mag(dst: &mut [u64], a: &[u64], b: &[u64], work: &mut Work) -> Result<()> {
    copy(dst, a, work)?;
    sub_in_place(dst, b, work)
}
fn signed_add(
    dst: &mut [u64],
    a: &[u64],
    an: bool,
    b: &[u64],
    bn: bool,
    work: &mut Work,
) -> Result<bool> {
    if an == bn {
        add_mag(dst, a, b, work)?;
        Ok(an && bits(dst, work)? != 0)
    } else {
        match cmp(a, b, work)? {
            Ordering::Less => {
                sub_mag(dst, b, a, work)?;
                Ok(bn)
            }
            Ordering::Equal => {
                clear(dst, work)?;
                Ok(false)
            }
            Ordering::Greater => {
                sub_mag(dst, a, b, work)?;
                Ok(an)
            }
        }
    }
}
fn cmp_small(a: &[u64], n: u64, work: &mut Work) -> Result<Ordering> {
    work.charge(a.len())?;
    if a.iter().skip(1).any(|&x| x != 0) {
        return Ok(Ordering::Greater);
    }
    Ok(a.first().copied().unwrap_or(0).cmp(&n))
}
fn add_small(a: &mut [u64], n: u64, work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    let mut carry = n;
    for x in a {
        let (v, overflow) = x.overflowing_add(carry);
        *x = v;
        carry = u64::from(overflow);
        if carry == 0 {
            return Ok(());
        }
    }
    Err(Error::Capacity)
}
fn sub_small(a: &mut [u64], n: u64, work: &mut Work) -> Result<()> {
    work.charge(a.len())?;
    let mut borrow = n;
    for x in a {
        let (v, overflow) = x.overflowing_sub(borrow);
        *x = v;
        borrow = u64::from(overflow);
        if borrow == 0 {
            return Ok(());
        }
    }
    Err(Error::Capacity)
}
fn adjust(a: &mut [u64], negative: bool, n: i32, work: &mut Work) -> Result<bool> {
    let nn = n < 0;
    let mag = u64::from(n.unsigned_abs());
    if nn == negative {
        add_small(a, mag, work)?;
        return Ok(negative && bits(a, work)? != 0);
    }
    match cmp_small(a, mag, work)? {
        Ordering::Less => {
            let old = a.first().copied().unwrap_or(0);
            clear(a, work)?;
            a[0] = mag - old;
            Ok(nn)
        }
        Ordering::Equal => {
            clear(a, work)?;
            Ok(false)
        }
        Ordering::Greater => {
            sub_small(a, mag, work)?;
            Ok(negative)
        }
    }
}
fn cmp_signed(a: &[u64], an: bool, b: &[u64], bn: bool, work: &mut Work) -> Result<Ordering> {
    work.charge(1)?;
    if an != bn {
        return Ok(if an {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let order = cmp(a, b, work)?;
    Ok(if an { order.reverse() } else { order })
}
fn zero_point(out: &mut Point, work: &mut Work) -> Result<()> {
    clear(&mut out.m, work)?;
    clear(&mut out.k, work)?;
    work.charge(1)?;
    out.flags = 0;
    Ok(())
}
fn valid_point(a: &Point, p: usize, work: &mut Work) -> Result<()> {
    work.charge(1)?;
    let n = bits(&a.m, work)?;
    if a.flags & !3 != 0 || (n != 0 && n != p) {
        return Err(Error::Malformed);
    }
    let kbits = bits(&a.k, work)?;
    if (n == 0 && (kbits != 0 || a.flags != 0)) || (kbits == 0 && a.flags & 2 != 0) {
        return Err(Error::Malformed);
    }
    Ok(())
}
fn emit(
    out: &mut Point,
    q: &[u64],
    k: &[u64],
    kn: bool,
    negative: bool,
    work: &mut Work,
) -> Result<()> {
    copy(&mut out.m, q, work)?;
    copy(&mut out.k, k, work)?;
    work.charge(1)?;
    out.flags = u64::from(negative) | (u64::from(kn) << 1);
    Ok(())
}

/// Divides aligned exact nonnegative raw0; e0 is immutable huge unit metadata.
/// mask 1 writes lower, mask 2 upper. Both directed mantissas share quotient/remainder.
#[allow(
    clippy::too_many_arguments,
    reason = "separate borrowed outputs and scratch prevent large aggregate copies"
)]
fn normalize_mask(
    p: usize,
    negative: bool,
    base_negative: bool,
    mask: u8,
    lo: &mut Point,
    hi: &mut Point,
    f: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    precision(p)?;
    let nb = bits(&f.raw0.n, work)?;
    let db = bits(&f.raw0.d, work)?;
    if db == 0 {
        return Err(Error::DenominatorZero);
    }
    if nb == 0 {
        if mask & 1 != 0 {
            zero_point(lo, work)?;
        }
        if mask & 2 != 0 {
            zero_point(hi, work)?;
        }
        return Ok(());
    }
    let mut k = i32::try_from(nb).map_err(|_| Error::Capacity)?
        - i32::try_from(db).map_err(|_| Error::Capacity)?;
    if k >= 0 {
        left(&mut f.w1, &f.raw0.d, k as usize, work)?;
        if cmp(&f.raw0.n, &f.w1, work)? == Ordering::Less {
            k -= 1;
        }
    } else {
        left(&mut f.w0, &f.raw0.n, k.unsigned_abs() as usize, work)?;
        if cmp(&f.w0, &f.raw0.d, work)? == Ordering::Less {
            k -= 1;
        }
    }
    let u = k - (p as i32 - 1);
    left(
        &mut f.w0,
        &f.raw0.n,
        if u < 0 { u.unsigned_abs() as usize } else { 0 },
        work,
    )?;
    left(
        &mut f.w1,
        &f.raw0.d,
        if u > 0 { u as usize } else { 0 },
        work,
    )?;
    let nbits = bits(&f.w0, work)?;
    let dbits = bits(&f.w1, work)?;
    if nbits < dbits {
        return Err(Error::Capacity);
    }
    let shift = nbits - dbits;
    if shift > p {
        return Err(Error::Capacity);
    }
    left(&mut f.w3, &f.w1, shift, work)?;
    clear(&mut f.q, work)?;
    for bit in (0..=shift).rev() {
        work.charge(1)?;
        if cmp(&f.w0, &f.w3, work)? != Ordering::Less {
            sub_in_place(&mut f.w0, &f.w3, work)?;
            f.q[bit / 64] |= 1 << (bit % 64);
        }
        right(&mut f.w3, 1, work)?;
    }
    if bits(&f.q, work)? != p {
        return Err(Error::Capacity);
    }
    let remainder = bits(&f.w0, work)? != 0;
    copy(&mut f.e1, &f.e0, work)?;
    let e1n = adjust(&mut f.e1, base_negative, k, work)?;
    if negative {
        if mask & 2 != 0 {
            emit(hi, &f.q, &f.e1, e1n, true, work)?;
        }
    } else if mask & 1 != 0 {
        emit(lo, &f.q, &f.e1, e1n, false, work)?;
    }
    copy(&mut f.e2, &f.e1, work)?;
    let mut e2n = e1n;
    if remainder {
        increment(&mut f.q, work)?;
        if bits(&f.q, work)? > p {
            right(&mut f.q, 1, work)?;
            e2n = adjust(&mut f.e2, e2n, 1, work)?;
        }
    }
    if negative {
        if mask & 1 != 0 {
            emit(lo, &f.q, &f.e2, e2n, true, work)?;
        }
    } else if mask & 2 != 0 {
        emit(hi, &f.q, &f.e2, e2n, false, work)?;
    }
    Ok(())
}

/// Caller stages exact N/D in raw0 and exact signed exponent in e0.
pub(in crate::bonds::enclosure) fn normalize(
    p: usize,
    negative: bool,
    base_negative: bool,
    lo: &mut Point,
    hi: &mut Point,
    f: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    normalize_mask(p, negative, base_negative, 3, lo, hi, f, work)
}

pub(in crate::bonds::enclosure) fn multiply(
    p: usize,
    a: &Point,
    b: &Point,
    lo: &mut Point,
    hi: &mut Point,
    f: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    precision(p)?;
    valid_point(a, p, work)?;
    valid_point(b, p, work)?;
    clear(&mut f.w2, work)?;
    let n = p / 64;
    work.charge(n.checked_mul(n).ok_or(Error::Capacity)?)?;
    for i in 0..n {
        let mut carry = 0_u128;
        for j in 0..n {
            let k = i + j;
            let v = u128::from(a.m[i]) * u128::from(b.m[j]) + u128::from(f.w2[k]) + carry;
            f.w2[k] = v as u64;
            carry = v >> 64;
        }
        f.w2[i + n] = carry as u64;
    }
    copy(&mut f.raw0.n, &f.w2, work)?;
    clear(&mut f.raw0.d, work)?;
    f.raw0.d[0] = 1;
    let en = signed_add(
        &mut f.e0,
        &a.k,
        a.flags & 2 != 0,
        &b.k,
        b.flags & 2 != 0,
        work,
    )?;
    let en = adjust(&mut f.e0, en, -2 * (p as i32 - 1), work)?;
    normalize(p, (a.flags ^ b.flags) & 1 != 0, en, lo, hi, f, work)
}

pub(in crate::bonds::enclosure) fn divide(
    p: usize,
    a: &Point,
    b: &Point,
    lo: &mut Point,
    hi: &mut Point,
    f: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    precision(p)?;
    valid_point(a, p, work)?;
    valid_point(b, p, work)?;
    if bits(&b.m, work)? == 0 {
        return Err(Error::DenominatorZero);
    }
    copy(&mut f.raw0.n, &a.m, work)?;
    copy(&mut f.raw0.d, &b.m, work)?;
    let bn = b.flags & 2 == 0 && bits(&b.k, work)? != 0;
    let en = signed_add(&mut f.e0, &a.k, a.flags & 2 != 0, &b.k, bn, work)?;
    normalize(p, (a.flags ^ b.flags) & 1 != 0, en, lo, hi, f, work)
}

fn aligned(
    dst: &mut [u64],
    a: &Point,
    k: &Point,
    upper: bool,
    p: usize,
    temp: &mut [u64],
    work: &mut Work,
) -> Result<bool> {
    if bits(&a.m, work)? == 0 {
        clear(dst, work)?;
        return Ok(false);
    }
    let an = a.flags & 2 == 0 && bits(&a.k, work)? != 0;
    let negative = signed_add(temp, &k.k, k.flags & 2 != 0, &a.k, an, work)?;
    if negative {
        return Err(Error::Capacity);
    }
    let gap = if cmp_small(temp, (p + 2) as u64, work)? == Ordering::Greater {
        p + 3
    } else {
        usize::try_from(temp[0]).map_err(|_| Error::Capacity)?
    };
    copy(dst, &a.m, work)?;
    let lost = sticky(&a.m, gap, work)?;
    right(dst, gap, work)?;
    let value_negative = a.flags & 1 != 0;
    if lost && (upper != value_negative) {
        increment(dst, work)?;
    }
    Ok(value_negative && bits(dst, work)? != 0)
}

/// Paper's coarse maximum-unit policy. Each endpoint contains the exact p-bit pair;
/// cancellation can be wider than a tightly rounded exact sum.
pub(in crate::bonds::enclosure) fn add(
    p: usize,
    a: &Point,
    b: &Point,
    lo: &mut Point,
    hi: &mut Point,
    f: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    precision(p)?;
    valid_point(a, p, work)?;
    valid_point(b, p, work)?;
    let az = bits(&a.m, work)? == 0;
    let bz = bits(&b.m, work)? == 0;
    if az && bz {
        zero_point(lo, work)?;
        zero_point(hi, work)?;
        return Ok(());
    }
    let k = if az {
        b
    } else if bz {
        a
    } else if cmp_signed(&a.k, a.flags & 2 != 0, &b.k, b.flags & 2 != 0, work)? == Ordering::Less {
        b
    } else {
        a
    };
    copy(&mut f.e0, &k.k, work)?;
    let en = adjust(&mut f.e0, k.flags & 2 != 0, -(p as i32 - 1), work)?;
    for upper in [false, true] {
        work.charge(1)?;
        let an = aligned(&mut f.w0, a, k, upper, p, &mut f.e1, work)?;
        let bn = aligned(&mut f.w1, b, k, upper, p, &mut f.e1, work)?;
        let negative = signed_add(&mut f.w2, &f.w0, an, &f.w1, bn, work)?;
        copy(&mut f.raw0.n, &f.w2, work)?;
        clear(&mut f.raw0.d, work)?;
        f.raw0.d[0] = 1;
        normalize_mask(p, negative, en, if upper { 2 } else { 1 }, lo, hi, f, work)?;
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) struct Export {
    pub(in crate::bonds::enclosure) lower: Option<u64>,
    pub(in crate::bonds::enclosure) upper: Option<u64>,
}
fn signed_bits(magnitude: Option<u64>, negative: bool) -> Option<u64> {
    magnitude.map(|x| if x != 0 && negative { x | (1 << 63) } else { x })
}
fn ieee_from_parts(mut significand: u64, mut k: i32) -> Option<u64> {
    if k < -1022 {
        return Some(significand);
    }
    if significand == 1 << 53 {
        significand >>= 1;
        k += 1;
    }
    if k > 1023 {
        return None;
    }
    Some(((k + 1023) as u64) << 52 | (significand - (1 << 52)))
}
pub(in crate::bonds::enclosure) fn export(p: usize, a: &Point, work: &mut Work) -> Result<Export> {
    precision(p)?;
    valid_point(a, p, work)?;
    if bits(&a.m, work)? == 0 {
        return Ok(Export {
            lower: Some(0),
            upper: Some(0),
        });
    }
    work.charge(8)?;
    let negative = a.flags & 1 != 0;
    let kn = a.flags & 2 != 0;
    let (down, up) = if !kn && cmp_small(&a.k, 1023, work)? == Ordering::Greater {
        (Some(0x7fef_ffff_ffff_ffff), None)
    } else if kn && cmp_small(&a.k, 1074, work)? == Ordering::Greater {
        (Some(0), Some(1))
    } else {
        let mag = i32::try_from(a.k[0]).map_err(|_| Error::Capacity)?;
        let k = if kn { -mag } else { mag };
        let unit = (k - 52).max(-1074);
        let shift = usize::try_from(unit - k + p as i32 - 1).map_err(|_| Error::Capacity)?;
        let word = shift / 64;
        let rem = shift % 64;
        work.charge(2)?;
        let mut q = a.m.get(word).copied().unwrap_or(0) >> rem;
        if rem != 0 {
            q |= a.m.get(word + 1).copied().unwrap_or(0) << (64 - rem);
        }
        let lost = sticky(&a.m, shift, work)?;
        (
            ieee_from_parts(q, k),
            ieee_from_parts(q + u64::from(lost), k),
        )
    };
    if negative {
        Ok(Export {
            lower: signed_bits(up, true),
            upper: signed_bits(down, true),
        })
    } else {
        Ok(Export {
            lower: down,
            upper: up,
        })
    }
}

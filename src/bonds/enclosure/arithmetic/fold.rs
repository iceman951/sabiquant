//! Complete bounded exact sum on the prescribed source-role or dyadic grid.
//!
//! Callers must already have completed financial admission and selected the
//! correct source/AI/basis operands. This module proves only their exact sum.
//! A session borrows one Frame and Work; each step consumes the session. An
//! ignored failed step cannot leave a finishable session or certify a partial sum.
#![forbid(unsafe_code)]

use super::scalar::{E, Frame, R, W, Work};
use super::source;
use std::cmp::Ordering;

/// Small checked failures, never an invalid financial yield diagnosis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Error {
    Capacity,
    DenominatorZero,
    DenominatorRoleMismatch,
    Incomplete,
    Nonfinite,
    RoleLimit,
    WorkLimit,
    WrongMode,
}

impl From<source::Error> for Error {
    fn from(value: source::Error) -> Self {
        match value {
            source::Error::Nonfinite => Self::Nonfinite,
            source::Error::DenominatorZero => Self::DenominatorZero,
            source::Error::Capacity => Self::Capacity,
            source::Error::WorkLimit => Self::WorkLimit,
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Raw0 contains signed N/D*2^exponent and E0 agrees with the explicit exponent.
/// Zero has nonnegative sign; its declared final exponent is retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::bonds::enclosure) struct StagedValue {
    pub(in crate::bonds::enclosure) negative: bool,
    pub(in crate::bonds::enclosure) exponent: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Generic,
    Named,
}

/// Unclonable, consuming state; the large numerical storage remains borrowed.
pub(in crate::bonds::enclosure) struct Session<'a> {
    frame: &'a mut Frame,
    work: &'a mut Work,
    expected_terms: u64,
    terms: u64,
    expected_roles: u8,
    roles: u8,
    negative: bool,
    mode: Mode,
}

fn charge(work: &mut Work, count: usize) -> Result<()> {
    let count = u64::try_from(count).map_err(|_| Error::Capacity)?;
    let next = work.used.checked_add(count).ok_or(Error::WorkLimit)?;
    if next > work.max {
        return Err(Error::WorkLimit);
    }
    work.used = next;
    Ok(())
}

fn bits(words: &[u64], work: &mut Work) -> Result<usize> {
    charge(work, words.len())?;
    for (i, &word) in words.iter().enumerate().rev() {
        if word != 0 {
            return Ok(i * 64 + 64 - word.leading_zeros() as usize);
        }
    }
    Ok(0)
}

fn word_len(words: &[u64], work: &mut Work) -> Result<usize> {
    Ok(bits(words, work)?.div_ceil(64))
}

fn compare(a: &[u64], b: &[u64], work: &mut Work) -> Result<Ordering> {
    charge(work, a.len().max(b.len()) + 2)?;
    for i in (0..a.len().max(b.len())).rev() {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return Ok(x.cmp(&y));
        }
    }
    Ok(Ordering::Equal)
}

fn difference(a: &mut [u64], b: &[u64], reverse: bool, work: &mut Work) -> Result<()> {
    charge(work, 5 * a.len() + 4)?;
    if a.len() != b.len() {
        return Err(Error::Capacity);
    }
    let mut borrow = false;
    for (x, y) in a.iter_mut().zip(b) {
        let (left, right) = if reverse { (*y, *x) } else { (*x, *y) };
        let (first, b1) = left.overflowing_sub(right);
        let (second, b2) = first.overflowing_sub(u64::from(borrow));
        *x = second;
        borrow = b1 || b2;
    }
    if borrow {
        return Err(Error::Capacity);
    }
    Ok(())
}

fn right_one(words: &mut [u64], work: &mut Work) -> Result<()> {
    charge(work, 3 * words.len() + 2)?;
    let mut carry = 0;
    for word in words.iter_mut().rev() {
        let next = *word << 63;
        *word = (*word >> 1) | carry;
        carry = next;
    }
    Ok(())
}

fn left_copy(input: &[u64], output: &mut [u64; W], shift: usize, work: &mut Work) -> Result<()> {
    charge(work, W + 8 * input.len() + 8)?;
    if input.len() > R || shift >= 64 * R {
        return Err(Error::Capacity);
    }
    output.fill(0);
    let offset = shift / 64;
    let tail = shift % 64;
    for (i, &word) in input.iter().enumerate() {
        if word == 0 {
            continue;
        }
        *output.get_mut(i + offset).ok_or(Error::Capacity)? |= word << tail;
        if tail != 0 {
            let carry = word >> (64 - tail);
            if carry != 0 {
                *output.get_mut(i + offset + 1).ok_or(Error::Capacity)? |= carry;
            }
        }
    }
    Ok(())
}

fn copy_to_raw(input: &[u64; W], output: &mut [u64; R], work: &mut Work) -> Result<()> {
    charge(work, W + R)?;
    if input[R..].iter().any(|&v| v != 0) {
        return Err(Error::Capacity);
    }
    output.fill(0);
    for (to, from) in output.iter_mut().zip(input) {
        *to = *from;
    }
    Ok(())
}

/// Fixed-capacity grade-school product; only active limbs are traversed.
fn multiply(a: &[u64], b: &[u64], out: &mut [u64; W], work: &mut Work) -> Result<()> {
    charge(work, 2)?;
    if a.len() > W || b.len() > W {
        return Err(Error::Capacity);
    }
    let an = word_len(a, work)?;
    let bn = word_len(b, work)?;
    charge(work, W + 8 * an * bn + 4 * W + 8)?;
    out.fill(0);
    for (i, &x) in a[..an].iter().enumerate() {
        let mut carry = 0_u128;
        for (j, &y) in b[..bn].iter().enumerate() {
            let at = i + j;
            let destination = out.get_mut(at).ok_or(Error::Capacity)?;
            let product = u128::from(x) * u128::from(y) + u128::from(*destination) + carry;
            *destination = product as u64;
            carry = product >> 64;
        }
        // Carry propagation is capped by W, with each visit charged before use.
        for word in out.iter_mut().skip(i + bn) {
            if carry == 0 {
                break;
            }
            charge(work, 4)?;
            let sum = u128::from(*word) + carry;
            *word = sum as u64;
            carry = sum >> 64;
        }
        if carry != 0 {
            return Err(Error::Capacity);
        }
    }
    Ok(())
}

/// Exact common denominator / term denominator in Q; no absolute exponent work.
fn quotient(frame: &mut Frame, work: &mut Work) -> Result<()> {
    let nb = bits(&frame.raw1.d, work)?;
    let db = bits(&frame.raw0.d, work)?;
    charge(work, 4)?;
    if db == 0 {
        return Err(Error::DenominatorZero);
    }
    if nb < db {
        return Err(Error::DenominatorRoleMismatch);
    }
    let shift = nb - db;
    let width = nb.div_ceil(64);
    charge(work, 2 * W + R)?;
    frame.w0.fill(0);
    frame.q.fill(0);
    for (to, from) in frame.w0.iter_mut().zip(&frame.raw1.d) {
        *to = *from;
    }
    left_copy(&frame.raw0.d, &mut frame.w1, shift, work)?;
    for bit in (0..=shift).rev() {
        charge(work, 2)?;
        if compare(&frame.w0[..width], &frame.w1[..width], work)? != Ordering::Less {
            difference(&mut frame.w0[..width], &frame.w1[..width], false, work)?;
            charge(work, 2)?;
            *frame.q.get_mut(bit / 64).ok_or(Error::Capacity)? |= 1_u64 << (bit % 64);
        }
        right_one(&mut frame.w1[..width], work)?;
    }
    charge(work, W)?;
    if frame.w0.iter().any(|&v| v != 0) {
        return Err(Error::DenominatorRoleMismatch);
    }
    Ok(())
}

fn accumulate(
    frame: &mut Frame,
    old_negative: bool,
    negative: bool,
    work: &mut Work,
) -> Result<bool> {
    charge(work, W)?;
    if frame.w2[R..].iter().any(|&v| v != 0) {
        return Err(Error::Capacity);
    }
    if frame.w2[..R].iter().all(|&v| v == 0) {
        return Ok(old_negative);
    }
    if old_negative == negative {
        charge(work, 4 * R + 2)?;
        let mut carry = 0_u128;
        for (a, b) in frame.raw1.n.iter_mut().zip(&frame.w2) {
            let sum = u128::from(*a) + u128::from(*b) + carry;
            *a = sum as u64;
            carry = sum >> 64;
        }
        if carry != 0 {
            return Err(Error::Capacity);
        }
        return Ok(old_negative);
    }
    let order = compare(&frame.raw1.n, &frame.w2[..R], work)?;
    let reverse = order == Ordering::Less;
    difference(&mut frame.raw1.n, &frame.w2[..R], reverse, work)?;
    Ok(if order == Ordering::Equal {
        false
    } else if reverse {
        negative
    } else {
        old_negative
    })
}

impl<'a> Session<'a> {
    /// Debit outer producer work without releasing the consuming fold session.
    /// A rejected debit consumes self, so a stopped stream cannot finish.
    pub(in crate::bonds::enclosure) fn debit(self, units: u64) -> Result<Self> {
        let next = self.work.used.checked_add(units).ok_or(Error::WorkLimit)?;
        if next > self.work.max {
            return Err(Error::WorkLimit);
        }
        self.work.used = next;
        Ok(self)
    }

    /// Inspect the same shared meter while its mutable borrow remains held.
    pub(in crate::bonds::enclosure) fn used(&self) -> u64 {
        self.work.used
    }

    fn new(
        frame: &'a mut Frame,
        work: &'a mut Work,
        mode: Mode,
        roles: u8,
        terms: u64,
    ) -> Result<Self> {
        charge(work, 2 * R + 10)?;
        if roles > 5 {
            return Err(Error::RoleLimit);
        }
        frame.raw1.n.fill(0);
        frame.raw1.d.fill(0);
        frame.raw1.controls.fill(0);
        frame.raw1.d[0] = 1;
        Ok(Self {
            frame,
            work,
            mode,
            expected_roles: roles,
            roles: 0,
            expected_terms: terms,
            terms: 0,
            negative: false,
        })
    }

    /// Begin an exact represented dyadic sum on the fixed 2^-1074 grid.
    pub(in crate::bonds::enclosure) fn generic(
        frame: &'a mut Frame,
        work: &'a mut Work,
        terms: u64,
    ) -> Result<Self> {
        Self::new(frame, work, Mode::Generic, 0, terms)
    }

    /// Begin a named-source sum with at most five fixed semantic denominator roles.
    pub(in crate::bonds::enclosure) fn named(
        frame: &'a mut Frame,
        work: &'a mut Work,
        roles: u8,
        terms: u64,
    ) -> Result<Self> {
        Self::new(frame, work, Mode::Named, roles, terms)
    }

    fn next_term(&mut self) -> Result<()> {
        charge(self.work, 3)?;
        if self.roles != self.expected_roles || self.terms >= self.expected_terms {
            return Err(Error::Incomplete);
        }
        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
        charge(self.work, 2)?;
        self.terms = self.terms.checked_add(1).ok_or(Error::Capacity)?;
        Ok(())
    }

    /// Add a fixed role BEFORE any coefficient; consume self even on failure.
    pub(in crate::bonds::enclosure) fn role(mut self, denominator: &[u64]) -> Result<Self> {
        charge(self.work, 6)?;
        if self.mode != Mode::Named {
            return Err(Error::WrongMode);
        }
        if self.terms != 0 || self.roles >= self.expected_roles {
            return Err(Error::Incomplete);
        }
        if denominator.len() > R {
            return Err(Error::Capacity);
        }
        let db = bits(denominator, self.work)?;
        if db == 0 {
            return Err(Error::DenominatorZero);
        }
        if db > 148 {
            return Err(Error::Capacity);
        }
        source::source(&[0], denominator, false, self.frame, self.work)?;
        multiply(
            &self.frame.raw1.d,
            &self.frame.raw0.d,
            &mut self.frame.w2,
            self.work,
        )?;
        copy_to_raw(&self.frame.w2, &mut self.frame.raw1.d, self.work)?;
        charge(self.work, 2)?;
        self.roles += 1;
        Ok(self)
    }

    /// Add an exact named coefficient whose denominator divides the common product.
    /// None denotes an integer/unit role, such as the separate Canadian par.
    pub(in crate::bonds::enclosure) fn rational(
        mut self,
        role: Option<usize>,
        numerator: &[u64],
        denominator: &[u64],
        negative: bool,
    ) -> Result<Self> {
        charge(self.work, 6)?;
        if self.mode != Mode::Named {
            return Err(Error::WrongMode);
        }
        self.next_term()?;
        if role.is_some_and(|i| i >= usize::from(self.roles)) {
            return Err(Error::RoleLimit);
        }
        if numerator.len() > R || denominator.len() > R {
            return Err(Error::Capacity);
        }
        let nb = bits(numerator, self.work)?;
        let db = bits(denominator, self.work)?;
        if nb > 160 || db > 148 {
            return Err(Error::Capacity);
        }
        if db == 0 {
            return Err(Error::DenominatorZero);
        }
        if role.is_none() && (db != 1 || denominator[0] != 1) {
            return Err(Error::DenominatorRoleMismatch);
        }
        source::source(numerator, denominator, negative, self.frame, self.work)?;
        quotient(self.frame, self.work)?;
        multiply(
            &self.frame.raw0.n,
            &self.frame.q,
            &mut self.frame.w2,
            self.work,
        )?;
        self.negative = accumulate(self.frame, self.negative, negative, self.work)?;
        self.commit()?;
        Ok(self)
    }

    fn grid_inner(mut self, negative: bool, significand: u64, shift: u32) -> Result<Self> {
        charge(self.work, 8)?;
        if significand >= 1_u64 << 53 || shift > 2045 {
            return Err(Error::Capacity);
        }
        charge(self.work, W + 8)?;
        self.frame.w2.fill(0);
        let at = shift as usize / 64;
        let tail = shift % 64;
        *self.frame.w2.get_mut(at).ok_or(Error::Capacity)? = significand << tail;
        if tail != 0 {
            *self.frame.w2.get_mut(at + 1).ok_or(Error::Capacity)? = significand >> (64 - tail);
        }
        self.negative = accumulate(self.frame, self.negative, negative, self.work)?;
        self.commit()?;
        Ok(self)
    }

    /// Add the literal decoded source-grid representation used by frozen streams.
    pub(in crate::bonds::enclosure) fn grid(
        mut self,
        negative: bool,
        significand: u64,
        shift: u32,
    ) -> Result<Self> {
        charge(self.work, 2)?;
        if self.mode != Mode::Generic {
            return Err(Error::WrongMode);
        }
        self.next_term()?;
        self.grid_inner(negative, significand, shift)
    }

    /// Decode actual prepared binary bits then use the same exact grid operation.
    pub(in crate::bonds::enclosure) fn binary(mut self, bits: u64, negate: bool) -> Result<Self> {
        charge(self.work, 2)?;
        if self.mode != Mode::Generic {
            return Err(Error::WrongMode);
        }
        self.next_term()?;
        let staged = source::binary(bits, self.frame, self.work)?;
        let shift = u32::try_from(staged.exponent + 1074).map_err(|_| Error::Capacity)?;
        let significand = self.frame.raw0.n[0];
        self.grid_inner(staged.negative != negate, significand, shift)
    }

    fn complete(&mut self, mode: Mode) -> Result<()> {
        charge(self.work, 4)?;
        if self.mode != mode {
            return Err(Error::WrongMode);
        }
        if self.roles != self.expected_roles || self.terms != self.expected_terms {
            return Err(Error::Incomplete);
        }
        Ok(())
    }

    fn stage_exponent(&mut self, exponent: i32) -> Result<StagedValue> {
        charge(self.work, E + R + 4)?;
        self.frame.e0.fill(0);
        self.frame.e0[0] = u64::from(exponent.unsigned_abs());
        let zero = self.frame.raw0.n.iter().all(|&v| v == 0);
        Ok(StagedValue {
            negative: self.negative && !zero,
            exponent,
        })
    }

    /// Finish the whole named sum at exponent zero; no partial sum can finish.
    pub(in crate::bonds::enclosure) fn finish_named(mut self) -> Result<StagedValue> {
        self.complete(Mode::Named)?;
        charge(self.work, 2 * R + 2)?;
        for (to, from) in self.frame.raw0.n.iter_mut().zip(&self.frame.raw1.n) {
            *to = *from;
        }
        for (to, from) in self.frame.raw0.d.iter_mut().zip(&self.frame.raw1.d) {
            *to = *from;
        }
        self.frame.raw0.controls.fill(0);
        self.stage_exponent(0)
    }

    /// Apply exact100/face to the completed fixed-grid sum, preserving range.
    /// final_exponent=-1074-face_binary_exponent; face_significand is positive.
    pub(in crate::bonds::enclosure) fn finish_generic(
        mut self,
        face_significand: u64,
        final_exponent: i32,
    ) -> Result<StagedValue> {
        self.complete(Mode::Generic)?;
        charge(self.work, 2 * R + 8)?;
        if face_significand == 0
            || face_significand >= 1_u64 << 53
            || !(-2045..=0).contains(&final_exponent)
        {
            return Err(Error::Capacity);
        }
        self.frame.raw0.n.fill(0);
        self.frame.raw0.d.fill(0);
        self.frame.raw0.controls.fill(0);
        self.frame.raw0.n[0] = 100;
        self.frame.raw0.d[0] = face_significand;
        multiply(
            &self.frame.raw1.n,
            &self.frame.raw0.n,
            &mut self.frame.w2,
            self.work,
        )?;
        copy_to_raw(&self.frame.w2, &mut self.frame.raw0.n, self.work)?;
        self.stage_exponent(final_exponent)
    }
}

//! Fallible fixed-slot storage for the producer's directed interval arithmetic.
#![forbid(unsafe_code)]

use super::scalar::{self, Frame, Point, Work, ZERO_POINT};
use super::source;
use std::cmp::Ordering;

pub(in crate::bonds::enclosure) const SLOT_COUNT: usize = 24;
pub(in crate::bonds::enclosure) const UNIT: usize = 21;
pub(in crate::bonds::enclosure) const RECIPROCAL: usize = 22;
pub(in crate::bonds::enclosure) const CANDIDATE: usize = 23;

/// Actual two-Point typed interval. Deliberately neither Copy nor Clone.
pub(in crate::bonds::enclosure) struct Interval {
    pub(in crate::bonds::enclosure) lower: Point,
    pub(in crate::bonds::enclosure) upper: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum Bound {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum EndpointSign {
    Negative,
    Zero,
    Positive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bonds::enclosure) enum ExponentView {
    ZeroValue,
    Small(i32),
    BeyondI32 { negative: bool },
}

/// Small facts about a borrowed Point; never an owned large endpoint view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::bonds::enclosure) struct EndpointFacts {
    pub(in crate::bonds::enclosure) sign: EndpointSign,
    pub(in crate::bonds::enclosure) exponent: ExponentView,
    /// Whether the nonzero absolute value is an exact power of two.
    pub(in crate::bonds::enclosure) power_of_two: bool,
}

#[derive(Debug)]
pub(in crate::bonds::enclosure) enum Error {
    Scalar(scalar::Error),
    Slots,
    UninitializedUnit,
    ZeroDivisorInterval,
    Reserve {
        attempts: usize,
        cause: std::collections::TryReserveError,
    },
}

impl From<scalar::Error> for Error {
    fn from(value: scalar::Error) -> Self {
        Self::Scalar(value)
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Small returned handle; no scalar Frame is constructed or returned here.
pub(in crate::bonds::enclosure) struct SlotArena {
    slots: Vec<Interval>,
    reservation_attempts: usize,
    post_reservation_capacity: usize,
    growth_events: usize,
    unit_precision: Option<usize>,
}

fn reserve_slots(slots: &mut Vec<Interval>, requested: usize, attempts: &mut usize) -> Result<()> {
    *attempts = attempts.checked_add(1).ok_or(scalar::Error::Capacity)?;
    slots
        .try_reserve_exact(requested)
        .map_err(|cause| Error::Reserve {
            attempts: *attempts,
            cause,
        })
}

fn charge(work: &mut Work, count: u64) -> Result<()> {
    let next = work
        .used
        .checked_add(count)
        .ok_or(scalar::Error::WorkLimit)?;
    if next > work.max {
        return Err(scalar::Error::WorkLimit.into());
    }
    work.used = next;
    Ok(())
}

fn copy_point(out: &mut Point, input: &Point, work: &mut Work) -> Result<()> {
    charge(work, (scalar::M + scalar::E + 1) as u64)?;
    out.m.copy_from_slice(&input.m);
    out.k.copy_from_slice(&input.k);
    out.flags = input.flags;
    Ok(())
}

fn endpoint(interval: &Interval, bound: Bound) -> &Point {
    match bound {
        Bound::Lower => &interval.lower,
        Bound::Upper => &interval.upper,
    }
}

fn endpoint_mut(interval: &mut Interval, bound: Bound) -> &mut Point {
    match bound {
        Bound::Lower => &mut interval.lower,
        Bound::Upper => &mut interval.upper,
    }
}

fn inspect_point(p: usize, point: &Point, work: &mut Work) -> Result<EndpointFacts> {
    if ![64, 128, 256, 512, 1024].contains(&p) {
        return Err(scalar::Error::Precision.into());
    }
    charge(work, (3 * scalar::M + 2 * scalar::E + 32) as u64)?;
    let magnitude_bits = point
        .m
        .iter()
        .rposition(|&word| word != 0)
        .map_or(0, |index| {
            index * 64 + 64 - point.m[index].leading_zeros() as usize
        });
    let exponent_zero = point.k.iter().all(|&word| word == 0);
    if point.flags & !3 != 0
        || (magnitude_bits != 0 && magnitude_bits != p)
        || (magnitude_bits == 0 && (!exponent_zero || point.flags != 0))
        || (exponent_zero && point.flags & 2 != 0)
    {
        return Err(scalar::Error::Malformed.into());
    }
    if magnitude_bits == 0 {
        return Ok(EndpointFacts {
            sign: EndpointSign::Zero,
            exponent: ExponentView::ZeroValue,
            power_of_two: false,
        });
    }
    let negative = point.flags & 2 != 0;
    let exponent = if point.k.iter().skip(1).any(|&word| word != 0) {
        ExponentView::BeyondI32 { negative }
    } else if negative {
        if point.k[0] <= 1_u64 << 31 {
            ExponentView::Small(-(point.k[0] as i64) as i32)
        } else {
            ExponentView::BeyondI32 { negative: true }
        }
    } else if point.k[0] <= i32::MAX as u64 {
        ExponentView::Small(point.k[0] as i32)
    } else {
        ExponentView::BeyondI32 { negative: false }
    };
    let unit_word = (p - 1) / 64;
    let unit_bit = 1_u64 << ((p - 1) % 64);
    let power_of_two = point
        .m
        .iter()
        .enumerate()
        .all(|(index, &word)| word == if index == unit_word { unit_bit } else { 0 });
    Ok(EndpointFacts {
        sign: if point.flags & 1 != 0 {
            EndpointSign::Negative
        } else {
            EndpointSign::Positive
        },
        exponent,
        power_of_two,
    })
}

fn point_cmp(a: &Point, b: &Point, work: &mut Work) -> Result<Ordering> {
    // Full finite fixed-width scans are prepaid; this is prototype accounting,
    // not an adopted complete financial-producer cost table.
    charge(work, (2 * scalar::M + 2 * scalar::E + 4) as u64)?;
    let az = a.m.iter().all(|&word| word == 0);
    let bz = b.m.iter().all(|&word| word == 0);
    let an = a.flags & 1 != 0;
    let bn = b.flags & 1 != 0;
    if az && bz {
        return Ok(Ordering::Equal);
    }
    if az {
        return Ok(if bn {
            Ordering::Greater
        } else {
            Ordering::Less
        });
    }
    if bz {
        return Ok(if an {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    if an != bn {
        return Ok(if an {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let akn = a.flags & 2 != 0;
    let bkn = b.flags & 2 != 0;
    let exponent_order = if akn != bkn {
        if akn {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    } else {
        let order = a.k.iter().rev().cmp(b.k.iter().rev());
        if akn { order.reverse() } else { order }
    };
    let order = if exponent_order == Ordering::Equal {
        a.m.iter().rev().cmp(b.m.iter().rev())
    } else {
        exponent_order
    };
    Ok(if an { order.reverse() } else { order })
}

impl SlotArena {
    /// Inspect only the COMPLETE final price interval after basis and units.
    /// The bool retains the exact-singleton fact even when its exponent rules
    /// out both preferred finite representations. No intermediate equality is
    /// authorized as a final-price proof by this arithmetic helper.
    ///
    /// Eligible mantissas are copied into borrowed Raw0 with D1 and signed E0;
    /// no Point, large integer or second Frame is constructed or returned.
    pub(in crate::bonds::enclosure) fn stage_final_point(
        &self,
        p: usize,
        input: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<(bool, Option<source::Seed>)> {
        let interval = self.slots.get(input).ok_or(Error::Slots)?;
        let facts = inspect_point(p, &interval.lower, work)?;
        inspect_point(p, &interval.upper, work)?;
        if point_cmp(&interval.lower, &interval.upper, work)? != Ordering::Equal {
            return Ok((false, None));
        }
        let k = match facts.exponent {
            ExponentView::ZeroValue => 0,
            ExponentView::Small(k) if (-1074..=1023).contains(&k) => k,
            _ => return Ok((true, None)),
        };
        charge(work, (2 * scalar::R + scalar::M + scalar::E + 12) as u64)?;
        let exponent = if facts.sign == EndpointSign::Zero {
            0
        } else {
            k - (i32::try_from(p).map_err(|_| scalar::Error::Capacity)? - 1)
        };
        frame.raw0.n.fill(0);
        frame.raw0.d.fill(0);
        frame.raw0.controls.fill(0);
        frame.e0.fill(0);
        frame.raw0.n[..scalar::M].copy_from_slice(&interval.lower.m);
        frame.raw0.d[0] = 1;
        frame.e0[0] = u64::from(exponent.unsigned_abs());
        Ok((
            true,
            Some(source::Seed {
                negative: facts.sign == EndpointSign::Negative,
                exponent,
            }),
        ))
    }

    /// Consume full staged Raw0 N/D and immutable signed E0 through actual scalar
    /// Normalize. Output is a stable algorithm/global slot, never unit or L15.
    pub(in crate::bonds::enclosure) fn normalize_staged(
        &mut self,
        index: usize,
        p: usize,
        negative: bool,
        base_negative: bool,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        if index >= SLOT_COUNT || index == UNIT || index == CANDIDATE {
            return Err(Error::Slots);
        }
        let out = self.slots.get_mut(index).ok_or(Error::Slots)?;
        scalar::normalize(
            p,
            negative,
            base_negative,
            &mut out.lower,
            &mut out.upper,
            frame,
            work,
        )?;
        Ok(())
    }

    /// A stable copy cannot depend on L15: it is clobbered by the next scalar
    /// operation. Existing immediate internal candidate copies remain private.
    fn validate_copy_indices(source: usize, destination: usize) -> Result<()> {
        if source >= SLOT_COUNT
            || destination >= SLOT_COUNT
            || source == destination
            || source == CANDIDATE
            || destination == CANDIDATE
            || destination == UNIT
        {
            return Err(Error::Slots);
        }
        Ok(())
    }

    pub(in crate::bonds::enclosure) fn copy_endpoint(
        &mut self,
        p: usize,
        source: usize,
        source_bound: Bound,
        destination: usize,
        destination_bound: Bound,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_copy_indices(source, destination)?;
        let [source, destination] = self
            .slots
            .get_disjoint_mut([source, destination])
            .map_err(|_| Error::Slots)?;
        let input = endpoint(source, source_bound);
        inspect_point(p, input, work)?;
        copy_point(endpoint_mut(destination, destination_bound), input, work)
    }

    pub(in crate::bonds::enclosure) fn copy_interval(
        &mut self,
        p: usize,
        source: usize,
        destination: usize,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_copy_indices(source, destination)?;
        let [source, destination] = self
            .slots
            .get_disjoint_mut([source, destination])
            .map_err(|_| Error::Slots)?;
        inspect_point(p, &source.lower, work)?;
        inspect_point(p, &source.upper, work)?;
        copy_point(&mut destination.lower, &source.lower, work)?;
        copy_point(&mut destination.upper, &source.upper, work)
    }

    /// Candidate facts may be read only immediately after a successful scalar
    /// call and before its next clobber; no stable algorithm input is established.
    pub(in crate::bonds::enclosure) fn inspect_endpoint(
        &self,
        p: usize,
        index: usize,
        bound: Bound,
        work: &mut Work,
    ) -> Result<EndpointFacts> {
        let interval = self.slots.get(index).ok_or(Error::Slots)?;
        inspect_point(p, endpoint(interval, bound), work)
    }
    #[inline(never)]
    pub(in crate::bonds::enclosure) fn try_new() -> Result<Self> {
        let mut slots = Vec::new();
        let mut reservation_attempts = 0;
        reserve_slots(&mut slots, SLOT_COUNT, &mut reservation_attempts)?;
        let post_reservation_capacity = slots.capacity();
        let mut growth_events = 0;
        for _ in 0..SLOT_COUNT {
            let before = slots.capacity();
            slots.push(Interval {
                lower: ZERO_POINT,
                upper: ZERO_POINT,
            });
            growth_events += usize::from(slots.capacity() != before);
        }
        Ok(Self {
            slots,
            reservation_attempts,
            post_reservation_capacity,
            growth_events,
            unit_precision: None,
        })
    }

    pub(in crate::bonds::enclosure) fn storage_facts(&self) -> (usize, usize, usize, usize) {
        (
            self.slots.len(),
            self.post_reservation_capacity,
            self.reservation_attempts,
            self.growth_events,
        )
    }

    pub(in crate::bonds::enclosure) fn slot(&self, index: usize) -> Option<&Interval> {
        self.slots.get(index)
    }

    pub(in crate::bonds::enclosure) fn initialize_unit(
        &mut self,
        p: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        self.unit_precision = None;
        self.normalize_slot(UNIT, p, 1, 1, false, 0, frame, work)?;
        self.unit_precision = Some(p);
        Ok(())
    }

    /// Setup uses the real Raw normalization operation with a borrowed Frame.
    #[allow(
        clippy::too_many_arguments,
        reason = "private borrowed setup avoids large values"
    )]
    fn normalize_slot(
        &mut self,
        index: usize,
        p: usize,
        n: u64,
        d: u64,
        negative: bool,
        exponent: i32,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        let out = self.slots.get_mut(index).ok_or(Error::Slots)?;
        charge(work, (2 * scalar::R + scalar::E + 2) as u64)?;
        frame.raw0.n.fill(0);
        frame.raw0.d.fill(0);
        frame.e0.fill(0);
        frame.raw0.n[0] = n;
        frame.raw0.d[0] = d;
        frame.e0[0] = u64::from(exponent.unsigned_abs());
        scalar::normalize(
            p,
            negative,
            exponent < 0,
            &mut out.lower,
            &mut out.upper,
            frame,
            work,
        )?;
        Ok(())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "private borrowed setup avoids large values"
    )]
    pub(in crate::bonds::enclosure) fn set_ratio(
        &mut self,
        index: usize,
        p: usize,
        n: u64,
        d: u64,
        negative: bool,
        exponent: i32,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        if index >= SLOT_COUNT || index == UNIT || index == CANDIDATE {
            return Err(Error::Slots);
        }
        self.normalize_slot(index, p, n, d, negative, exponent, frame, work)
    }

    pub(in crate::bonds::enclosure) fn set_integer_bounds(
        &mut self,
        index: usize,
        p: usize,
        lower: i32,
        upper: i32,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        if index >= SLOT_COUNT || index == UNIT || index == CANDIDATE || lower > upper {
            return Err(Error::Slots);
        }
        self.normalize_slot(
            index,
            p,
            u64::from(lower.unsigned_abs()),
            1,
            lower < 0,
            0,
            frame,
            work,
        )?;
        self.normalize_slot(
            CANDIDATE,
            p,
            u64::from(upper.unsigned_abs()),
            1,
            upper < 0,
            0,
            frame,
            work,
        )?;
        let [out, candidate] = self
            .slots
            .get_disjoint_mut([index, CANDIDATE])
            .map_err(|_| Error::Slots)?;
        copy_point(&mut out.upper, &candidate.upper, work)
    }

    fn validate_binary_indices(a: usize, b: usize, out: usize) -> Result<()> {
        if [a, b, out]
            .iter()
            .any(|&index| index >= SLOT_COUNT || index == CANDIDATE)
            || out == UNIT
            || out == a
            || out == b
        {
            return Err(Error::Slots);
        }
        Ok(())
    }

    pub(in crate::bonds::enclosure) fn multiply(
        &mut self,
        p: usize,
        a: usize,
        b: usize,
        out: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_binary_indices(a, b, out)?;
        if a == b {
            let [a, candidate, out] = self
                .slots
                .get_disjoint_mut([a, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            multiply_inputs(p, a, a, candidate, out, frame, work)
        } else {
            let [a, b, candidate, out] = self
                .slots
                .get_disjoint_mut([a, b, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            multiply_inputs(p, a, b, candidate, out, frame, work)
        }
    }

    pub(in crate::bonds::enclosure) fn add(
        &mut self,
        p: usize,
        a: usize,
        b: usize,
        out: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_binary_indices(a, b, out)?;
        if a == b {
            let [a, candidate, out] = self
                .slots
                .get_disjoint_mut([a, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            add_inputs(p, a, a, candidate, out, frame, work)
        } else {
            let [a, b, candidate, out] = self
                .slots
                .get_disjoint_mut([a, b, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            add_inputs(p, a, b, candidate, out, frame, work)
        }
    }

    pub(in crate::bonds::enclosure) fn reciprocal(
        &mut self,
        p: usize,
        input: usize,
        out: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_binary_indices(input, UNIT, out)?;
        if self.unit_precision != Some(p) {
            return Err(Error::UninitializedUnit);
        }
        if input == UNIT {
            let [unit, candidate, out] = self
                .slots
                .get_disjoint_mut([UNIT, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            reciprocal_inputs(p, unit, unit, candidate, out, frame, work)
        } else {
            let [input, unit, candidate, out] = self
                .slots
                .get_disjoint_mut([input, UNIT, CANDIDATE, out])
                .map_err(|_| Error::Slots)?;
            reciprocal_inputs(p, input, unit, candidate, out, frame, work)
        }
    }

    pub(in crate::bonds::enclosure) fn divide(
        &mut self,
        p: usize,
        a: usize,
        b: usize,
        out: usize,
        frame: &mut Frame,
        work: &mut Work,
    ) -> Result<()> {
        Self::validate_binary_indices(a, b, out)?;
        if [a, b, out].contains(&RECIPROCAL) {
            return Err(Error::Slots);
        }
        self.reciprocal(p, b, RECIPROCAL, frame, work)?;
        self.multiply(p, a, RECIPROCAL, out, frame, work)
    }

    pub(in crate::bonds::enclosure) fn export(
        &self,
        p: usize,
        input: usize,
        work: &mut Work,
    ) -> Result<(Option<u64>, Option<u64>)> {
        let interval = self.slots.get(input).ok_or(Error::Slots)?;
        let lower = scalar::export(p, &interval.lower, work)?;
        let upper = scalar::export(p, &interval.upper, work)?;
        Ok((lower.lower, upper.upper))
    }
}

fn multiply_inputs(
    p: usize,
    a: &Interval,
    b: &Interval,
    candidate: &mut Interval,
    out: &mut Interval,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    for (position, (aa, bb)) in [
        (&a.lower, &b.lower),
        (&a.lower, &b.upper),
        (&a.upper, &b.lower),
        (&a.upper, &b.upper),
    ]
    .into_iter()
    .enumerate()
    {
        scalar::multiply(
            p,
            aa,
            bb,
            &mut candidate.lower,
            &mut candidate.upper,
            frame,
            work,
        )?;
        if position == 0 || point_cmp(&candidate.lower, &out.lower, work)? == Ordering::Less {
            copy_point(&mut out.lower, &candidate.lower, work)?;
        }
        if position == 0 || point_cmp(&candidate.upper, &out.upper, work)? == Ordering::Greater {
            copy_point(&mut out.upper, &candidate.upper, work)?;
        }
    }
    Ok(())
}

fn add_inputs(
    p: usize,
    a: &Interval,
    b: &Interval,
    candidate: &mut Interval,
    out: &mut Interval,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    scalar::add(
        p,
        &a.lower,
        &b.lower,
        &mut candidate.lower,
        &mut candidate.upper,
        frame,
        work,
    )?;
    copy_point(&mut out.lower, &candidate.lower, work)?;
    scalar::add(
        p,
        &a.upper,
        &b.upper,
        &mut candidate.lower,
        &mut candidate.upper,
        frame,
        work,
    )?;
    copy_point(&mut out.upper, &candidate.upper, work)
}

fn reciprocal_inputs(
    p: usize,
    input: &Interval,
    unit: &Interval,
    candidate: &mut Interval,
    out: &mut Interval,
    frame: &mut Frame,
    work: &mut Work,
) -> Result<()> {
    charge(work, (2 * scalar::M + 2) as u64)?;
    let lower_negative = input.lower.flags & 1 != 0;
    let upper_negative = input.upper.flags & 1 != 0;
    let lower_zero = input.lower.m.iter().all(|&word| word == 0);
    let upper_zero = input.upper.m.iter().all(|&word| word == 0);
    if lower_zero || upper_zero || lower_negative != upper_negative {
        return Err(Error::ZeroDivisorInterval);
    }
    scalar::divide(
        p,
        &unit.lower,
        &input.upper,
        &mut candidate.lower,
        &mut candidate.upper,
        frame,
        work,
    )?;
    copy_point(&mut out.lower, &candidate.lower, work)?;
    scalar::divide(
        p,
        &unit.upper,
        &input.lower,
        &mut candidate.lower,
        &mut candidate.upper,
        frame,
        work,
    )?;
    copy_point(&mut out.upper, &candidate.upper, work)
}

#[path = "kernel.rs"]
pub(in crate::bonds::enclosure) mod kernel;

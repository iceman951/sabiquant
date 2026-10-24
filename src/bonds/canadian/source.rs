//! Canonical bounded source ratios and exact two-edge civil-date geometry.

use super::{CanadianError, CanadianFrequency, CanadianPeriodKind, capacity, wide};
use crate::amounts::DecimalAmount;
use crate::time::Date;

/// A canonical exact source amount per100 with two compact five-word magnitudes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Source {
    pub(super) numerator: [u64; 5],
    pub(super) denominator: [u64; 5],
}

impl Source {
    /// Canonical zero, independent of the separately retained civil geometry.
    pub(super) const ZERO: Self = Self {
        numerator: [0; 5],
        denominator: [1, 0, 0, 0, 0],
    };

    /// Derives one reviewed source form and canonicalizes it once at preparation.
    pub(super) fn coupon(
        coupon: DecimalAmount,
        frequency: CanadianFrequency,
        geometry: Geometry,
        payable: bool,
        index: Option<usize>,
        phase: &'static str,
    ) -> Result<Self, CanadianError> {
        if coupon.coefficient() == 0 || geometry.start == geometry.end {
            return Ok(Self::ZERO);
        }
        let f = frequency.payments_per_year();
        let (length_num, length_den) = if payable {
            let mut n = u64::from(geometry.full_count) * 365;
            for fragment in [geometry.first, geometry.second].into_iter().flatten() {
                let days = fragment.days;
                n += if f * days < 365 {
                    u64::from(f * days)
                } else {
                    u64::from(365 - f * (fragment.period_days - days))
                };
            }
            (n, f * 365)
        } else {
            let length = geometry.coordinate();
            (length.numerator, f * length.denominator)
        };
        let mut numerator = [0; 5];
        let mut denominator = [0; 5];
        let coefficient = coupon.coefficient().unsigned_abs();
        if !wide::set_u128(&mut numerator, coefficient)
            || !wide::mul_small_assign(&mut numerator, length_num)
            || !wide::pow10_into(u16::from(coupon.scale()), &mut denominator)
            || !wide::mul_small_assign(&mut denominator, u64::from(length_den))
        {
            return Err(capacity(index, phase, "derive source"));
        }
        Self::canonical(numerator, denominator, index, phase)
    }

    /// Canonical source Euclid uses at most297 remainders for reviewed160/148bits.
    fn canonical(
        numerator: [u64; 5],
        denominator: [u64; 5],
        index: Option<usize>,
        phase: &'static str,
    ) -> Result<Self, CanadianError> {
        let error = || capacity(index, phase, "derive source");
        if wide::is_zero(&denominator) {
            return Err(error());
        }
        if wide::is_zero(&numerator) {
            return Ok(Self::ZERO);
        }
        if wide::bits(&numerator) > 160 || wide::bits(&denominator) > 148 {
            return Err(error());
        }
        let mut a = numerator;
        let mut b = denominator;
        let mut quotient = [0; 5];
        let mut remainder = [0; 5];
        let mut aligned = [0; 5];
        for step in 0..=297 {
            if wide::is_zero(&b) {
                if !wide::div_rem_into(&numerator, &a, &mut quotient, &mut remainder, &mut aligned)
                    || !wide::is_zero(&remainder)
                {
                    return Err(error());
                }
                let reduced_numerator = quotient;
                if !wide::div_rem_into(
                    &denominator,
                    &a,
                    &mut quotient,
                    &mut remainder,
                    &mut aligned,
                ) || !wide::is_zero(&remainder)
                {
                    return Err(error());
                }
                return Ok(Self {
                    numerator: reduced_numerator,
                    denominator: quotient,
                });
            }
            if step == 297 {
                break;
            }
            if !wide::div_rem_into(&a, &b, &mut quotient, &mut remainder, &mut aligned) {
                return Err(error());
            }
            a = b;
            b = remainder;
        }
        Err(error())
    }

    /// Adds the sole terminal par100 before source-PV observation.
    pub(super) fn with_par(self, index: usize) -> Result<Self, CanadianError> {
        let mut par = self.denominator;
        if !wide::mul_small_assign(&mut par, 100) || !wide::add_assign(&mut par, &self.numerator) {
            return Err(capacity(
                Some(index),
                "source present value",
                "derive source",
            ));
        }
        Self::canonical(par, self.denominator, Some(index), "source present value")
    }

    /// Direct nearest/even IEEE source observation; no decimal view intervenes.
    pub(super) fn observe(
        self,
        index: Option<usize>,
        phase: &'static str,
    ) -> Result<f64, CanadianError> {
        let mut numerator = [0; 5];
        let mut denominator = [0; 5];
        let mut aligned = [0; 5];
        wide::round_to_f64(
            &self.numerator,
            &self.denominator,
            0,
            &mut numerator,
            &mut denominator,
            &mut aligned,
        )
        .map(f64::from_bits)
        .ok_or_else(|| capacity(index, phase, "round rational"))
    }
}

/// Reduced exact quasi-coordinate length, not an observed floating horizon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Coordinate {
    pub(super) numerator: u64,
    pub(super) denominator: u32,
}

/// One nonzero proper intersection with its actual containing Q cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Fragment {
    pub(super) quasi_start: Date,
    pub(super) quasi_end: Date,
    pub(super) days: u32,
    pub(super) period_days: u32,
}

/// Full civil provenance; endpoint pairs exist even without proper fragments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Geometry {
    pub(super) start: Date,
    pub(super) end: Date,
    pub(super) first_quasi: (Date, Date),
    pub(super) last_quasi: (Date, Date),
    pub(super) full_count: u32,
    pub(super) first: Option<Fragment>,
    pub(super) second: Option<Fragment>,
}

impl Geometry {
    /// Constructs from already validated interval/cell indices in the owned cycle.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "validated Q length is at most20000"
    )]
    pub(super) fn cells(quasi: &[Date], start: Date, end: Date, left: usize, right: usize) -> Self {
        let first_quasi = (quasi[left], quasi[left + 1]);
        let last_quasi = (quasi[right], quasi[right + 1]);
        let mut result = Self {
            start,
            end,
            first_quasi,
            last_quasi,
            full_count: 0,
            first: None,
            second: None,
        };
        if start == end {
            return result;
        }
        let fragment = |a: Date, b: Date, pair: (Date, Date)| Fragment {
            quasi_start: pair.0,
            quasi_end: pair.1,
            days: b.days_since(a).unsigned_abs(),
            period_days: pair.1.days_since(pair.0).unsigned_abs(),
        };
        if left == right {
            if start == first_quasi.0 && end == first_quasi.1 {
                result.full_count = 1;
            } else {
                result.first = Some(fragment(start, end, first_quasi));
            }
            return result;
        }
        result.full_count = (right - left + 1) as u32;
        if start != first_quasi.0 {
            result.full_count -= 1;
            result.first = Some(fragment(start, first_quasi.1, first_quasi));
        }
        if end != last_quasi.1 {
            result.full_count -= 1;
            let last = fragment(last_quasi.0, end, last_quasi);
            if result.first.is_some() {
                result.second = Some(last);
            } else {
                result.first = Some(last);
            }
        }
        result
    }

    /// Queries at most two containing cells without source arithmetic or allocation.
    pub(super) fn new(quasi: &[Date], start: Date, end: Date) -> Self {
        let left = quasi.partition_point(|date| *date <= start) - 1;
        let right = if start == end {
            left
        } else {
            quasi.partition_point(|date| *date < end) - 1
        };
        Self::cells(quasi, start, end, left, right)
    }

    /// Exact two-edge quasi length, reduced without repeated floating addition.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "proper cell denominators are at most184; product<=33856"
    )]
    pub(super) fn coordinate(self) -> Coordinate {
        let a = self.first.map_or((0, 1), |x| (x.days, x.period_days));
        let b = self.second.map_or((0, 1), |x| (x.days, x.period_days));
        let denominator = u64::from(a.1) * u64::from(b.1);
        let numerator = u64::from(self.full_count) * denominator
            + u64::from(a.0) * u64::from(b.1)
            + u64::from(b.0) * u64::from(a.1);
        let mut n = numerator;
        let mut d = denominator;
        while d != 0 {
            let remainder = n % d;
            n = d;
            d = remainder;
        }
        Coordinate {
            numerator: numerator / n,
            denominator: (denominator / n) as u32,
        }
    }

    /// Complete adjacent Q periods are regular; sole both-off-Q is composite.
    pub(super) fn period_kind(self, single: bool) -> CanadianPeriodKind {
        if self.full_count == 1 && self.first.is_none() && self.second.is_none() {
            CanadianPeriodKind::Regular
        } else if single && self.start != self.first_quasi.0 && self.end != self.last_quasi.1 {
            CanadianPeriodKind::Composite
        } else if self.coordinate().numerator < u64::from(self.coordinate().denominator) {
            CanadianPeriodKind::Short
        } else {
            CanadianPeriodKind::Long
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Checked = Result<(), Box<dyn std::error::Error>>;

    fn exact(source: Source, numerator: u128, denominator: u128) {
        assert_eq!(wide::to_u128(&source.numerator), Some(numerator));
        assert_eq!(wide::to_u128(&source.denominator), Some(denominator));
    }

    fn native_gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let remainder = a % b;
            a = b;
            b = remainder;
        }
        a
    }

    /// FR-CAD-30/32/51: canonical cache identity, including long Euclid chains.
    #[test]
    fn canonical_pairs_match_native_gcd_and_large_coprime_controls() -> Checked {
        let values = [
            0,
            1,
            2,
            73,
            127,
            (1_u128 << 64) + 5,
            u128::MAX / 2,
            u128::MAX,
        ];
        for numerator in values {
            for denominator in values.into_iter().filter(|value| *value != 0) {
                let mut n = [0; 5];
                let mut d = [0; 5];
                assert!(wide::set_u128(&mut n, numerator));
                assert!(wide::set_u128(&mut d, denominator));
                let actual = Source::canonical(n, d, None, "source valuation accrued")?;
                if numerator == 0 {
                    assert_eq!(actual, Source::ZERO);
                } else {
                    let gcd = native_gcd(numerator, denominator);
                    exact(actual, numerator / gcd, denominator / gcd);
                }
            }
        }
        // (2^159+2^80)/(2^147+2^68) has the exact common factor
        // 2^68*(2^79+1), so the canonical value is4096/1.
        let mut n = [0; 5];
        let mut d = [0; 5];
        assert!(wide::add_shifted_u64(&mut n, 1, 159));
        assert!(wide::add_shifted_u64(&mut n, 1, 80));
        assert!(wide::add_shifted_u64(&mut d, 1, 147));
        assert!(wide::add_shifted_u64(&mut d, 1, 68));
        exact(
            Source::canonical(n, d, Some(7), "source payable coupon")?,
            4096,
            1,
        );

        // Consecutive Fibonacci numbers are coprime. Their148-bit-bounded
        // source form exercises substantially more Euclid steps than small ints.
        let mut a = [0; 5];
        let mut b = [0; 5];
        assert!(wide::set_u128(&mut b, 1));
        for _ in 0..210 {
            let mut next = b;
            assert!(wide::add_assign(&mut next, &a));
            a = b;
            b = next;
        }
        assert!(wide::bits(&a) > 128 && wide::bits(&a) <= 148);
        let canonical = Source::canonical(b, a, None, "source valuation accrued")?;
        assert_eq!(canonical.numerator, b);
        assert_eq!(canonical.denominator, a);
        Ok(())
    }

    /// FR-CAD-20/25/28/29: one proper fragment and empty/right-pair provenance.
    #[test]
    fn same_cell_and_empty_geometry_keep_source_policies_separate() -> Checked {
        let jan1 = Date::from_ymd(2025, 1, 1)?;
        let feb1 = Date::from_ymd(2025, 2, 1)?;
        let mar1 = Date::from_ymd(2025, 3, 1)?;
        let quasi = [jan1, feb1, mar1];
        let start = Date::from_ymd(2025, 1, 5)?;
        let end = Date::from_ymd(2025, 1, 23)?;
        let geometry = Geometry::new(&quasi, start, end);
        assert_eq!(geometry.full_count, 0);
        assert_eq!(
            geometry.first.map(|x| (x.days, x.period_days)),
            Some((18, 31))
        );
        assert_eq!(geometry.second, None);
        assert_eq!(geometry.first_quasi, (jan1, feb1));
        assert_eq!(geometry.last_quasi, (jan1, feb1));
        assert_eq!(
            geometry.coordinate(),
            Coordinate {
                numerator: 18,
                denominator: 31
            }
        );
        assert_eq!(geometry.period_kind(true), CanadianPeriodKind::Composite);
        let coupon = DecimalAmount::new(5, 0)?;
        exact(
            Source::coupon(
                coupon,
                CanadianFrequency::Monthly,
                geometry,
                false,
                Some(0),
                "source valuation coupon",
            )?,
            15,
            62,
        );
        exact(
            Source::coupon(
                coupon,
                CanadianFrequency::Monthly,
                geometry,
                true,
                Some(0),
                "source payable coupon",
            )?,
            18,
            73,
        );
        assert_eq!(
            Source::coupon(
                DecimalAmount::new(0, 0)?,
                CanadianFrequency::Monthly,
                geometry,
                true,
                Some(0),
                "source payable coupon"
            )?,
            Source::ZERO
        );
        // The geometry is independently retained even when C is zero.
        assert_eq!(geometry.first.map(|x| x.days), Some(18));
        let empty = Geometry::new(&quasi, feb1, feb1);
        assert_eq!(empty.first_quasi, (feb1, mar1));
        assert_eq!(empty.last_quasi, (feb1, mar1));
        assert_eq!(empty.full_count, 0);
        assert_eq!(empty.first, None);
        assert_eq!(empty.second, None);
        assert_eq!(
            empty.coordinate(),
            Coordinate {
                numerator: 0,
                denominator: 1
            }
        );
        let full = Geometry::new(&quasi, jan1, mar1);
        assert_eq!(full.full_count, 2);
        assert_eq!(full.first_quasi, (jan1, feb1));
        assert_eq!(full.last_quasi, (feb1, mar1));
        assert_eq!(full.first, None);
        assert_eq!(full.second, None);
        Ok(())
    }

    /// FR-CAD-26/27: completed periods never take the proper365 fragment branch.
    #[test]
    fn late184_drop_and_completed181_184_are_exact() -> Checked {
        let jul1 = Date::from_ymd(2024, 7, 1)?;
        let jan1 = Date::from_ymd(2025, 1, 1)?;
        let next_jul1 = Date::from_ymd(2025, 7, 1)?;
        let quasi = [jul1, jan1, next_jul1];
        let coupon = DecimalAmount::new(5, 0)?;
        let late182 = Geometry::new(&quasi, jul1, Date::from_ymd(2024, 12, 30)?);
        let late183 = Geometry::new(&quasi, jul1, Date::from_ymd(2024, 12, 31)?);
        exact(
            Source::coupon(
                coupon,
                CanadianFrequency::Semiannual,
                late182,
                true,
                None,
                "source settlement accrued",
            )?,
            182,
            73,
        );
        exact(
            Source::coupon(
                coupon,
                CanadianFrequency::Semiannual,
                late183,
                true,
                None,
                "source settlement accrued",
            )?,
            363,
            146,
        );
        for (start, end, days) in [(jul1, jan1, 184), (jan1, next_jul1, 181)] {
            let completed = Geometry::new(&quasi, start, end);
            assert_eq!(end.days_since(start), days);
            let amount = Source::coupon(
                coupon,
                CanadianFrequency::Semiannual,
                completed,
                true,
                Some(0),
                "source payable coupon",
            )?;
            exact(amount, 5, 2);
            assert_eq!(
                amount.observe(Some(0), "source payable coupon")?.to_bits(),
                2.5_f64.to_bits()
            );
        }
        Ok(())
    }

    /// FR-CAD-32/71: this is an internal storage-superset refusal, not a forged
    /// reachable public financial failure.
    #[test]
    fn source_capacity_refusal_keeps_original_context() {
        let mut n = [0; 5];
        assert!(wide::add_shifted_u64(&mut n, 1, 160));
        let result = Source::canonical(n, [1, 0, 0, 0, 0], Some(9), "source payable coupon");
        assert!(matches!(
            result,
            Err(CanadianError::InternalCapacity {
                index: Some(9),
                phase: "source payable coupon",
                operation: "derive source"
            })
        ));
        let zero_denominator = Source::canonical([0; 5], [0; 5], Some(9), "source payable coupon");
        assert!(matches!(
            zero_denominator,
            Err(CanadianError::InternalCapacity {
                index: Some(9),
                phase: "source payable coupon",
                operation: "derive source"
            })
        ));
        assert_eq!(
            Source::ZERO
                .observe(None, "source valuation accrued")
                .map(f64::to_bits),
            Ok(0)
        );
    }
}

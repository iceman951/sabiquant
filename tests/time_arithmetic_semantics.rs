//! Arithmetic boundaries when tenors have equal values but different units.
//!
//! Period equality compares durations within unit families. Checked period
//! arithmetic preserves matching stored units and uses the finer mixed unit,
//! so normalization can change whether an integer result is representable.
//! Date arithmetic widens both representations and admits the same dates.

use sabiquant::time::{Date, DateError, Period, PeriodError, TimeUnit};

#[test]
fn equal_periods_can_have_different_multiplication_representability() -> Result<(), PeriodError> {
    let cases = [
        (
            Period::new(2_147_483_646, TimeUnit::Days),
            Period::new(306_783_378, TimeUnit::Weeks),
            613_566_756,
        ),
        (
            Period::new(2_147_483_640, TimeUnit::Months),
            Period::new(178_956_970, TimeUnit::Years),
            357_913_940,
        ),
        (
            Period::new(-2_147_483_646, TimeUnit::Days),
            Period::new(-306_783_378, TimeUnit::Weeks),
            -613_566_756,
        ),
        (
            Period::new(-2_147_483_640, TimeUnit::Months),
            Period::new(-178_956_970, TimeUnit::Years),
            -357_913_940,
        ),
    ];
    for (fine, coarse, expected_coarse) in cases {
        assert_eq!(fine, coarse);
        assert_eq!(fine.normalized(), coarse);
        assert_eq!(fine.frequency(), coarse.frequency());
        assert!(
            matches!(fine.checked_mul(2), Err(PeriodError::Overflow { operation: "multiply", length, unit, .. })
            if length == 2 * i64::from(fine.length()) && unit == fine.unit())
        );
        let doubled = coarse.checked_mul(2)?;
        assert_eq!(
            (doubled.length(), doubled.unit()),
            (expected_coarse, coarse.unit())
        );
    }
    Ok(())
}

#[test]
fn equal_operands_can_have_different_addition_and_subtraction_representability()
-> Result<(), PeriodError> {
    let cases = [
        (
            Period::new(2_147_483_640, TimeUnit::Months),
            Period::new(178_956_970, TimeUnit::Years),
            1,
            2_147_483_652_i64,
        ),
        (
            Period::new(-2_147_483_640, TimeUnit::Months),
            Period::new(-178_956_970, TimeUnit::Years),
            -1,
            -2_147_483_652_i64,
        ),
        (
            Period::new(2_147_483_646, TimeUnit::Days),
            Period::new(306_783_378, TimeUnit::Weeks),
            1,
            2_147_483_653_i64,
        ),
        (
            Period::new(-2_147_483_646, TimeUnit::Days),
            Period::new(-306_783_378, TimeUnit::Weeks),
            -1,
            -2_147_483_653_i64,
        ),
    ];
    for (fine, coarse, delta, expected_fine) in cases {
        assert_eq!(fine, coarse);
        let addend = Period::new(delta, coarse.unit());
        let subtrahend = Period::new(-delta, coarse.unit());
        let sum = coarse.checked_add(addend)?;
        let difference = coarse.checked_sub(subtrahend)?;
        assert_eq!(
            (sum.length(), sum.unit()),
            (coarse.length() + delta, coarse.unit())
        );
        assert_eq!(
            (difference.length(), difference.unit()),
            (sum.length(), sum.unit())
        );
        for (actual, expected_operation) in [
            (fine.checked_add(addend), "add"),
            (addend.checked_add(fine), "add"),
            (fine.checked_sub(subtrahend), "subtract"),
        ] {
            assert!(
                matches!(actual, Err(PeriodError::Overflow { operation, length, unit, .. })
                if operation == expected_operation && length == expected_fine && unit == fine.unit())
            );
        }
        // Successful normalization of the result is deliberately not a
        // fallback: the chosen finer unit defines this operation's range.
        assert_eq!(fine.normalized().checked_add(addend)?, sum);
    }
    Ok(())
}

#[test]
fn every_zero_unit_preserves_nonzero_identity_and_cross_family_rejection() -> Result<(), PeriodError>
{
    for unit in TimeUnit::ALL {
        for length in [i32::MIN, -12, -7, -1, 1, 7, 12, i32::MAX] {
            let value = Period::new(length, unit);
            for zero_unit in TimeUnit::ALL {
                let zero = Period::new(0, zero_unit);
                for result in [
                    zero.checked_add(value)?,
                    value.checked_add(zero)?,
                    value.checked_sub(zero)?,
                ] {
                    assert_eq!((result.length(), result.unit()), (length, unit));
                }
                if length == i32::MIN {
                    assert!(
                        matches!(zero.checked_sub(value), Err(PeriodError::Overflow { operation: "subtract", length: 2_147_483_648, unit: result_unit, .. }) if result_unit == unit)
                    );
                } else {
                    let result = zero.checked_sub(value)?;
                    assert_eq!((result.length(), result.unit()), (-length, unit));
                }
            }
        }
    }
    // Zero accepts both families, while nonzero tenors retain their family.
    // Cancellation therefore does not make the domain of addition associative.
    let month = Period::new(1, TimeUnit::Months);
    let negative_month = Period::new(-1, TimeUnit::Months);
    let day = Period::new(1, TimeUnit::Days);
    assert_eq!(month.checked_add(negative_month)?.checked_add(day)?, day);
    assert!(matches!(
        negative_month.checked_add(day),
        Err(PeriodError::IncompatibleUnits {
            left: TimeUnit::Months,
            right: TimeUnit::Days,
            ..
        })
    ));
    Ok(())
}

fn same_date_result(left: Result<Date, DateError>, right: Result<Date, DateError>) -> bool {
    // The error context records the original unit and count; it is expected
    // to differ even when both equivalent shifts leave the supported range.
    left.map_err(|error| error.kind()) == right.map_err(|error| error.kind())
}

#[test]
fn normalized_tenors_admit_identical_dates_at_every_supported_month_boundary()
-> Result<(), DateError> {
    let pairs = [
        (
            Period::new(7, TimeUnit::Days),
            Period::new(1, TimeUnit::Weeks),
        ),
        (
            Period::new(-14, TimeUnit::Days),
            Period::new(-2, TimeUnit::Weeks),
        ),
        (
            Period::new(12, TimeUnit::Months),
            Period::new(1, TimeUnit::Years),
        ),
        (
            Period::new(-24, TimeUnit::Months),
            Period::new(-2, TimeUnit::Years),
        ),
        (
            Period::new(2_147_483_646, TimeUnit::Days),
            Period::new(306_783_378, TimeUnit::Weeks),
        ),
        (
            Period::new(2_147_483_640, TimeUnit::Months),
            Period::new(178_956_970, TimeUnit::Years),
        ),
    ];
    for year in 1..=9999 {
        for month in 1..=12 {
            let first = Date::from_ymd(year, month, 1)?;
            let last = first.last_day_of_month();
            for date in [
                first,
                first.checked_add_days(1)?,
                last.checked_sub_days(1)?,
                last,
            ] {
                for (left, right) in pairs {
                    assert_eq!(left, right);
                    assert!(
                        same_date_result(
                            date.checked_add_period(left),
                            date.checked_add_period(right)
                        ),
                        "add {left} or {right} to {date}"
                    );
                    assert!(
                        same_date_result(
                            date.checked_sub_period(left),
                            date.checked_sub_period(right)
                        ),
                        "subtract {left} or {right} from {date}"
                    );
                }
                for unit in TimeUnit::ALL {
                    let zero = Period::new(0, unit);
                    assert_eq!(date.checked_add_period(zero)?, date);
                    assert_eq!(date.checked_sub_period(zero)?, date);
                }
            }
        }
    }
    Ok(())
}

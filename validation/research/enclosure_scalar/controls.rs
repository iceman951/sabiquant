//! Private postimplementation resource/defense controls, not financial fixtures.
#![forbid(unsafe_code)]
use bounded_scalar::{self as core, Error, Frame, Point, Work, ZERO_FRAME, ZERO_POINT};

fn set(out: &mut Point, p: usize, k: i32, negative: bool, odd: bool) {
    out.m.fill(0);
    out.k.fill(0);
    out.m[p / 64 - 1] = 1 << 63;
    if odd {
        out.m[0] |= 1;
    }
    out.k[0] = u64::from(k.unsigned_abs());
    out.flags = u64::from(negative) | (u64::from(k < 0) << 1);
}
fn work() -> Work {
    Work {
        used: 0,
        max: u64::MAX,
    }
}

#[test]
fn one_third_and_negative_one_third_exact_directed_words() {
    let mut f = ZERO_FRAME;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    for p in [64, 128, 256, 512, 1024] {
        for negative in [false, true] {
            f.raw0.n.fill(0);
            f.raw0.d.fill(0);
            f.e0.fill(0);
            f.raw0.n[0] = 1;
            f.raw0.d[0] = 3;
            let mut w = work();
            core::normalize(p, negative, false, &mut lo, &mut hi, &mut f, &mut w)
                .expect("1/3 valid");
            for (point, ceiling_magnitude) in [(&lo, negative), (&hi, !negative)] {
                for i in 0..p / 64 {
                    assert_eq!(
                        point.m[i],
                        0xaaaa_aaaa_aaaa_aaaa + u64::from(i == 0 && ceiling_magnitude)
                    );
                }
                assert!(point.m[p / 64..].iter().all(|&x| x == 0));
                assert_eq!(point.k[0], 2);
                assert!(point.k[1..].iter().all(|&x| x == 0));
                assert_eq!(point.flags, u64::from(negative) | 2);
            }
        }
    }
}

#[test]
fn exact_two_multiply_divide_and_signed_cancel() {
    let mut f = ZERO_FRAME;
    let mut a = ZERO_POINT;
    let mut b = ZERO_POINT;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    for p in [64, 128, 256, 512, 1024] {
        set(&mut a, p, 1, false, false);
        set(&mut b, p, 1, false, false);
        core::multiply(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()).expect("2*2");
        assert_eq!(lo.k[0], 2);
        assert_eq!(hi.k[0], 2);
        assert_eq!(lo.m[p / 64 - 1], 1 << 63);
        assert_eq!(lo.flags, 0);
        core::divide(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()).expect("2/2");
        assert!(lo.k.iter().all(|&x| x == 0));
        assert_eq!(lo.flags, 0);
        b.flags |= 1;
        core::add(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()).expect("2-2");
        assert!(
            lo.m.iter()
                .chain(&lo.k)
                .chain(&hi.m)
                .chain(&hi.k)
                .all(|&x| x == 0)
        );
        assert_eq!((lo.flags, hi.flags), (0, 0));
    }
}

#[test]
fn finite_binary64_export_at_max_and_subnormal_boundaries() {
    let mut a = ZERO_POINT;
    set(&mut a, 64, 1023, false, false);
    a.m[0] = u64::MAX - 2047;
    assert_eq!(
        core::export(64, &a, &mut work()).expect("MAX"),
        core::Export {
            lower: Some(0x7fef_ffff_ffff_ffff),
            upper: Some(0x7fef_ffff_ffff_ffff)
        }
    );
    a.m[0] += 1;
    assert_eq!(
        core::export(64, &a, &mut work()).expect("aboveMAX"),
        core::Export {
            lower: Some(0x7fef_ffff_ffff_ffff),
            upper: None
        }
    );
    set(&mut a, 64, -1075, false, false);
    assert_eq!(
        core::export(64, &a, &mut work()).expect("tiny+"),
        core::Export {
            lower: Some(0),
            upper: Some(1)
        }
    );
    a.flags |= 1;
    assert_eq!(
        core::export(64, &a, &mut work()).expect("tiny-"),
        core::Export {
            lower: Some(0x8000_0000_0000_0001),
            upper: Some(0)
        }
    );
    set(&mut a, 64, 0, false, false);
    a.k[32] = 1 << 4;
    assert_eq!(
        core::export(64, &a, &mut work()).expect("huge+K"),
        core::Export {
            lower: Some(0x7fef_ffff_ffff_ffff),
            upper: None
        }
    );
    a.flags = 2;
    assert_eq!(
        core::export(64, &a, &mut work()).expect("huge-K"),
        core::Export {
            lower: Some(0),
            upper: Some(1)
        }
    );
}

#[test]
fn denominator_zero_precedes_zero_numerator_and_zero_is_canonical() {
    let mut f = ZERO_FRAME;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    assert_eq!(
        core::normalize(64, true, false, &mut lo, &mut hi, &mut f, &mut work()),
        Err(Error::DenominatorZero)
    );
    f.raw0.d[0] = 1;
    f.e0[32] = 1 << 4;
    core::normalize(64, true, true, &mut lo, &mut hi, &mut f, &mut work()).expect("zero");
    assert!(
        lo.m.iter()
            .chain(&lo.k)
            .chain(&hi.m)
            .chain(&hi.k)
            .all(|&x| x == 0)
    );
    assert_eq!((lo.flags, hi.flags), (0, 0));
    let a = ZERO_POINT;
    let b = ZERO_POINT;
    assert_eq!(
        core::divide(64, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
        Err(Error::DenominatorZero)
    );
}

#[test]
fn invalid_precisions_and_noncanonical_points_return_small_errors() {
    let mut f = ZERO_FRAME;
    let mut a = ZERO_POINT;
    let b = ZERO_POINT;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    for p in [0, 1, 63, 65, 96, 1025, usize::MAX] {
        assert_eq!(
            core::normalize(p, false, false, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Precision)
        );
        assert_eq!(
            core::multiply(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Precision)
        );
        assert_eq!(
            core::divide(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Precision)
        );
        assert_eq!(
            core::add(p, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Precision)
        );
        assert_eq!(core::export(p, &a, &mut work()), Err(Error::Precision));
    }
    for (m, k, flags) in [
        (0, 0, 1),
        (0, 1, 0),
        (0, 0, 2),
        (1, 0, 0),
        (1 << 63, 0, 4),
        (1 << 63, 0, 2),
    ] {
        a.m[0] = m;
        a.k[0] = k;
        a.flags = flags;
        assert_eq!(
            core::multiply(64, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Malformed)
        );
        assert_eq!(
            core::divide(64, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Malformed)
        );
        assert_eq!(
            core::add(64, &a, &b, &mut lo, &mut hi, &mut f, &mut work()),
            Err(Error::Malformed)
        );
        assert_eq!(core::export(64, &a, &mut work()), Err(Error::Malformed));
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "test adapter loans separate fixed buffers without returning arrays"
)]
fn exercise(
    op: u8,
    p: usize,
    max: u64,
    f: &mut Frame,
    a: &mut Point,
    b: &mut Point,
    lo: &mut Point,
    hi: &mut Point,
) -> (Result<(), Error>, u64) {
    set(a, p, 2, false, true);
    set(b, p, -3, true, true);
    f.raw0.n.fill(0);
    f.raw0.d.fill(0);
    f.e0.fill(0);
    f.raw0.n[0] = 1;
    f.raw0.d[0] = 3;
    let mut w = Work { used: 0, max };
    let result = match op {
        0 => core::normalize(p, false, false, lo, hi, f, &mut w),
        1 => core::multiply(p, a, b, lo, hi, f, &mut w),
        2 => core::divide(p, a, b, lo, hi, f, &mut w),
        3 => core::add(p, a, b, lo, hi, f, &mut w),
        _ => core::export(p, a, &mut w).map(|_| ()),
    };
    (result, w.used)
}

#[test]
fn work_limits_before_charge_all_precisions_all_five_operations() {
    let mut f = ZERO_FRAME;
    let mut a = ZERO_POINT;
    let mut b = ZERO_POINT;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    for p in [64, 128, 256, 512, 1024] {
        for op in 0..5 {
            let (result, used) =
                exercise(op, p, u64::MAX, &mut f, &mut a, &mut b, &mut lo, &mut hi);
            assert_eq!(result, Ok(()));
            for max in (0..=used.min(100)).chain([used / 2, used - 1, used, used + 1, u64::MAX]) {
                let (result, actual) =
                    exercise(op, p, max, &mut f, &mut a, &mut b, &mut lo, &mut hi);
                assert!(actual <= max, "p{p}op{op}max{max}actual{actual}");
                if max < used {
                    assert_eq!(result, Err(Error::WorkLimit));
                } else {
                    assert_eq!(result, Ok(()));
                    assert_eq!(actual, used);
                }
            }
        }
    }
}

#[test]
fn work_counter_overflow_does_not_wrap_or_mutate_used() {
    let mut f = ZERO_FRAME;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    let mut w = Work {
        used: u64::MAX - 1,
        max: u64::MAX,
    };
    assert_eq!(
        core::normalize(64, false, false, &mut lo, &mut hi, &mut f, &mut w),
        Err(Error::WorkLimit)
    );
    assert_eq!(w.used, u64::MAX - 1);
}

fn failure(kind: u8, max: u64) -> (Result<(), Error>, u64) {
    let mut f = ZERO_FRAME;
    let mut a = ZERO_POINT;
    let mut b = ZERO_POINT;
    let mut lo = ZERO_POINT;
    let mut hi = ZERO_POINT;
    set(&mut a, 64, 0, false, false);
    set(&mut b, 64, 0, false, false);
    f.raw0.n[0] = 2;
    f.raw0.d[0] = 1;
    match kind {
        0 => f.raw0.d[0] = 0,
        1 | 3 | 4 => a.flags = 4,
        2 => b.m.fill(0),
        5 => f.e0.fill(u64::MAX),
        _ => {
            a.k.fill(u64::MAX);
            b.k.fill(u64::MAX);
            if kind == 7 {
                b.flags = 2;
            }
        }
    }
    let mut w = Work { used: 0, max };
    let result = match kind {
        0 | 5 => core::normalize(64, false, false, &mut lo, &mut hi, &mut f, &mut w),
        1 | 6 => core::multiply(64, &a, &b, &mut lo, &mut hi, &mut f, &mut w),
        2 | 7 => core::divide(64, &a, &b, &mut lo, &mut hi, &mut f, &mut w),
        3 | 8 => core::add(64, &a, &b, &mut lo, &mut hi, &mut f, &mut w),
        _ => core::export(64, &a, &mut w).map(|_| ()),
    };
    (result, w.used)
}

#[test]
fn work_thresholds_for_private_domain_malformed_and_capacity_defenses() {
    for kind in 0..9 {
        let expected = match kind {
            0 | 2 => Error::DenominatorZero,
            1 | 3 | 4 => Error::Malformed,
            _ => Error::Capacity,
        };
        let (result, used) = failure(kind, u64::MAX);
        assert_eq!(result, Err(expected));
        for max in [0, 1, used - 1, used, used + 1, u64::MAX] {
            let (result, actual) = failure(kind, max);
            assert!(actual <= max);
            if max < used {
                assert_eq!(result, Err(Error::WorkLimit));
            } else {
                assert_eq!(result, Err(expected));
                assert_eq!(actual, used);
            }
        }
    }
}

#[test]
fn logical_type_layouts_are_actual_native_sizes_only() {
    assert_eq!(std::mem::size_of::<Point>(), 400);
    assert_eq!(std::mem::size_of::<Frame>(), 4248);
    assert_eq!(std::mem::size_of::<core::Raw>(), 560);
    assert_eq!(std::mem::align_of::<Point>(), 8);
    assert!(std::mem::size_of::<Error>() <= 8);
}

//! Independent exact-rational validation of the private production kernel.
//!
//! This excluded integration harness compiles the actual dependency-free
//! kernel source. It exposes no production API and duplicates no library tests.
//! Public coupon tests separately exercise input validation and error wrappers.

mod support;

#[path = "../src/cashflows/product/binary.rs"]
mod binary;

use support::fixture::Fixture;

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records exact arithmetic fixture and permutation counts under --nocapture"
)]
fn exact_original_three_factor_products_match_fraction_bits_and_status() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/cashflows/product-oracle.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.record_count(), 105_272);
    let mut counts = [0_usize; 4];
    for row in fixture.records() {
        let factors = [
            row.real_bits("a").unwrap(),
            row.real_bits("b").unwrap(),
            row.real_bits("c").unwrap(),
        ];
        assert!(
            factors.iter().all(|value| value.is_finite()),
            "{}",
            row.id()
        );
        let expected = row.real_bits("expected").unwrap();
        let behavior = row.text("behavior").unwrap();
        match behavior {
            "value" => {
                counts[0] += 1;
                assert!(
                    expected.is_finite() && expected.to_bits() << 1 != 0,
                    "{}",
                    row.id()
                );
            }
            "zero" => {
                counts[1] += 1;
                assert_eq!(expected.to_bits(), 0, "{}", row.id());
                assert!(
                    factors.iter().any(|value| value.to_bits() << 1 == 0),
                    "{}",
                    row.id()
                );
            }
            "underflow" => {
                counts[2] += 1;
                assert_eq!(expected.to_bits() << 1, 0, "{}", row.id());
                assert!(
                    factors.iter().all(|value| value.to_bits() << 1 != 0),
                    "{}",
                    row.id()
                );
                assert_eq!(
                    row.text("error_kind").unwrap(),
                    "value.result_out_of_domain"
                );
            }
            "overflow" => {
                counts[3] += 1;
                assert!(expected.is_infinite(), "{}", row.id());
                assert_eq!(row.text("error_kind").unwrap(), "value.result_not_finite");
            }
            other => panic!("invalid oracle behavior {other}"),
        }
        for [i, j, k] in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            let actual = binary::multiply(factors[i], factors[j], factors[k]);
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "{} permutation [{i},{j},{k}]: {factors:?}",
                row.id()
            );
        }
    }
    assert_eq!(counts, [95_393, 296, 5_202, 4_381]);
    println!(
        "exact product oracle: {} rows, {} permutation calls, {} finite nonzero / {} canonical zero / {} underflow / {} overflow; every result 0 ULP",
        fixture.record_count(),
        fixture.record_count() * 6,
        counts[0],
        counts[1],
        counts[2],
        counts[3]
    );
}

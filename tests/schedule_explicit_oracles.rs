//! Independent civil-span membership oracle for explicit coupon-date schedules.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
use sabiquant::schedule::{
    DateGenerationRule, ExplicitStubDates, PeriodRegularity, ScheduleBuilder, ScheduleCalendar,
    ScheduleError,
};
use sabiquant::time::{Date, Period};
use std::error::Error;
use support::fixture::Fixture;

#[test]
fn independent_explicit_oracle_checks_every_raw_date_flag_error_context_and_final_cap()
-> Result<(), Box<dyn Error>> {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/explicit_oracles.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 21_883);
    assert_eq!(
        fixture.header("reference"),
        Some("oracle:Python datetime/calendar")
    );
    assert!(
        fixture
            .header("reference-source")
            .unwrap()
            .contains("anchor-coordinate membership")
    );
    assert!(
        fixture
            .header("tolerance")
            .unwrap()
            .starts_with("TOL-E0 exact")
    );
    let mut values = 0;
    let mut irregular = 0;
    let mut total_boundaries = 0;
    let mut regular_flags = 0;
    let mut stub_flags = 0;
    for row in fixture.records() {
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let first = match row.text("first")? {
            "" => None,
            text => Some(text.parse::<Date>()?),
        };
        let penultimate = match row.text("penultimate")? {
            "" => None,
            text => Some(text.parse::<Date>()?),
        };
        let roles = ExplicitStubDates::new(first, penultimate)?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let rule: DateGenerationRule = row.parse("rule")?;
        let eom: EndOfMonth = row.parse("eom")?;
        let limit: u32 = row.parse("limit")?;
        let anchor: Date = row.parse("anchor")?;
        let endpoint: Date = row.parse("endpoint")?;
        let builder = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?;
        let result = builder.build_explicit(tenor, rule, roles, eom, limit);
        if row.text("status")? == "schedule.irregular_explicit_middle" {
            let error = result.unwrap_err();
            let ScheduleError::IrregularExplicitMiddle {
                rule: rejected_rule,
                tenor: rejected_tenor,
                anchor: rejected_anchor,
                endpoint: rejected_endpoint,
                regular_boundary,
                ..
            } = error
            else {
                panic!("{}: wrong error {error}", row.id());
            };
            assert_eq!(rejected_rule, rule, "{}", row.id());
            assert_eq!(
                (rejected_tenor.length(), rejected_tenor.unit()),
                (tenor.length(), tenor.unit())
            );
            assert_eq!(rejected_anchor, anchor);
            assert_eq!(rejected_endpoint, endpoint);
            assert_eq!(regular_boundary, row.parse::<Date>("last_regular")?);
            assert!(row.text("dates")?.is_empty());
            assert!(row.text("regularity")?.is_empty());
            irregular += 1;
            continue;
        }
        assert_eq!(row.text("status")?, "value");
        let dates: Vec<Date> = row
            .text("dates")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let flags: Vec<PeriodRegularity> = row
            .text("regularity")?
            .split(';')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let schedule = result.unwrap_or_else(|error| panic!("{}: {error}", row.id()));
        assert_eq!(schedule.contractual_dates(), dates, "{} raw", row.id());
        assert_eq!(schedule.adjusted_dates(), dates, "{} adjusted", row.id());
        assert_eq!(schedule.regularity(), flags, "{} flags", row.id());
        assert_eq!(schedule.explicit_stub_dates(), Some(roles));
        assert_eq!(schedule.lattice_anchor(), Some(anchor));
        assert_eq!(schedule.generation_rule(), rule);
        assert_eq!(schedule.tenor(), Some(tenor));
        assert_eq!(schedule.stub_policy(), None);
        assert_eq!(schedule.end_of_month(), Some(eom));
        assert_eq!(dates.len(), usize::try_from(limit)?);
        assert_eq!(
            schedule,
            builder.build_explicit(tenor, rule, roles, eom, limit + 10)?
        );
        assert!(
            matches!(builder.build_explicit(tenor, rule, roles, eom, limit - 1),
            Err(ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. })
                if maximum == limit - 1 && required_at_least == limit),
            "{} cap",
            row.id()
        );
        total_boundaries += dates.len();
        regular_flags += flags
            .iter()
            .filter(|flag| **flag == PeriodRegularity::Regular)
            .count();
        stub_flags += flags
            .iter()
            .filter(|flag| **flag == PeriodRegularity::Stub)
            .count();
        values += 1;
    }
    assert_eq!(
        (
            values,
            irregular,
            total_boundaries,
            regular_flags,
            stub_flags
        ),
        (17_789, 4_094, 70_819, 35_818, 17_212)
    );
    Ok(())
}

//! Exact independent civil-enumeration oracle for the automatic OldCDS rule.
//!
//! FR-SCHED-33..37: every fixture row is rebuilt through the public API and
//! compared at TOL-E0. The generator enumerated quarter projections, interior
//! twentieths and lattice membership without reading production code.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar};
use sabiquant::schedule::{
    DateGenerationRule, PeriodRegularity, Schedule, ScheduleBuilder, ScheduleCalendar,
    ScheduleError,
};
use sabiquant::time::{Date, Period};
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// The fixture's documented order-sensitive checksum: sum of (i+1)*ordinal,
/// where ordinal(0001-01-01) is one, reduced modulo the Mersenne prime 2^61-1.
fn checksum(dates: &[Date]) -> TestResult<u64> {
    const MODULUS: u128 = (1 << 61) - 1;
    let mut sum = 0_u128;
    for (index, date) in dates.iter().enumerate() {
        let ordinal = u128::try_from(i64::from(date.days_since(Date::MIN)) + 1)?;
        sum = (sum + u128::try_from(index + 1)? * ordinal) % MODULUS;
    }
    Ok(u64::try_from(sum)?)
}

fn flag_code(flag: PeriodRegularity) -> TestResult<char> {
    match flag {
        PeriodRegularity::Regular => Ok('R'),
        PeriodRegularity::Stub => Ok('S'),
        other => Err(format!("OldCDS never reports {other}").into()),
    }
}

/// Run-length text `rle:R5;S1` used by the fixture for long schedules.
fn run_length(flags: &[PeriodRegularity]) -> TestResult<String> {
    let mut runs: Vec<(char, usize)> = Vec::new();
    for &flag in flags {
        let code = flag_code(flag)?;
        match runs.last_mut() {
            Some((last, count)) if *last == code => *count += 1,
            _ => runs.push((code, 1)),
        }
    }
    let text: Vec<String> = runs
        .iter()
        .map(|(code, count)| format!("{code}{count}"))
        .collect();
    Ok(format!("rle:{}", text.join(";")))
}

fn joined_dates(dates: &[Date]) -> String {
    dates
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(";")
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let text = fixture
        .header(key)
        .ok_or_else(|| format!("missing header {key}"))?;
    let mut counts = BTreeMap::new();
    for item in text.split(';') {
        let (name, count) = item
            .split_once('=')
            .ok_or_else(|| format!("malformed {key} entry {item}"))?;
        counts.insert(name.to_owned(), count.parse()?);
    }
    Ok(counts)
}

#[derive(Default)]
struct Census {
    statuses: BTreeMap<String, usize>,
    profiles: BTreeMap<String, usize>,
    flags: BTreeMap<String, usize>,
    omitted: BTreeMap<String, usize>,
    anchors: BTreeMap<String, usize>,
    shapes: BTreeMap<String, usize>,
    checksum_rows: usize,
    maximum: usize,
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str, by: usize) {
    *map.entry(key.to_owned()).or_default() += by;
}

fn check_value(row: &Record<'_>, schedule: &Schedule, census: &mut Census) -> TestResult {
    let id = row.id();
    let start: Date = row.parse("start")?;
    let end: Date = row.parse("end")?;
    let raw = schedule.contractual_dates();
    let count: usize = row.parse("count")?;
    assert_eq!(raw.len(), count, "{id}");
    // NullCalendar and two Unadjusted conventions: adjustment is identity.
    assert_eq!(schedule.adjusted_dates(), raw, "{id}");
    let dates = row.text("dates")?;
    let flags = row.text("regularity")?;
    if let Some(expected) = dates.strip_prefix("checksum:") {
        assert!(count > 1024, "{id}");
        assert_eq!(checksum(raw)?, expected.parse::<u64>()?, "{id}");
        assert_eq!(run_length(schedule.regularity())?, flags, "{id}");
        census.checksum_rows += 1;
    } else {
        assert_eq!(joined_dates(raw), dates, "{id}");
        let actual: TestResult<Vec<String>> = schedule
            .regularity()
            .iter()
            .map(|&flag| Ok(flag_code(flag)?.to_string()))
            .collect();
        assert_eq!(actual?.join(";"), flags, "{id}");
    }
    for &flag in schedule.regularity() {
        bump(&mut census.flags, &flag_code(flag)?.to_string(), 1);
    }
    census.maximum = census.maximum.max(count);
    assert_eq!(schedule.generation_rule(), DateGenerationRule::OldCds);
    let tenor = schedule.tenor().ok_or("OldCDS retains its tenor")?;
    assert_eq!(
        (tenor.length(), tenor.unit()),
        (row.parse("length")?, row.parse("unit")?),
        "{id}"
    );
    assert_eq!(schedule.stub_policy(), None);
    assert_eq!(schedule.end_of_month(), None);
    assert_eq!(schedule.explicit_stub_dates(), None);
    assert_eq!(schedule.roll_provenance(), None);
    assert_eq!(schedule.lattice_anchor(), None);
    let provenance = schedule
        .old_cds_provenance()
        .ok_or("OldCDS retains provenance")?;
    let initial: Date = row.parse("initial_quarter")?;
    let anchor: Date = row.parse("recurrence_anchor")?;
    let resolved: Date = row.parse("resolved_end")?;
    let omitted: Option<Date> = row.parse_optional("omitted_quarter")?;
    assert_eq!(provenance.generation_start(), start, "{id}");
    assert_eq!(provenance.requested_end(), end, "{id}");
    assert_eq!(provenance.initial_quarter(), initial, "{id}");
    assert_eq!(provenance.recurrence_anchor(), anchor, "{id}");
    assert_eq!(provenance.resolved_end(), resolved, "{id}");
    assert_eq!(provenance.omitted_quarter(), omitted, "{id}");
    assert_eq!(
        (provenance.tenor().length(), provenance.tenor().unit()),
        (tenor.length(), tenor.unit())
    );
    assert_eq!(raw.first(), Some(&start), "{id}");
    assert_eq!(raw.last(), Some(&resolved), "{id}");
    bump(
        &mut census.omitted,
        if omitted.is_some() { "some" } else { "none" },
        1,
    );
    let class = if initial == resolved {
        "a-equals-r"
    } else if anchor == initial {
        "distance30plus-retains-a"
    } else if initial == start {
        "distance0-shift"
    } else {
        "distance1to29-shift-omits-a"
    };
    bump(&mut census.anchors, class, 1);
    bump(
        &mut census.shapes,
        if anchor == resolved {
            "b-equals-r"
        } else {
            "b-before-r"
        },
        1,
    );
    Ok(())
}

fn check_error(row: &Record<'_>, error: &ScheduleError) -> TestResult {
    let id = row.id();
    let start: Date = row.parse("start")?;
    let end: Date = row.parse("end")?;
    let limit: u32 = row.parse("limit")?;
    assert_eq!(error.kind(), row.text("status")?, "{id}");
    match error {
        ScheduleError::InvalidBoundaryLimit {
            requested,
            minimum,
            maximum,
            ..
        } => {
            assert_eq!(
                (*requested, *minimum, *maximum),
                (
                    limit,
                    ScheduleBuilder::MIN_BOUNDARIES,
                    ScheduleBuilder::MAX_BOUNDARIES
                ),
                "{id}"
            );
        }
        ScheduleError::InvalidOldCdsTenor {
            generation_start,
            requested_end,
            tenor,
            ..
        } => {
            assert_eq!((*generation_start, *requested_end), (start, end), "{id}");
            assert_eq!(
                (tenor.length(), tenor.unit()),
                (row.parse("length")?, row.parse("unit")?),
                "{id}"
            );
        }
        ScheduleError::OldCdsDate {
            generation_start,
            requested_end,
            tenor,
            role,
            input,
            cause,
            ..
        } => {
            assert_eq!((*generation_start, *requested_end), (start, end), "{id}");
            assert_eq!(
                (tenor.length(), tenor.unit()),
                (row.parse("length")?, row.parse("unit")?),
                "{id}"
            );
            assert_eq!(*role, row.text("error_role")?, "{id}");
            assert_eq!(*input, row.parse::<Date>("error_input")?, "{id}");
            assert_eq!(cause.kind(), "date.out_of_range", "{id}");
            let source = error.source().ok_or("OldCdsDate exposes its cause")?;
            assert_eq!(source.to_string(), cause.to_string());
        }
        ScheduleError::BoundaryLimitExceeded {
            maximum,
            required_at_least,
            ..
        } => {
            assert_eq!(*maximum, limit, "{id}");
            assert_eq!(*required_at_least, row.parse::<u32>("required")?, "{id}");
        }
        other => return Err(format!("{id}: unexpected error {other:?}").into()),
    }
    Ok(())
}

/// FR-SCHED-33..37: all 162,644 independent civil-enumeration rows, including
/// every start day of a Gregorian cycle, distance/terminal controls, the
/// stored-tenor domain, civil extremes, the 119,987-boundary maximum and
/// exact caps, agree exactly in dates, flags, provenance and typed outcomes.
#[test]
fn independent_old_cds_oracle_matches_every_row_exactly() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/old-cds-oracle.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 162_644);
    assert_eq!(
        fixture.header("fixture-set"),
        Some("SCHEDULE-OLDCDS-CIVIL-ORACLE")
    );
    assert!(
        fixture
            .header("tolerance")
            .ok_or("tolerance")?
            .starts_with("TOL-E0 exact")
    );
    assert!(
        fixture
            .header("reference-source")
            .ok_or("reference-source")?
            .contains("no Rust or QuantLib outputs")
    );
    let mut census = Census::default();
    for row in fixture.records() {
        let builder = ScheduleBuilder::new(
            row.parse("start")?,
            row.parse("end")?,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?;
        let tenor = Period::new(row.parse("length")?, row.parse("unit")?);
        let status = row.text("status")?;
        bump(&mut census.statuses, status, 1);
        bump(&mut census.profiles, row.text("profile")?, 1);
        match builder.build_old_cds(tenor, row.parse("limit")?) {
            Ok(schedule) => {
                assert_eq!(status, "value", "{}", row.id());
                check_value(&row, &schedule, &mut census)?;
            }
            Err(error) => check_error(&row, &error)?,
        }
    }
    assert_eq!(census.statuses, header_counts(&fixture, "status-counts")?);
    assert_eq!(census.profiles, header_counts(&fixture, "profile-counts")?);
    assert_eq!(census.flags, header_counts(&fixture, "flag-counts")?);
    assert_eq!(
        census.omitted,
        header_counts(&fixture, "omitted-quarter-counts")?
    );
    assert_eq!(
        census.anchors,
        header_counts(&fixture, "anchor-class-counts")?
    );
    assert_eq!(
        census.shapes,
        header_counts(&fixture, "terminal-shape-counts")?
    );
    assert_eq!(
        census.checksum_rows,
        fixture
            .header("checksum-row-count")
            .ok_or("checksums")?
            .parse()?
    );
    assert_eq!(census.maximum, 119_987);
    assert_eq!(census.statuses["value"], 162_301);
    assert_eq!((census.flags["R"], census.flags["S"]), (2_140_006, 206_408));
    Ok(())
}

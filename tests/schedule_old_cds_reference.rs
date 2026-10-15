//! Exact classification of public reference observations for OldCDS.
//!
//! FR-SCHED-38: each attested reference row is compared with the actual local
//! operation and placed in exactly one declared class. Agreement in a class
//! is not parity; distinctions are deliberate local contract choices.

mod support;

use sabiquant::calendar::{BusinessDayConvention, Calendar, TargetCalendar};
use sabiquant::schedule::{
    PeriodRegularity, Schedule, ScheduleBuilder, ScheduleCalendar, ScheduleError,
};
use sabiquant::time::{Date, Period};
use std::collections::BTreeMap;
use std::error::Error;
use support::fixture::{Fixture, Record};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn calendar(name: &str) -> TestResult<Calendar> {
    match name {
        "Null" => Ok(Calendar::null()),
        "Weekends" => Ok(Calendar::weekends_only()),
        other => Err(format!("unclassified probe calendar {other}").into()),
    }
}

fn dates(text: &str) -> TestResult<Vec<Date>> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(';').map(|field| Ok(field.parse()?)).collect()
}

fn regularity(text: &str) -> TestResult<Vec<PeriodRegularity>> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(';').map(|field| Ok(field.parse()?)).collect()
}

fn joined<T: ToString>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(";")
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    let text = fixture.header(key).ok_or("missing census header")?;
    let mut counts = BTreeMap::new();
    for item in text.split(';') {
        let (name, count) = item.split_once('=').ok_or("malformed census entry")?;
        counts.insert(name.to_owned(), count.parse()?);
    }
    Ok(counts)
}

fn month_coordinate(date: Date) -> i64 {
    i64::from(date.year()) * 12 + i64::from(date.month())
}

/// Inputs and one actual local outcome for a reference row.
struct Local {
    start: Date,
    end: Date,
    termination: BusinessDayConvention,
    tenor: Period,
    result: Result<Schedule, ScheduleError>,
}

fn adjust(
    calendar: &ScheduleCalendar,
    date: Date,
    convention: BusinessDayConvention,
) -> TestResult<Date> {
    match calendar {
        ScheduleCalendar::Plain(value) => Ok(value.adjust(date, convention)?),
        ScheduleCalendar::Target(value) => Ok(value.adjust(date, convention)?),
        _ => Err("unclassified schedule calendar".into()),
    }
}

/// S1: the reference keeps a start that the ordinary convention moves.
fn restore_start(mut vector: Vec<Date>, start: Date) -> Option<Vec<Date>> {
    match vector.first_mut() {
        Some(first) if *first != start => *first = start,
        _ => return None,
    }
    Some(vector)
}

/// The documented class of one reference value against the local result.
fn classify(local: &Local, reference: &[Date], flags: &[PeriodRegularity]) -> TestResult<String> {
    let schedule = match &local.result {
        Err(ScheduleError::InvalidOldCdsTenor { .. }) => return Ok("domain-rejected".into()),
        Err(ScheduleError::AdjustedOrder { .. }) => return Ok("local-AdjustedOrder".into()),
        Err(other) => return Err(format!("unexpected local error {other}").into()),
        Ok(schedule) => schedule,
    };
    if reference.is_empty() {
        return Ok("UNCLASSIFIED".into());
    }
    let adjusted = schedule.adjusted_dates();
    let raw = schedule.contractual_dates();
    if reference == adjusted {
        return Ok(if flags == schedule.regularity() {
            "exact".into()
        } else {
            "flags-only".into()
        });
    }
    let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
    let end_adjusted = adjust(schedule.calendar(), local.end, local.termination)?;
    // T1: E is a non-quarter point of the B-anchored stride lattice.
    let stride = match local.tenor.unit() {
        sabiquant::time::TimeUnit::Years => 12 * i64::from(local.tenor.length()),
        _ => i64::from(local.tenor.length()),
    };
    let distance = month_coordinate(local.end) - month_coordinate(provenance.recurrence_anchor());
    let t1 = (local.end.day() == 20
        && !local.end.month().is_multiple_of(3)
        && distance >= 0
        && distance % stride == 0)
        .then(|| {
            let mut vector = adjusted.to_vec();
            if let Some(last) = vector.last_mut() {
                *last = end_adjusted;
            }
            vector
        });
    // T2: E and the last raw stride point coincide after terminal adjustment.
    let t2 = if raw.len() >= 3 {
        let stride_point = raw[raw.len() - 2];
        (adjust(schedule.calendar(), stride_point, local.termination)? == end_adjusted).then(|| {
            let mut vector = adjusted[..adjusted.len() - 2].to_vec();
            vector.push(end_adjusted);
            vector
        })
    } else {
        None
    };
    let mut matches = Vec::new();
    if t1.as_deref() == Some(reference) {
        matches.push("T1-offquarter-exact-end");
    }
    if t2.as_deref() == Some(reference) {
        matches.push("T2-adjusted-end-coincidence");
    }
    if restore_start(adjusted.to_vec(), local.start).as_deref() == Some(reference) {
        matches.push("S1-unadjusted-first");
    }
    if t2
        .and_then(|vector| restore_start(vector, local.start))
        .as_deref()
        == Some(reference)
    {
        matches.push("S1+T2");
    }
    Ok(match matches.as_slice() {
        [single] => (*single).to_owned(),
        _ => "UNCLASSIFIED".to_owned(),
    })
}

fn build(row: &Record<'_>, calendar_value: ScheduleCalendar) -> TestResult<Local> {
    let start: Date = row.parse("start")?;
    let end: Date = row.parse("end")?;
    let convention: BusinessDayConvention = row.parse("convention")?;
    let termination: BusinessDayConvention = row.parse("termination")?;
    let tenor = Period::new(row.parse("tenor_length")?, row.parse("tenor_unit")?);
    let result = ScheduleBuilder::new(start, end, calendar_value, convention, termination)?
        .build_old_cds(tenor, ScheduleBuilder::MAX_BOUNDARIES);
    Ok(Local {
        start,
        end,
        termination,
        tenor,
        result,
    })
}

/// The fixture's recorded independent local model must equal the actual operation.
fn check_recorded_local(row: &Record<'_>, local: &Local) -> TestResult {
    let id = row.id();
    match &local.result {
        Ok(schedule) => {
            assert_eq!(row.text("local_status")?, "value", "{id}");
            assert_eq!(
                joined(schedule.contractual_dates()),
                row.text("local_raw_dates")?,
                "{id}"
            );
            assert_eq!(
                joined(schedule.adjusted_dates()),
                row.text("local_dates")?,
                "{id}"
            );
            assert_eq!(
                schedule.regularity(),
                regularity(row.text("local_regularity")?)?,
                "{id}"
            );
            let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
            assert_eq!(
                provenance.initial_quarter(),
                row.parse::<Date>("initial_quarter")?
            );
            assert_eq!(
                provenance.recurrence_anchor(),
                row.parse::<Date>("recurrence_anchor")?
            );
            assert_eq!(
                provenance.resolved_end(),
                row.parse::<Date>("resolved_end")?
            );
            assert_eq!(
                provenance.omitted_quarter(),
                row.parse_optional::<Date>("omitted_quarter")?
            );
            assert_eq!(
                schedule.contractual_dates().len(),
                row.parse::<usize>("boundary_count")?
            );
        }
        Err(ScheduleError::InvalidOldCdsTenor { .. }) => {
            assert_eq!(row.text("local_status")?, "InvalidOldCdsTenor", "{id}");
        }
        Err(ScheduleError::AdjustedOrder {
            index,
            previous_contractual,
            contractual,
            previous_adjusted,
            adjusted,
            ..
        }) => {
            assert_eq!(row.text("local_status")?, "AdjustedOrder", "{id}");
            assert_eq!(row.text("local_dates")?, "", "{id}");
            // The raw definition exists; its earliest adjusted pair collides.
            let raw = dates(row.text("local_raw_dates")?)?;
            let annotation = row.text("adopted_local_contract_annotation")?;
            let model = annotation_dates(annotation, "adjusted")?;
            let violation: usize = annotation_number(annotation, "first_order_violation_index")?;
            assert_eq!(*index, violation + 1, "{id}");
            assert_eq!(
                (*previous_contractual, *contractual),
                (raw[index - 1], raw[*index]),
                "{id}"
            );
            assert_eq!(
                (*previous_adjusted, *adjusted),
                (model[index - 1], model[*index]),
                "{id}"
            );
        }
        Err(other) => return Err(format!("{id}: unexpected {other}").into()),
    }
    Ok(())
}

/// FR-SCHED-38: all 1,199 attested probes (551 reproduced, 648 extra) fall
/// into exactly the recorded class; none is UNCLASSIFIED.
#[test]
fn attested_reference_probes_are_exactly_classified() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/old-cds-probes.fixture",
    ))?;
    assert_eq!(fixture.record_count(), 1_199);
    assert_eq!(
        fixture.header("quantlib-module-manifest-sha256"),
        Some("e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d")
    );
    let mut tenor = BTreeMap::<String, usize>::new();
    let mut extra = BTreeMap::<String, usize>::new();
    for row in fixture.records() {
        let id = row.id();
        assert_eq!(row.text("rule")?, "OldCDS");
        assert_eq!(row.text("end_of_month")?, "false");
        let local = build(
            &row,
            ScheduleCalendar::Plain(calendar(row.text("calendar")?)?),
        )?;
        check_recorded_local(&row, &local)?;
        let reference = dates(row.text("reference_dates")?)?;
        let flags = regularity(row.text("reference_regularity")?)?;
        if row.text("status")? == "value" {
            assert!(!reference.is_empty(), "{id}");
        }
        let class = classify(&local, &reference, &flags)?;
        assert_ne!(class, "UNCLASSIFIED", "{id}");
        assert_eq!(class, row.text("classification")?, "{id}");
        if class == "flags-only" {
            let local_flags = local
                .result
                .as_ref()
                .map_err(|e| e.to_string())?
                .regularity();
            assert_eq!(
                flag_positions(&flags, local_flags),
                row.text("flag_differences")?,
                "{id}"
            );
        }
        let census = if id.starts_with("OLD-TENOR-") {
            &mut tenor
        } else {
            &mut extra
        };
        *census.entry(class).or_default() += 1;
    }
    assert_eq!(
        tenor,
        header_counts(&fixture, "old-tenor-classification-counts")?
    );
    assert_eq!(
        extra,
        header_counts(&fixture, "old-extra-classification-counts")?
    );
    assert_eq!(tenor.values().sum::<usize>(), 551);
    assert_eq!(
        (tenor["exact"], tenor["flags-only"]),
        (152, 299),
        "Codex's original 551 observations"
    );
    Ok(())
}

/// Differing interval indices with first/interior/last, or sole for one interval.
fn flag_positions(reference: &[PeriodRegularity], local: &[PeriodRegularity]) -> String {
    let intervals = local.len();
    reference
        .iter()
        .zip(local)
        .enumerate()
        .filter(|(_, (left, right))| left != right)
        .map(|(index, _)| {
            let position = if intervals == 1 {
                "sole"
            } else if index == 0 {
                "first"
            } else if index + 1 == intervals {
                "last"
            } else {
                "interior"
            };
            format!("{index}:{position}")
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn annotation_dates(annotation: &str, key: &str) -> TestResult<Vec<Date>> {
    let marker = format!("\"{key}\":[");
    assert_eq!(annotation.matches(&marker).count(), 1, "{key}");
    let (_, tail) = annotation.split_once(&marker).ok_or("annotation dates")?;
    let body = tail
        .split_once(']')
        .ok_or("unterminated annotation dates")?
        .0;
    body.split(',')
        .map(|field| Ok(field.trim_matches('"').parse()?))
        .collect()
}

fn annotation_number(annotation: &str, key: &str) -> TestResult<usize> {
    let marker = format!("\"{key}\":");
    let (_, tail) = annotation.split_once(&marker).ok_or("annotation number")?;
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    Ok(digits.parse()?)
}

fn json_dates(observation: &str) -> TestResult<Vec<Date>> {
    let (_, tail) = observation
        .split_once("\"dates\":[")
        .ok_or("observation dates")?;
    let body = tail.split_once(']').ok_or("unterminated dates")?.0;
    body.split(',')
        .map(|field| Ok(field.trim_matches('"').parse()?))
        .collect()
}

fn json_flags(observation: &str) -> TestResult<Vec<PeriodRegularity>> {
    let (_, tail) = observation
        .split_once("\"regularity\":[")
        .ok_or("observation flags")?;
    let body = tail.split_once(']').ok_or("unterminated flags")?.0;
    body.split(',')
        .map(|field| match field {
            "true" => Ok(PeriodRegularity::Regular),
            "false" => Ok(PeriodRegularity::Stub),
            other => Err(format!("unexpected flag {other}").into()),
        })
        .collect()
}

/// FR-SCHED-38: the 98 frozen roll-probe OldCDS rows: 56 exact, 29 flags-only,
/// 4 outside the stored-tenor domain and 9 using EOM or explicit coupon
/// roles, which this automatic operation does not accept. The frozen fixture
/// is read unchanged.
#[test]
fn frozen_roll_probe_old_cds_rows_keep_their_classification() -> TestResult {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/roll-probes.fixture",
    ))?;
    let classified = Fixture::load(Fixture::repository_path(
        "validation/fixtures/schedule/old-cds-probes.fixture",
    ))?;
    let mut counts = BTreeMap::<String, Vec<String>>::new();
    let mut positions = BTreeMap::<String, usize>::new();
    for row in fixture.records() {
        if row.text("rule")? != "OldCDS" {
            continue;
        }
        let id = row.id().to_owned();
        let observation = row.text("public_observation")?;
        let class = if row.parse::<bool>("end_of_month")?
            || !row.text("first_coupon_date")?.is_empty()
            || !row.text("penultimate_coupon_date")?.is_empty()
        {
            // The reference rejects these combinations too.
            assert_eq!(row.text("status")?, "constructor-error", "{id}");
            "outside-options".to_owned()
        } else {
            let schedule_calendar = match row.text("calendar")? {
                "TARGET" => ScheduleCalendar::Target(TargetCalendar::new()),
                name => ScheduleCalendar::Plain(calendar(name)?),
            };
            let local = build(&row, schedule_calendar)?;
            if row.text("status")? == "value" {
                let flags = json_flags(observation)?;
                let class = classify(&local, &json_dates(observation)?, &flags)?;
                if class == "flags-only" {
                    let local_flags = local
                        .result
                        .as_ref()
                        .map_err(|e| e.to_string())?
                        .regularity();
                    for item in flag_positions(&flags, local_flags).split(';') {
                        let (_, position) = item.split_once(':').ok_or("flag position")?;
                        *positions.entry(position.to_owned()).or_default() += 1;
                    }
                }
                class
            } else {
                classify(&local, &[], &[])?
            }
        };
        assert_ne!(class, "UNCLASSIFIED", "{id}");
        counts.entry(class).or_default().push(id);
    }
    let sizes: BTreeMap<&str, usize> = counts.iter().map(|(k, v)| (k.as_str(), v.len())).collect();
    assert_eq!(
        sizes,
        BTreeMap::from([
            ("domain-rejected", 4),
            ("exact", 56),
            ("flags-only", 29),
            ("outside-options", 9)
        ])
    );
    // The probe report's position census: first 9, last 13, sole 7.
    assert_eq!(
        positions,
        BTreeMap::from([("first".into(), 9), ("last".into(), 13), ("sole".into(), 7)])
    );
    for (class, key) in [
        ("flags-only", "frozen-oldcds-flags-only-ids"),
        ("domain-rejected", "frozen-oldcds-domain-rejected-ids"),
        ("outside-options", "frozen-oldcds-outside-options-ids"),
    ] {
        let mut recorded: Vec<&str> = classified.header(key).ok_or(key)?.split(';').collect();
        let mut actual: Vec<&str> = counts[class].iter().map(String::as_str).collect();
        recorded.sort_unstable();
        actual.sort_unstable();
        assert_eq!(actual, recorded, "{class}");
    }
    Ok(())
}

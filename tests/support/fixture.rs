//! Reader for reference fixtures (F-CORE-06).
//!
//! It implements schema version 1 of the fixture format in
//! TESTING_STRATEGY §7.3 (decided under OQ-13), with the standard library
//! only (ADR-0002). It enforces every rule of the format and reports each
//! violation as a typed [`FixtureError`] with its line number, so a malformed
//! fixture is never read silently. Line endings may be LF or CRLF
//! (TESTING_STRATEGY §11).
//!
//! Fields are read by column name through [`Record`]: as raw text, as any
//! [`FromStr`] type, or as a real number written as shortest round-trip
//! decimal text or as an IEEE 754 bit pattern.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// The version of the fixture format that this reader implements.
pub const SCHEMA_VERSION: &str = "1";

/// The header keys that every fixture must have (NFR-12).
pub const REQUIRED_KEYS: [&str; 11] = [
    "schema",
    "fixture-set",
    "domain",
    "description",
    "reference",
    "reference-version",
    "reference-source",
    "tolerance",
    "reproduction",
    "generated-at",
    "platform",
];

/// A parsed fixture: its header, its columns and its records.
#[derive(Clone, Debug)]
pub struct Fixture {
    origin: String,
    header: Vec<(String, String)>,
    columns: Vec<String>,
    /// The fields of every record, record after record.
    fields: Vec<String>,
    /// The line number of every record.
    record_lines: Vec<usize>,
}

impl Fixture {
    /// Parses fixture text. Errors name the origin `<inline>`.
    ///
    /// # Errors
    ///
    /// A [`FixtureError`] for the first violation of the format.
    pub fn parse(text: &str) -> Result<Self, FixtureError> {
        Self::parse_from("<inline>", text)
    }

    /// Reads and parses the fixture file at `path`.
    ///
    /// # Errors
    ///
    /// [`FixtureErrorKind::Io`] if the file cannot be read or is not UTF-8;
    /// otherwise a [`FixtureError`] for the first violation of the format.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FixtureError> {
        let path = path.as_ref();
        let origin = path.display().to_string();
        #[expect(
            clippy::disallowed_methods,
            reason = "tests may read committed fixture files (NFR-13)"
        )]
        let text = std::fs::read_to_string(path);
        match text {
            Ok(text) => Self::parse_from(&origin, &text),
            Err(error) => Err(FixtureError {
                origin,
                line: None,
                kind: FixtureErrorKind::Io {
                    kind: error.kind(),
                    message: error.to_string(),
                },
            }),
        }
    }

    /// Returns the path of a committed file, given relative to the repository
    /// root, for example `"validation/fixtures/day-count/a360.fixture"`.
    pub fn repository_path(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    fn parse_from(origin: &str, text: &str) -> Result<Self, FixtureError> {
        use FixtureErrorKind as Kind;
        let fail = |line: Option<usize>, kind: Kind| FixtureError {
            origin: origin.to_owned(),
            line,
            kind,
        };
        if text.starts_with('\u{feff}') {
            return Err(fail(Some(1), Kind::ByteOrderMark));
        }
        // `lines` splits at LF and CRLF alike, and ignores a final line ending.
        let mut lines = text
            .lines()
            .enumerate()
            .map(|(index, line)| (index + 1, line));

        // Rules 1 to 4: the header, ended by one empty line.
        let mut header: Vec<(String, String)> = Vec::new();
        let mut ended = false;
        for (number, line) in lines.by_ref() {
            if line.is_empty() {
                if header.is_empty() {
                    return Err(fail(Some(number), Kind::MissingSchema));
                }
                ended = true;
                break;
            }
            if line.chars().any(char::is_control) {
                return Err(fail(Some(number), Kind::ControlCharacter));
            }
            let entry = header_line(line);
            if number == 1 {
                match entry {
                    Some(("schema", SCHEMA_VERSION)) => {}
                    Some(("schema", version)) => {
                        return Err(fail(Some(1), Kind::UnsupportedSchema(version.to_owned())));
                    }
                    _ => return Err(fail(Some(1), Kind::MissingSchema)),
                }
            }
            let Some((key, value)) = entry else {
                return Err(fail(Some(number), Kind::MalformedHeaderLine));
            };
            if header.iter().any(|(existing, _)| existing == key) {
                return Err(fail(Some(number), Kind::DuplicateHeaderKey(key.to_owned())));
            }
            header.push((key.to_owned(), value.to_owned()));
        }
        if header.is_empty() {
            return Err(fail(Some(1), Kind::MissingSchema));
        }
        if !ended {
            return Err(fail(None, Kind::MissingSeparator));
        }
        for key in REQUIRED_KEYS {
            if !header.iter().any(|(existing, _)| existing == key) {
                return Err(fail(None, Kind::MissingHeaderKey(key)));
            }
        }

        // Rule 5: the column line.
        let Some((number, line)) = lines.next() else {
            return Err(fail(None, Kind::MissingColumns));
        };
        if line.is_empty() {
            return Err(fail(Some(number), Kind::EmptyLine));
        }
        if line.chars().any(|c| c.is_control() && c != '\t') {
            return Err(fail(Some(number), Kind::ControlCharacter));
        }
        let mut columns: Vec<String> = Vec::new();
        for name in line.split('\t') {
            if !is_column_name(name) {
                return Err(fail(
                    Some(number),
                    Kind::MalformedColumnName(name.to_owned()),
                ));
            }
            if columns.iter().any(|existing| existing == name) {
                return Err(fail(Some(number), Kind::DuplicateColumn(name.to_owned())));
            }
            columns.push(name.to_owned());
        }
        if columns[0] != "id" {
            return Err(fail(
                Some(number),
                Kind::FirstColumnNotId(columns[0].clone()),
            ));
        }

        // Rule 6: the records.
        let mut fields = Vec::new();
        let mut record_lines = Vec::new();
        let mut ids: BTreeMap<&str, usize> = BTreeMap::new();
        for (number, line) in lines {
            if line.is_empty() {
                return Err(fail(Some(number), Kind::EmptyLine));
            }
            if line.chars().any(|c| c.is_control() && c != '\t') {
                return Err(fail(Some(number), Kind::ControlCharacter));
            }
            let record: Vec<&str> = line.split('\t').collect();
            if record.len() != columns.len() {
                return Err(fail(
                    Some(number),
                    Kind::FieldCount {
                        expected: columns.len(),
                        found: record.len(),
                    },
                ));
            }
            let id = record[0];
            if id.is_empty() {
                return Err(fail(Some(number), Kind::EmptyId));
            }
            if let Some(&first_line) = ids.get(id) {
                return Err(fail(
                    Some(number),
                    Kind::DuplicateId {
                        id: id.to_owned(),
                        first_line,
                    },
                ));
            }
            ids.insert(id, number);
            fields.extend(record.iter().map(|field| (*field).to_owned()));
            record_lines.push(number);
        }
        if record_lines.is_empty() {
            return Err(fail(None, Kind::NoRecords));
        }

        Ok(Self {
            origin: origin.to_owned(),
            header,
            columns,
            fields,
            record_lines,
        })
    }

    /// Returns where the fixture came from: its path, or `<inline>`.
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// Returns the value of a header key, if present. Every key in
    /// [`REQUIRED_KEYS`] is present in a parsed fixture.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.header
            .iter()
            .find(|(existing, _)| existing == key)
            .map(|(_, value)| value.as_str())
    }

    /// Returns the column names, in order. The first is always `id`.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Returns the number of records, which is at least one.
    pub fn record_count(&self) -> usize {
        self.record_lines.len()
    }

    /// Returns the records, in file order.
    pub fn records(&self) -> impl ExactSizeIterator<Item = Record<'_>> {
        (0..self.record_lines.len()).map(move |index| Record {
            fixture: self,
            index,
        })
    }
}

/// One record of a [`Fixture`].
#[derive(Clone, Copy, Debug)]
pub struct Record<'a> {
    fixture: &'a Fixture,
    index: usize,
}

impl<'a> Record<'a> {
    /// Returns the line number of the record in its file.
    pub fn line(&self) -> usize {
        self.fixture.record_lines[self.index]
    }

    /// Returns the record identifier: the `id` field, which is never empty.
    pub fn id(&self) -> &'a str {
        &self.fixture.fields[self.index * self.fixture.columns.len()]
    }

    /// Returns the raw text of a field. An empty field means "no value".
    ///
    /// # Errors
    ///
    /// [`FixtureErrorKind::UnknownColumn`] if the fixture has no such column.
    pub fn text(&self, column: &str) -> Result<&'a str, FixtureError> {
        let fixture = self.fixture;
        let position = fixture
            .columns
            .iter()
            .position(|name| name == column)
            .ok_or_else(|| self.error(FixtureErrorKind::UnknownColumn(column.to_owned())))?;
        Ok(&fixture.fields[self.index * fixture.columns.len() + position])
    }

    /// Returns the text of a field, or `None` if it is empty.
    ///
    /// # Errors
    ///
    /// As [`text`](Self::text).
    pub fn optional_text(&self, column: &str) -> Result<Option<&'a str>, FixtureError> {
        let text = self.text(column)?;
        Ok((!text.is_empty()).then_some(text))
    }

    /// Parses a field that must have a value, with the [`FromStr`]
    /// implementation of `T`.
    ///
    /// # Errors
    ///
    /// [`FixtureErrorKind::UnknownColumn`], [`FixtureErrorKind::EmptyField`],
    /// or [`FixtureErrorKind::InvalidField`] with the parser's message.
    pub fn parse<T>(&self, column: &str) -> Result<T, FixtureError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        let text = self.required_text(column)?;
        text.parse()
            .map_err(|error: T::Err| self.invalid(column, text, error.to_string()))
    }

    /// Parses a field with the [`FromStr`] implementation of `T`, or returns
    /// `None` if it is empty.
    ///
    /// # Errors
    ///
    /// As [`parse`](Self::parse), except that an empty field is not an error.
    pub fn parse_optional<T>(&self, column: &str) -> Result<Option<T>, FixtureError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        if self.text(column)?.is_empty() {
            Ok(None)
        } else {
            self.parse(column).map(Some)
        }
    }

    /// Reads a real number written as shortest round-trip decimal text, such
    /// as `0.5`, `-0.0` or `1e-07`, or as `nan`, `inf` or `-inf` (format rule
    /// 7). Decimal text is converted with correct rounding, so it gives the
    /// exact value that the generator wrote.
    ///
    /// # Errors
    ///
    /// [`FixtureErrorKind::UnknownColumn`], [`FixtureErrorKind::EmptyField`],
    /// or [`FixtureErrorKind::InvalidField`] for any other spelling.
    pub fn real(&self, column: &str) -> Result<f64, FixtureError> {
        let text = self.required_text(column)?;
        parse_real(text).ok_or_else(|| {
            self.invalid(
                column,
                text,
                "expected decimal text, `nan`, `inf` or `-inf`".to_owned(),
            )
        })
    }

    /// Reads a real number written as its IEEE 754 bit pattern: `0x`
    /// followed by 16 hexadecimal digits (format rule 7).
    ///
    /// # Errors
    ///
    /// [`FixtureErrorKind::UnknownColumn`], [`FixtureErrorKind::EmptyField`],
    /// or [`FixtureErrorKind::InvalidField`] for any other spelling.
    pub fn real_bits(&self, column: &str) -> Result<f64, FixtureError> {
        let text = self.required_text(column)?;
        parse_bits(text).ok_or_else(|| {
            self.invalid(
                column,
                text,
                "expected `0x` followed by 16 hexadecimal digits".to_owned(),
            )
        })
    }

    fn required_text(&self, column: &str) -> Result<&'a str, FixtureError> {
        let text = self.text(column)?;
        if text.is_empty() {
            Err(self.error(FixtureErrorKind::EmptyField(column.to_owned())))
        } else {
            Ok(text)
        }
    }

    fn invalid(&self, column: &str, text: &str, reason: String) -> FixtureError {
        self.error(FixtureErrorKind::InvalidField {
            column: column.to_owned(),
            text: text.to_owned(),
            reason,
        })
    }

    fn error(&self, kind: FixtureErrorKind) -> FixtureError {
        FixtureError {
            origin: self.fixture.origin.clone(),
            line: Some(self.line()),
            kind,
        }
    }
}

/// A header line is `key: value`: the key is lowercase ASCII letters, digits
/// and hyphens, starting with a letter; the value is non-empty, without
/// surrounding whitespace (format rule 3).
fn header_line(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(": ")?;
    let key_ok = key.starts_with(|c: char| c.is_ascii_lowercase())
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let value_ok = !value.is_empty() && value.trim() == value;
    (key_ok && value_ok).then_some((key, value))
}

/// A column name is lowercase ASCII letters, digits and underscores, starting
/// with a letter (format rule 5).
fn is_column_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Parses `nan`, `inf`, `-inf`, or decimal text of the form
/// `-?digits(.digits)?([eE][+-]?digits)?`.
fn parse_real(text: &str) -> Option<f64> {
    match text {
        "nan" => Some(f64::NAN),
        "inf" => Some(f64::INFINITY),
        "-inf" => Some(f64::NEG_INFINITY),
        _ if is_decimal(text) => text.parse().ok(),
        _ => None,
    }
}

fn is_decimal(text: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let (integer, fraction) = match mantissa.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (mantissa, None),
    };
    digits(integer)
        && fraction.is_none_or(digits)
        && exponent.is_none_or(|e| digits(e.strip_prefix(['+', '-']).unwrap_or(e)))
}

/// Parses `0x` followed by exactly 16 hexadecimal digits.
fn parse_bits(text: &str) -> Option<f64> {
    let hex = text.strip_prefix("0x")?;
    if hex.len() == 16 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        u64::from_str_radix(hex, 16).ok().map(f64::from_bits)
    } else {
        None
    }
}

/// A violation of the fixture format, or a field that cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureError {
    origin: String,
    line: Option<usize>,
    kind: FixtureErrorKind,
}

impl FixtureError {
    /// Returns what went wrong.
    pub fn kind(&self) -> &FixtureErrorKind {
        &self.kind
    }

    /// Returns the line where it went wrong, when one line is to blame.
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

/// What went wrong when reading a fixture. The rule numbers refer to
/// TESTING_STRATEGY §7.3.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureErrorKind {
    /// The file could not be read, or is not valid UTF-8 (rule 1).
    Io {
        /// The kind of I/O error.
        kind: io::ErrorKind,
        /// The operating system's description.
        message: String,
    },
    /// The text starts with a byte-order mark (rule 1).
    ByteOrderMark,
    /// A line contains a control character other than a table separator
    /// (rule 1).
    ControlCharacter,
    /// The first line is not `schema: <version>` (rule 3).
    MissingSchema,
    /// The schema version is not the one this reader implements.
    UnsupportedSchema(String),
    /// A header line is not `key: value` (rule 3).
    MalformedHeaderLine,
    /// A header key appears twice (rule 3).
    DuplicateHeaderKey(String),
    /// A required header key is missing (rule 4).
    MissingHeaderKey(&'static str),
    /// The header is not followed by an empty line (rule 2).
    MissingSeparator,
    /// The empty line after the header is not followed by the column line
    /// (rule 2).
    MissingColumns,
    /// A column name is malformed (rule 5).
    MalformedColumnName(String),
    /// A column name appears twice (rule 5).
    DuplicateColumn(String),
    /// The first column is not `id` (rule 5).
    FirstColumnNotId(String),
    /// An empty line where a column line or a record was expected (rule 2).
    EmptyLine,
    /// A record has the wrong number of fields (rule 6).
    FieldCount {
        /// The number of columns.
        expected: usize,
        /// The number of fields in the record.
        found: usize,
    },
    /// A record has an empty `id` (rule 6).
    EmptyId,
    /// Two records share an `id` (rule 6).
    DuplicateId {
        /// The repeated identifier.
        id: String,
        /// The line of its first occurrence.
        first_line: usize,
    },
    /// The table has no records (rule 2).
    NoRecords,
    /// A test asked for a column that the fixture does not have.
    UnknownColumn(String),
    /// A field that must have a value is empty.
    EmptyField(String),
    /// A field cannot be read as the requested type.
    InvalidField {
        /// The column of the field.
        column: String,
        /// The text of the field.
        text: String,
        /// Why it cannot be read.
        reason: String,
    },
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FixtureErrorKind as Kind;
        match self.line {
            Some(line) => write!(f, "{}:{line}: ", self.origin)?,
            None => write!(f, "{}: ", self.origin)?,
        }
        match &self.kind {
            Kind::Io { message, .. } => write!(f, "cannot read the fixture: {message}"),
            Kind::ByteOrderMark => f.write_str("the text starts with a byte-order mark"),
            Kind::ControlCharacter => f.write_str("the line contains a control character"),
            Kind::MissingSchema => write!(f, "the first line must be `schema: {SCHEMA_VERSION}`"),
            Kind::UnsupportedSchema(version) => write!(
                f,
                "schema version {version:?} is not supported; this reader implements {SCHEMA_VERSION:?}"
            ),
            Kind::MalformedHeaderLine => f.write_str("a header line must be `key: value`"),
            Kind::DuplicateHeaderKey(key) => write!(f, "header key {key:?} appears twice"),
            Kind::MissingHeaderKey(key) => write!(f, "required header key {key:?} is missing"),
            Kind::MissingSeparator => f.write_str("the header must end with an empty line"),
            Kind::MissingColumns => f.write_str("the column line is missing"),
            Kind::MalformedColumnName(name) => write!(f, "malformed column name {name:?}"),
            Kind::DuplicateColumn(name) => write!(f, "column {name:?} appears twice"),
            Kind::FirstColumnNotId(name) => {
                write!(f, "the first column must be \"id\", not {name:?}")
            }
            Kind::EmptyLine => f.write_str("unexpected empty line"),
            Kind::FieldCount { expected, found } => {
                write!(f, "expected {expected} tab-separated fields, found {found}")
            }
            Kind::EmptyId => f.write_str("the record has an empty id"),
            Kind::DuplicateId { id, first_line } => {
                write!(f, "id {id:?} was already used on line {first_line}")
            }
            Kind::NoRecords => f.write_str("the fixture has no records"),
            Kind::UnknownColumn(column) => write!(f, "the fixture has no column {column:?}"),
            Kind::EmptyField(column) => write!(f, "field {column:?} is empty"),
            Kind::InvalidField {
                column,
                text,
                reason,
            } => write!(f, "field {column:?} = {text:?} cannot be read: {reason}"),
        }
    }
}

impl std::error::Error for FixtureError {}

//! Shared test-only pins, parsers and independent unbounded integer arithmetic.
#![allow(dead_code)]
use crate::support::fixture::{Fixture, Record};
use sabiquant::amounts::{AmountQuantum, AmountRounding, DecimalAmount};
use sabiquant::bonds::{TreasuryDailyBasis, TreasurySourcePolicy, TreasuryTieRule, UsTreasury};
use sabiquant::calendar::EndOfMonth;
use sabiquant::settlement::SettlementEvidence;
use sabiquant::time::Date;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
pub(crate) type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(crate) const SOURCE: &str = "US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1";
pub(crate) fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn sha256_block(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut schedule = [0_u32; 64];
    for (word, bytes) in schedule.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for index in 16..64 {
        let early = schedule[index - 15];
        let late = schedule[index - 2];
        let sigma0 = early.rotate_right(7) ^ early.rotate_right(18) ^ (early >> 3);
        let sigma1 = late.rotate_right(17) ^ late.rotate_right(19) ^ (late >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(sigma0)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(sigma1);
    }
    let mut work = *state;
    for (constant, word) in SHA256_K.iter().zip(schedule) {
        let [a, b, c, d, e, f, g, h] = work;
        let big1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ (!e & g);
        let first = h
            .wrapping_add(big1)
            .wrapping_add(choose)
            .wrapping_add(*constant)
            .wrapping_add(word);
        let big0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let second = big0.wrapping_add(majority);
        work = [
            first.wrapping_add(second),
            a,
            b,
            c,
            d.wrapping_add(first),
            e,
            f,
            g,
        ];
    }
    for (value, add) in state.iter_mut().zip(work) {
        *value = value.wrapping_add(add);
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> TestResult<String> {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let (blocks, remainder) = bytes.as_chunks::<64>();
    for block in blocks {
        sha256_block(&mut state, block);
    }
    let mut tail = remainder.to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    let bits = u64::try_from(bytes.len())?
        .checked_mul(8)
        .ok_or("message too long for SHA-256")?;
    tail.extend_from_slice(&bits.to_be_bytes());
    for block in tail.as_chunks::<64>().0 {
        sha256_block(&mut state, block);
    }
    Ok(state.iter().map(|word| format!("{word:08x}")).collect())
}

pub(crate) fn amount(s: &str) -> TestResult<DecimalAmount> {
    let (c, e) = s.split_once(':').ok_or("amount C:S")?;
    Ok(DecimalAmount::new(c.parse()?, e.parse()?)?)
}
pub(crate) fn cell(a: DecimalAmount) -> String {
    format!("{}:{}", a.coefficient(), a.scale())
}
pub(crate) fn quantum(s: &str) -> TestResult<AmountQuantum> {
    let (c, e) = s.split_once(':').ok_or("quantum C:S")?;
    Ok(AmountQuantum::new(c.parse()?, e.parse()?)?)
}
pub(crate) fn bits(s: &str) -> TestResult<f64> {
    if s.len() != 16
        || !s
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err("binary64 encoding".into());
    }
    Ok(f64::from_bits(u64::from_str_radix(s, 16)?))
}
pub(crate) fn date(s: &str) -> TestResult<Date> {
    Ok(s.parse()?)
}
pub(crate) fn dates(s: &str) -> TestResult<Vec<Date>> {
    if s == "empty" {
        Ok(vec![])
    } else {
        s.split(',').map(date).collect()
    }
}
pub(crate) fn evidence(s: &str) -> TestResult<SettlementEvidence> {
    let p = s.split(':').collect::<Vec<_>>();
    let a = |c: &str, e: &str| amount(&format!("{c}:{e}"));
    Ok(match p.as_slice() {
        ["d", c, e] => SettlementEvidence::exact_decimal(a(c, e)?),
        ["di", a0, a1, b0, b1] => {
            SettlementEvidence::declared_decimal_interval(a(a0, a1)?, a(b0, b1)?)?
        }
        ["b", v] => SettlementEvidence::supplied_binary64(bits(v)?)?,
        ["bi", a, b] => SettlementEvidence::declared_binary64_interval(bits(a)?, bits(b)?)?,
        _ => return Err("evidence encoding".into()),
    })
}
pub(crate) fn policy(b: &str, t: &str) -> TestResult<TreasurySourcePolicy> {
    Ok(TreasurySourcePolicy::new(
        match b {
            "ExactRatio" => TreasuryDailyBasis::ExactRatio,
            "NineDecimal" => TreasuryDailyBasis::NineDecimal,
            _ => return Err("daily basis".into()),
        },
        match t {
            "NearestTiesEven" => TreasuryTieRule::NearestTiesEven,
            "NearestTiesAwayFromZero" => TreasuryTieRule::NearestTiesAwayFromZero,
            _ => return Err("tie rule".into()),
        },
    ))
}
pub(crate) fn hex(s: &str) -> String {
    let mut r = String::from("x:");
    for b in s.bytes() {
        r.push_str(&format!("{b:02x}"))
    }
    r
}
pub(crate) fn context(s: &str) -> TestResult<BTreeMap<&str, &str>> {
    if s == "-" {
        return Ok(BTreeMap::new());
    }
    let mut m = BTreeMap::new();
    for p in s.split(',') {
        let (k, v) = p.split_once('=').ok_or("context")?;
        if m.insert(k, v).is_some() {
            return Err("duplicate context".into());
        }
    }
    Ok(m)
}
pub(crate) fn count_json(m: &BTreeMap<String, usize>) -> String {
    format!(
        "{{{}}}",
        m.iter()
            .map(|(k, v)| format!("\"{k}\":{v}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
pub(crate) fn bump(m: &mut BTreeMap<String, usize>, s: &str) {
    *m.entry(s.to_owned()).or_default() += 1;
}
#[expect(
    clippy::disallowed_methods,
    reason = "test reads frozen local evidence"
)]
pub(crate) fn pinned(path: &str, sha: &str, count: usize, cols: usize) -> TestResult<Fixture> {
    let text = std::fs::read_to_string(Fixture::repository_path(path))?;
    assert_eq!(sha256(text.as_bytes())?, sha, "fixture SHA {path}");
    let f = Fixture::parse(&text)?;
    assert_eq!(f.record_count(), count);
    assert_eq!(f.columns().len(), cols);
    let (_, body) = text.split_once("\n\n").ok_or("body")?;
    let (_, rows) = body.split_once('\n').ok_or("records")?;
    let req = include_str!("../../docs/REQUIREMENTS.md")
        .lines()
        .filter(|s| s.starts_with("| FR-UST-"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    for (key, data) in [
        ("body", body.as_bytes()),
        ("records", rows.as_bytes()),
        (
            "adopted-contract",
            include_bytes!("../../docs/M8_US_TREASURY_CONTRACT.md").as_slice(),
        ),
        (
            "cash-contract",
            include_bytes!("../../docs/M8_US_TREASURY_CASH_CONTRACT.md").as_slice(),
        ),
        ("requirements-rows", req.as_bytes()),
    ] {
        assert_eq!(
            f.header(&format!("{key}-sha256")),
            Some(sha256(data)?.as_str()),
            "{key} SHA"
        );
        assert_eq!(
            f.header(&format!("{key}-fnv64")),
            Some(format!("{:016x}", fnv64(data)).as_str()),
            "{key} FNV"
        );
    }
    assert_eq!(
        f.header("adopted-contract-sha256"),
        Some("50174f42faa68cc0424a0bf47678af5d3038290bdffec3358f4bac8f763d4d93")
    );
    assert_eq!(
        f.header("cash-contract-sha256"),
        Some("25d106a89071ce6cfa0e981e5f4a805c88c188cb0000f49dc7f69f7d2fd03585")
    );
    assert_eq!(
        f.header("requirements-rows-sha256"),
        Some("722e090018bff9bbe25a2574f778766c06068c832dee6db9fe90041e1bcafeb3")
    );
    let generator = if path.contains("source-cash") {
        include_bytes!("../../validation/generator/treasury_source_cash_oracles.py").as_slice()
    } else {
        include_bytes!("../../validation/generator/treasury_ideal_oracles.py").as_slice()
    };
    assert_eq!(
        f.header("generator-sha256"),
        Some(sha256(generator)?.as_str())
    );
    assert_eq!(
        f.header("generator-fnv64"),
        Some(format!("{:016x}", fnv64(generator)).as_str())
    );
    let names = f
        .header("input-columns")
        .ok_or("input columns")?
        .split(',')
        .collect::<Vec<_>>();
    let mut unique = BTreeSet::new();
    let mut inputs = String::new();
    let mut profiles = BTreeMap::new();
    for r in f.records() {
        let key = names
            .iter()
            .map(|n| r.text(n))
            .collect::<Result<Vec<_>, _>>()?
            .join("\t");
        assert!(
            unique.insert(key.clone()),
            "duplicate serialized input {}",
            r.id()
        );
        inputs.push_str(&key);
        inputs.push('\n');
        bump(
            &mut profiles,
            r.text(if path.contains("source-cash") {
                "profile"
            } else {
                "family"
            })?,
        );
    }
    assert_eq!(
        f.header("unique-input-count"),
        Some(unique.len().to_string().as_str())
    );
    assert_eq!(
        f.header("inputs-sha256"),
        Some(sha256(inputs.as_bytes())?.as_str())
    );
    assert_eq!(
        f.header("inputs-fnv64"),
        Some(format!("{:016x}", fnv64(inputs.as_bytes())).as_str())
    );
    assert_eq!(
        f.header("profile-counts"),
        Some(count_json(&profiles).as_str())
    );
    Ok(f)
}
/// Unbounded unsigned test integer; operations deliberately differ from production limbs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Big(Vec<u32>);
impl Ord for Big {
    fn cmp(&self, b: &Self) -> Ordering {
        self.bits()
            .cmp(&b.bits())
            .then_with(|| self.0.iter().rev().cmp(b.0.iter().rev()))
    }
}
impl PartialOrd for Big {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        Some(self.cmp(b))
    }
}
impl Big {
    fn zero() -> Self {
        Self(vec![])
    }
    fn one() -> Self {
        Self(vec![1])
    }
    fn normalize(mut self) -> Self {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
        self
    }
    pub(crate) fn parse(s: &str) -> TestResult<Self> {
        if s.is_empty() {
            return Err("empty integer".into());
        }
        let mut n = Self::zero();
        for c in s.bytes() {
            if !c.is_ascii_digit() {
                return Err("integer digit".into());
            }
            n = n.small(10).add(&Self(vec![u32::from(c - b'0')]));
        }
        Ok(n)
    }
    fn bits(&self) -> usize {
        self.0.last().map_or(0, |n| {
            (self.0.len() - 1) * 32 + usize::try_from(32 - n.leading_zeros()).unwrap_or(32)
        })
    }
    fn bit(&self, i: usize) -> bool {
        self.0.get(i / 32).is_some_and(|w| w >> (i % 32) & 1 == 1)
    }
    fn shl(&self, k: usize) -> Self {
        let mut n = Self(vec![0; (self.bits() + k) / 32 + 1]);
        for i in 0..self.bits() {
            if self.bit(i) {
                n.0[(i + k) / 32] |= 1 << ((i + k) % 32)
            }
        }
        n.normalize()
    }
    fn small(&self, k: u32) -> Self {
        let mut n = vec![];
        let mut c = 0;
        for x in &self.0 {
            let p = u64::from(*x) * u64::from(k) + c;
            n.push(u32::try_from(p & 0xffff_ffff).unwrap_or(0));
            c = p >> 32;
        }
        if c > 0 {
            n.push(u32::try_from(c).unwrap_or(0));
        }
        Self(n).normalize()
    }
    fn add(&self, b: &Self) -> Self {
        let mut n = vec![];
        let mut c = 0;
        for i in 0..self.0.len().max(b.0.len()) {
            let v =
                u64::from(*self.0.get(i).unwrap_or(&0)) + u64::from(*b.0.get(i).unwrap_or(&0)) + c;
            n.push(u32::try_from(v & 0xffff_ffff).unwrap_or(0));
            c = v >> 32;
        }
        if c > 0 {
            n.push(1)
        }
        Self(n).normalize()
    }
    fn sub(&self, b: &Self) -> Self {
        assert!(self >= b);
        let mut n = vec![];
        let mut borrow = 0_i64;
        for i in 0..self.0.len() {
            let v = i64::from(self.0[i]) - i64::from(*b.0.get(i).unwrap_or(&0)) - borrow;
            borrow = i64::from(v < 0);
            n.push(u32::try_from(v & 0xffff_ffff).unwrap_or(0));
        }
        Self(n).normalize()
    }
    fn mul(&self, b: &Self) -> Self {
        let mut n = Self::zero();
        for (i, w) in b.0.iter().enumerate() {
            n = n.add(&self.small(*w).shl(i * 32));
        }
        n
    }
    fn div(&self, b: &Self) -> (Self, Self) {
        assert!(b.bits() > 0);
        let mut q = Self(vec![0; self.bits() / 32 + 1]);
        let mut r = Self::zero();
        for i in (0..self.bits()).rev() {
            r = r.shl(1);
            if self.bit(i) {
                r = r.add(&Self::one());
            }
            if r >= *b {
                r = r.sub(b);
                q.0[i / 32] |= 1 << (i % 32);
            }
        }
        (q.normalize(), r)
    }
    fn pow10(e: usize) -> Self {
        let mut n = Self::one();
        for _ in 0..e {
            n = n.small(10)
        }
        n
    }
    fn u128(&self) -> Option<u128> {
        if self.bits() > 128 {
            return None;
        }
        let mut n = 0;
        for w in self.0.iter().rev() {
            n = (n << 32) | u128::from(*w)
        }
        Some(n)
    }
}
/// Signed rational without reduction: exact comparisons do not depend on gcd.
#[derive(Clone, Debug)]
pub(crate) struct Rat {
    negative: bool,
    n: Big,
    d: Big,
}
impl Rat {
    pub(crate) fn is_zero(&self) -> bool {
        self.n.bits() == 0
    }
    pub(crate) fn divide_pow2(&self, k: u32) -> Self {
        Self {
            negative: self.negative,
            n: self.n.clone(),
            d: self.d.shl(usize::try_from(k).unwrap_or(0)),
        }
    }
    pub(crate) fn parse(s: &str) -> TestResult<Self> {
        if let Some((n, d)) = s.split_once('/') {
            return Ok(Self {
                negative: n.starts_with('-'),
                n: Big::parse(n.trim_start_matches('-'))?,
                d: Big::parse(d)?,
            });
        }
        let (m, e) = s.split_once(['e', 'E']).unwrap_or((s, "0"));
        let e: i32 = e.parse()?;
        let negative = m.starts_with('-');
        let m = m.trim_start_matches(['-', '+']);
        let (a, b) = m.split_once('.').unwrap_or((m, ""));
        let n = Big::parse(&format!("{a}{b}"))?;
        let scale = i32::try_from(b.len())? - e;
        Ok(if scale >= 0 {
            Self {
                negative,
                n,
                d: Big::pow10(usize::try_from(scale)?),
            }
        } else {
            Self {
                negative,
                n: n.mul(&Big::pow10(usize::try_from(-scale)?)),
                d: Big::one(),
            }
        })
    }
    pub(crate) fn binary(v: f64) -> Self {
        assert!(v.is_finite());
        let b = v.to_bits();
        let e = i32::try_from((b >> 52) & 2047).unwrap_or(0);
        let m = (b & ((1 << 52) - 1)) | if e == 0 { 0 } else { 1 << 52 };
        let k = if e == 0 { -1074 } else { e - 1075 };
        let n = Big::parse(&m.to_string()).unwrap_or_else(|_| Big::zero());
        let (n, d) = if k < 0 {
            (n, Big::one().shl(usize::try_from(-k).unwrap_or(0)))
        } else {
            (n.shl(usize::try_from(k).unwrap_or(0)), Big::one())
        };
        Self {
            negative: v.is_sign_negative() && m != 0,
            n,
            d,
        }
    }
    pub(crate) fn sub(&self, b: &Self) -> Self {
        let a = self.n.mul(&b.d);
        let c = b.n.mul(&self.d);
        let (n, negative) = if self.negative != b.negative {
            (a.add(&c), self.negative)
        } else if a >= c {
            (a.sub(&c), self.negative)
        } else {
            (c.sub(&a), !self.negative)
        };
        Self {
            negative,
            n,
            d: self.d.mul(&b.d),
        }
    }
    pub(crate) fn abs_le(&self, b: &Self) -> bool {
        self.n.mul(&b.d) <= b.n.mul(&self.d)
    }
    pub(crate) fn rounded(
        &self,
        scale: u8,
        mode: AmountRounding,
    ) -> TestResult<Result<DecimalAmount, (&'static str, bool)>> {
        let (q, r) = self.n.mul(&Big::pow10(usize::from(scale))).div(&self.d);
        if mode == AmountRounding::Exact && r.bits() > 0 {
            return Ok(Err(("amount.inexact", self.negative)));
        }
        let twice = r.small(2).cmp(&self.d);
        let up = match mode {
            AmountRounding::AwayFromZero => r.bits() > 0,
            AmountRounding::TowardPositiveInfinity => !self.negative && r.bits() > 0,
            AmountRounding::TowardNegativeInfinity => self.negative && r.bits() > 0,
            AmountRounding::NearestTiesAwayFromZero => twice != Ordering::Less,
            AmountRounding::NearestTiesEven => {
                twice == Ordering::Greater || (twice == Ordering::Equal && q.bit(0))
            }
            _ => false,
        };
        let mut q = if up { q.add(&Big::one()) } else { q };
        let mut scale = scale;
        while scale > 0 {
            let (v, r) = q.div(&Big(vec![10]));
            if r.bits() != 0 {
                break;
            }
            q = v;
            scale -= 1;
        }
        let Some(u) = q.u128() else {
            return Ok(Err(("amount.out_of_range", self.negative)));
        };
        let c = if self.negative {
            if u == 1_u128 << 127 {
                i128::MIN
            } else if let Ok(c) = i128::try_from(u) {
                -c
            } else {
                return Ok(Err(("amount.out_of_range", true)));
            }
        } else if let Ok(c) = i128::try_from(u) {
            c
        } else {
            return Ok(Err(("amount.out_of_range", false)));
        };
        Ok(Ok(DecimalAmount::new(c, scale)?))
    }
    pub(crate) fn binary_round(&self) -> TestResult<f64> {
        if self.n.bits() == 0 {
            return Ok(0.);
        }
        let mut e = i32::try_from(self.n.bits())? - i32::try_from(self.d.bits())?;
        let compare = if e >= 0 {
            self.n.cmp(&self.d.shl(usize::try_from(e)?))
        } else {
            self.n.shl(usize::try_from(-e)?).cmp(&self.d)
        };
        if compare == Ordering::Less {
            e -= 1;
        }
        let k = 52 - e.max(-1022);
        let (n, d) = if k >= 0 {
            (self.n.shl(usize::try_from(k)?), self.d.clone())
        } else {
            (self.n.clone(), self.d.shl(usize::try_from(-k)?))
        };
        let (q, r) = n.div(&d);
        let cmp = r.small(2).cmp(&d);
        let up = cmp == Ordering::Greater || (cmp == Ordering::Equal && q.bit(0));
        let q = if up { q.add(&Big::one()) } else { q };
        let m = u64::try_from(q.u128().ok_or("mantissa")?)?;
        let b = if e < -1022 {
            m
        } else {
            let base = u64::try_from(e + 1023)? << 52;
            base + (m - (1 << 52))
        };
        Ok(f64::from_bits(b | if self.negative { 1 << 63 } else { 0 }))
    }
}
/// Generate ideal fixture's maturity-anchored direct cycle; no recurrence drift.
pub(crate) fn ideal_instrument(r: &Record<'_>) -> TestResult<UsTreasury> {
    let m = date(r.text("maturity")?)?;
    let count: usize = r.parse("quasi_count")?;
    let eom: EndOfMonth = r.text("eom")?.parse()?;
    let mut q = Vec::new();
    for i in 0..count {
        let months = i64::try_from((count - 1 - i) * 6)?;
        let mut v = m.checked_add_months(-months)?;
        if eom == EndOfMonth::Preserve && m.day() == Date::days_in_month(m.year(), m.month())? {
            v = Date::from_ymd(
                v.year(),
                v.month(),
                Date::days_in_month(v.year(), v.month())?,
            )?;
        }
        q.push(v);
    }
    Ok(UsTreasury::new(
        date(r.text("dated")?)?,
        date(r.text("issue")?)?,
        m,
        r.text("coupon")?.parse()?,
        date(r.text("first_coupon")?)?,
        q,
        eom,
        policy(r.text("daily_basis")?, r.text("tie_rule")?)?,
        20_000,
    )?)
}

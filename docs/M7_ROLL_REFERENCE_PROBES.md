# M7 civil-roll public reference observations

This is agent engineering evidence for the [adopted local civil contract](M7_ROLL_CONTRACT.md),
following the [seven-family research](M7_SPECIAL_RULES_RESEARCH.md). It records actual
QuantLib 1.43 public behavior and exact distinctions. It is not owner approval,
a financial specification, universal library equivalence, or completion of M7.
The generator, fixture and Rust consumer are separate from the earlier cashflow
tranche. No C++ generation implementation was read, copied or translated.

## Actual collection and attestation

The [new generator](../validation/generator/schedule_roll_probes.py) collected
**538 Schedule constructor observations**, retaining every returned vector,
regularity flag, metadata value and RuntimeError. The
[fixture](../validation/fixtures/schedule/roll-probes.fixture) is **478,179 bytes**.
A fresh complete `--check` passed on **2026-10-14**, with all 538 public observations
unchanged after adopting the current contract labels/provenance. Its original
collection timestamp remains `2026-10-13T17:48:07Z`.

Each case runs in its own isolated Python `-I` subprocess, with a **10-second**
timeout and fresh wheel/module verification. Inputs are positive spans of at
most 366 days inside 2026. Stored tenors range from −1 to 12; nonpositive controls
are named explicitly. Optional coupon roles lie within their requested span.
Outputs are bounded to 400 dates and years 2025–2028; timeout, unexpected stderr,
worker failure or changed attestation aborts before writing the fixture. Empty,
non-strict or collapsed outputs would be retained, not repaired.

The actual platform is Darwin arm64, Python 3.9.6. The runtime reports QuantLib
1.43 and matches all three attested module files from the pinned wheel. The
source baseline recorded in the fixture is commit
`6b57206e04598f092efee66e3b367efc84771995`; the wheel's exact source commit has **not**
been independently attested. Byte-attested package/runtime evidence does not
certify source-to-wheel provenance or a market convention.

| Scenario group | Observations |
|---|---:|
| Original common control | 7 |
| Before/on/after rolls, 3M/6M/12M | 189 |
| Same-month and year-end spans | 42 |
| Tenor domains, including 1Y and zero/negative controls | 56 |
| EOM options | 28 |
| First/penultimate/both/reversed coupon roles | 35 |
| All seven ordinary conventions × two terminal conventions | 98 |
| Legacy first-roll distances 0/1/29/30/31 in all four quarters | 40 |
| Fixed constructor end versus separately quoted maturity | 36 |
| TARGET calendar controls | 7 |
| **Total** | **538** |

These are bounded case observations with unique IDs, not a claim of 538 unique
input tuples or exhaustive product/calendar coverage. All seven named reference
families are included. There are **466 returned values and 72 constructor errors**.
The 36 maturity cases also retain **144 separate public `cdsMaturity` calls** with
quoted tenors 0M/3M/6M/12M; accessor calls are not counted as Schedule cases.

## Exact comparison against the adopted API

The independent Python annotation selects civil months by congruence and third
Wednesdays by enumerating days 15–21. It neither reads Rust results nor translates
the reference recurrence. Its narrow comparison covers Null/Unadjusted requests
without EOM or explicit roles. The Rust
[consumer](../tests/schedule_roll_reference.rs) checks exact dates, flags, provenance,
stored tenor, absent artificial metadata, and cap/cap−1/cap+1 behavior.

A separate 60-case comparison pins hand-enumerated raw lattices for all seven
ordinary adjustments with both terminal choices, plus four TARGET requests.
Expected adjustments use an independent bounded weekday search. Every involved
candidate neighborhood is checked against all six 2026 TARGET closures; none
intersects one, so this comparison does not infer holiday behavior from weekday
behavior. Broader market coverage and guards have separate tests.

| Eligible source requests | Same dates and flags | Flags differ only | Dates differ | Local typed rejection |
|---|---:|---:|---:|---:|
| Independent Null/Unadjusted civil annotation | 79 | 78 | 6 | 25 |
| Calendar/adjustment controls | 31 | 22 | 7 | 0 |
| **248 API-compatible requests** | **110** | **100** | **13** | **25** |

The 25 errors are **19 `schedule.invalid_roll_tenor`** and **6
`schedule.resolved_roll_range`**. They are checked through public builders. There
are 223 successful adopted schedules. All retained dates and classifications use
exact E0; no tolerance hides a missing date or differing flag.

The remaining **290 observations** are classified explicitly: **254** belong to
OldCDS/CDS/CDS2015, absent from the local closed roll enum; **36** use EOM or explicit
coupon-role arguments, absent from `build_roll`. The test verifies those exclusions
and preserves their raw observations. It does not invent corresponding local calls.
The fixture's `outside-local-options` annotation also includes the 60 separately
tested calendar cases, explaining its total of 96.

### Distinctions retained without changing the contract

- **Genuine adjacent-roll regularity.** Off-roll Wednesday starts and off-stride
  terminal twentieths can be local Stub while the reference reports Regular.
  The aggregate contains 100 flags-only distinctions. A shared vector therefore
  does not establish shared regular-period meaning.
- **Six Wednesday interior-date distinctions.** ThirdWednesday with March
  17/18/19 starts and September 17 ends, at 3M or 6M, retains September 16 as an
  interior local roll. At 3M the reference returns `[start, June 17, September 17]`,
  while the adopted vector is `[start, June 17, September 16, September 17]`.
  At 6M the reference returns `[start, September 17]`, while the adopted vector
  is `[start, September 16, September 17]`. All six IDs and exact flags are checked.
- **Seven projected-terminal distinctions.** Twentieth with requested end
  November 21 and terminal Preceding, under every ordinary convention, ends at
  November 20 in the reference and omits the December boundary. The adopted
  raw endpoint remains December 20, then adjusts to December 18. Its penultimate
  November 20 remains in the vector. The test pins both vectors and their flags;
  adjusting a retained raw endpoint does not alter the contractual generation.
- **Seven zero-tenor metadata substitutions.** A reference request of 0 Months
  for any of the seven families returns two original boundaries and reports
  rule Zero, tenor 0 Years and Regular. The local four-family API rejects the
  original nonpositive stored tenor rather than silently replacing its rule.
- **Same-month inclusive collapse.** Projecting both endpoints into one month's
  third Wednesday is rejected locally as a resolved-range error. Corresponding
  reference observations retain the constructor's degenerate-schedule error;
  no single-date result was returned or fabricated.

Exactly **five non-strict reference vectors** are retained: ThirdWednesday at
1 Day and 1 Week; ThirdWednesdayInclusive at 1 Day and 1 Week; and ThirdWednesday
with reversed first/penultimate roles. The daily/weekly vectors contain repeated
Wednesdays, and the exclusive cases can reverse their final dates. The reversed
role control returns `[February 6, September 16, March 18, November 17]`.
These observations are outside the local stored-tenor/role contract; they are not
sorted, deduplicated or presented as acceptable local schedules.

## CDS-family evidence remains reference-only

For every 2026 quarterly twentieth, the 40 paired OldCDS/TwentiethIMM controls
show that OldCDS omits the upcoming twentieth when the start is **1 or 29 days**
before it, and retains it at **30 or 31 days**. At distance zero the date vectors
agree, though reference flags can differ. The consumer checks this exact dated
observation across all four quarters. This does not establish the financial
cutoff's equality/start-role rules for other contracts or dates.

For CDS2015 on December 21, the separate quoted-maturity results are 0M: null
Date; 3M: March 20, 2027; 6M: June 20, 2027; 12M: December 20, 2027. These values
are distinct from the constructor's fixed December 31, 2026 end. No quoted tenor,
CDS maturity, settlement, accrual or premium-leg convention is inferred from a
generic Schedule constructor. Primary-source qualifications remain in the
[research](M7_SPECIAL_RULES_RESEARCH.md) and
[CDS source addendum](M7_CDS_SOURCE_ADDENDUM.md); these runtime observations do not
close those financial gaps. In particular, reference CDS TARGET outputs extending
into 2025 do not bypass the local verified-coverage guard.

## Verification and immutable pins

All **four** new Rust tests passed native macOS arm64 **debug and release**, with
zero failures/ignored tests, and focused strict Clippy passed. These are focused
results; the [parent validation report](M7_ROLL_VALIDATION.md) owns final aggregate,
coverage, documentation, package and platform evidence. Full M7, owner review,
financial approval and native Linux/Windows execution remain separate gates.

```sh
/private/tmp/sabiquant-validation-venv/bin/python -I \
  validation/generator/schedule_roll_probes.py \
  --quantlib-wheel /private/tmp/sabiquant-validation-wheels/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH cargo test --offline --locked --test schedule_roll_reference -- --nocapture
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH cargo test --offline --locked --release --test schedule_roll_reference -- --nocapture
PATH=/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin:$PATH cargo clippy --offline --locked --test schedule_roll_reference -- -D warnings
```

| Artifact | SHA-256 |
|---|---|
| New generator | `03907177449e185b565b02961a06fc33c1edc5f58e16e98ff3eed6e2b4ff27f3` |
| New fixture | `88e2bacdc1ab999c1979f19c5d2f4fa485f3492cceab040cca6cc927e9edf35a` |
| New Rust consumer | `37ded5f0ca8a21cd789cbdc3ae732cbdb0cf9b50f5983f9f7f296903e707332e` |
| Adopted contract | `1787bedf20c07fd45dd2077c2ed6eeda3641c402111440e1807eaae6d347cd68` |
| Frozen seven-family research | `5c7b5c325d7a9edb069a6c32eb33a9814b3267d1b6b79206ec73bed8ac1a8582` |
| Wheel | `85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c` |
| Three-file module manifest | `e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d` |

The original generator and 46-row fixture remain byte-identical:
`schedule_probes.py` SHA-256
`2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d`,
`probes.fixture` SHA-256
`6e4a2ebbac1a20367c931de3413adf39cce1a63b3b77f44555449bdfd49bfbd7`.
The original research and adopted contract were not edited. Local links above
refer to current workspace files; no remote publication is implied.

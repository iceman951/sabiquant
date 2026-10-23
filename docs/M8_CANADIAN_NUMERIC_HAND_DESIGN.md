# Canadian blind numerical hand expectations

Authored2026-10-23 and recorded2026-10-24 Bangkok from the complete adopted
[Canadian contract](M8_CANADIAN_CONTRACT.md), before Canadian production.
The [frozen public test body](../validation/hand/bonds_canadian_numeric_hand.rs)
has eight future Rust test functions. Its 14 arithmetic groups and199 exact
rounding certificates are [committed separately](../validation/hand/canadian_numeric_hand_derivations.json).
These are reviewed expectations; no Canadian Rust test has compiled or run.

## Independent derivation and review

The root author used the adopted specifications, existing public generic
error/solver APIs and older Treasury/Gilt hand bodies for API style. No new
Canadian oracle financial helper, expected corpus or production source was
read before freezing these hand expectations. Root may later implement
production; independence here is from the separately authored oracle routes.

The [standalone derivation](../validation/generator/canadian_numeric_hand_derivation.py)
uses exact Python integers/Fraction. IEEE rounding binary-searches the ordered
lattice of all finite nonnegative binary64 bit patterns, compares exact
adjacent midpoints and applies even parity at a tie. Each selected full risk
term first rounds to extended53, retained sums remain exact, and final getters
use the independent IEEE-lattice search. No elementary function is evaluated.

The199 records comprise157 IEEE nearest-cell and42 extended53 certificates,
including repeated value confirmations. Python's Fraction-to-float conversion
corroborates all157 IEEE records. A second producer's independently authored
significand/exponent reference also agrees with all199 records. Those are
mathematical checks, not199 Rust tests or a unique-financial-input census.

A separate reviewer froze its own ideal financial generator before reading
the hands. It read the complete791-line Rust body and192-line private
derivation, checked every certificate with its own previously frozen rounder,
and verified all78 hex-literal occurrences (64 distinct values) against the
certificates. It also checked full dated inputs, public signatures, phases,
immediate error causes and cached-callback protocols. No blocking mathematical
or API issue was found; the body remained byte-identical.

## Controls and bindings

| Control | Independent assertion |
|---|---|
| Regular C12, f2/4/12, y0 | Dirty106/103/101; all six selected risk views distinguish periodic compound time from MM actual/365. |
| Compound zero and signed MIN_SUB | Represented exponent is zero and delta=f; identity prices/risks prescribe no native libm bits. |
| One-day MM C0, y-300/-280 | Exact positive affine domain outside the compound floor; observed -280 Macaulay is below ideal1/365 but above1/730. |
| One-day MM at MAX | Raw derivatives and convexity fail independently; moment/Macaulay stay normal and Modified stays nonzero subnormal. |
| Two coupons at Jun1 midcell | Exact A3, AI1104/365, lambdas1/2 and3/2; separate compound/MM six risks and one-shot clean/all-in bridges. |
| Three typed y0 targets | Each resolves112; cached lower FunctionZero uses one callback and leaves an invalid unqueried compound MAX upper unevaluated. |
| MM observed image witness | Target3fd5375375375375 is below ideal181/546 by59/4917930793088581632; cached upper FunctionZero2/0 is asserted with no ideal-root claim. |
| Ceased coupon and true T | Future position0 maps to original ordinal1; original0 remains Ceased; last-coupon selection changes explicitly and delayed T changes only the MM clock. |
| Signed/zero external targets | Clean0, Clean-MIN_SUB and SettlementDirty0 can resolve positive before an invoked lower-domain failure; ModelDirty0 rejects at zero evaluations. |
| Admission and transport | Signed finite prices/canonical zero; distinct NaN/+inf/-inf classes; immediate root/domain/ValueError layers; exact MM floor and neighboring quotes. |

These controls complement the four independent corpora. They do not replace
the separate source/cash hands, full CAD01–72 semantic mapping, native
elementary profile, composed boundary stability or actual implementation
resource/allocation/layout checks required by the
[preproduction protocol](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md).

## Frozen identity and actual verification

| Artifact/evidence | SHA256 |
|---|---|
| Public hand body, identical to reviewed private freeze | cdaa3f65a586b0a29ae2d62ff4f06114995bec1b85e17f465c8fafadb72f30b1 |
| Exact derivation JSON, identical to reviewed private freeze | a4df1f9791d90e095a06131a09b83fd9dd970e0e825a9c61f0c2b881ac27b788 |
| Portable proof generator | afd6edea585ef7d8ace9b4a61c7a8a5ffcdfba43e4d8ff1ba9d1f93291e5291c |
| Private pre-read hand manifest | 1ee303a3a59fb0281681a43478294bcf2c11d8625815d926254d1a5fdf130d96 |
| Independent full review | 50d180418beac8f82e40d0e511e5f1a9425186e81a11d0311dfd6cafe5469fba |
| Independent review evidence | e0b75e8f53626ac0e76cb42837ff61f22c42ee6e5810316eedb9bfa5ee333a62 |
| Root crosscheck against separate numeric helper | 890b23863ea61cf9a1c71584d9f1725cf1e8e2b0cc28047df6373db8f780ab93 |
| Public hand/proof validation evidence | e69913832e3baa8ede0c16b52b91a87cf9c6a50523bc7ade79cd96401d592b48 |

The portable proof generator changes paths/CLI only and reproduces the exact
reviewed JSON bytes. From the repository root:

```sh
python3 -I validation/generator/canadian_numeric_hand_derivation.py --check
rustfmt --edition 2024 --check validation/hand/bonds_canadian_numeric_hand.rs
```

Both actual checks passed with Python3.9.6 and Rust1.99 on macOS arm64.
Twelve actual proof/tool checks also passed: fresh exclusive generation,
overwrite/source-alias refusal, changed proof/line-ending/JSON/contract
refusal, parsing, unchanged tracked production inputs and whitespace checks.
Rustfmt proves parsing/formatting only. The hand body stays under
`validation/hand/` until the complete preproduction gate permits Canadian
production. Its later integration copy must preserve expectations, compile
against the actual public API, and pass debug/release plus the broader
acceptance checks. No native/foreign runtime, financial acceptance or wider
M8 completion is established by this record.

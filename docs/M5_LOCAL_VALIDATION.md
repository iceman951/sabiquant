# Local M5 correction evidence

This batch follows saved M4 commit `9aa59c2` under the same delegated
local-only authority. It resolves concrete findings from the independent
[API review](M5_LOCAL_API_REVIEW.md),
[package review](M5_LOCAL_PACKAGE_REVIEW.md) and
[requirement audit](M5_LOCAL_VALIDATION_AUDIT.md). It does not supply owner API
approval, native primary-platform CI or permission to publish.

## Resulting public behavior

Calendar/TARGET input options are now named `EndOfMonth`,
`EndpointInclusion` and `HolidayWeekendInclusion` values. No implicit boolean
adapters or financial defaults are added. TARGET no longer implements
`Default`; its explicit constructor still chooses strict verified coverage.
All calculation rules, endpoint semantics and coverage guards are preserved.

ICMA and owned accrual contexts implement definition-based Hash. Independent
construction, cloning, equal canonical joint calendars, frequency/termination
differences and TARGET policies are exercised as actual HashMap/HashSet keys.
Quotes remain real-valued PartialEq types; their equality is documented as
input/context equality rather than financial equivalence.

The normalized equality `7D == 1W` continues to represent a calendar-duration
relation. Calendar advancement retains business Days versus calendar Weeks.
A dedicated regression over all seven adjustments and both EOM options
demonstrates the difference; documentation explains why such results cannot
be cached by normalized Period alone.

Compiled contextual examples and reference search aliases make construction
requirements discoverable. Catalog labels remain distinct from complete
financial contexts. Specification prose now records ICMA's `O(log n+k)`
query work, typed error layers and the actual primary-source provenance
limitations. Numerical range/guard and rounded-switch qualifications remain
explicit, rather than treated as universal bitwise or relative guarantees.

## Native verification

On 2026-10-12, Rust 1.99.0 on aarch64 macOS passed **292 debug tests** and
**291 release tests**, over 29 result groups with zero failures or ignored
checks. Fourteen documentation tests include eleven executable examples and
three compile-fail examples rejecting boolean options and an implicit
default calendar. The debug-only internal invariant assertion explains the
one-test difference.

Formatting, strict all-target Clippy, warning-denied public/private rustdoc,
dependency policy and whitespace checks pass. All-target Linux GNU and
Windows MSVC compile checks also pass; those checks do not execute native
tests or prove R3 results.

All **598,564** existing fixture rows remain unchanged. The full suite reads
their independent/reference evidence offline; this batch changes typed
call sites rather than regenerating financial values. M4 error distributions,
range-boundary layers and 1,245,184 date-conversion cases remain applicable
on the documented domains.

Native LLVM instrumentation in
`/private/tmp/sabiquant-m5-final-coverage` measures production source regions
with embedded unit-test modules excluded:

| Source | Covered / regions | Coverage |
|---|---:|---:|
| Calendar | 799 / 838 | 95.35% |
| TARGET | 386 / 405 | 95.31% |
| Day count | 314 / 314 | 100% |
| ICMA | 163 / 164 | 99.39% |
| Rates | 778 / 785 | 99.11% |

Uncovered regions retain the documented defensive/invariant and formatter
limitations. These are source-region metrics, not branch coverage or proof
of every input. No impossible private values were introduced to inflate them.

## Extracted package verification

The package allowlist now includes only four owned `tests/support` files and
`clippy.toml` in addition to the library/licence/readme/changelog material.
They supply embedded unit-test support and the corresponding lint policy;
external integration suites, fixtures, generators and third-party reference
tools remain excluded.

The source-frozen local archive has 26 files and 94,876 compressed bytes.
SHA-256 is
`ff8756a4a9bf5ffed647e28d5df3820f3b94531a24b5113975dcd03528d54b88`.
It captures `9aa59c2` plus dirty M5 changes, not a clean release tag. Extracted
verification passes **73 unit tests**, **14 doctests**, strict all-target
Clippy and warning-denied public/private documentation builds. Original
archive files match the source-frozen workspace, including the changelog.
The manifest remains version `0.0.0`, standard-library-only and `publish=false`.

Repository-qualified README links replace dangling packaged relative paths.
This fixes archive link resolution without claiming remote availability for
unpublished local documents. A publication decision still requires those
targets, current CI and a clean reviewed artifact to exist.

## Remaining gates

US 30/360 and AFB remain explicitly excluded pending their source decisions.
The original 1999 ISDA reference scan/QR-01 has not been inspected; the
consulted 2006 text is a TD-hosted copy with authorship/amendment qualification.
TARGET needs a maintainer appointment and continuing annual source review.
Native primary-platform CI, actual security-setting verification, traceability
review, independent owner API approval and an explicit release decision remain
open. No remote write or publication was performed. Local implementation can
continue while these release gates are pending; M5 is not declared complete.

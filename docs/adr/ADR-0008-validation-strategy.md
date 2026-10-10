# ADR-0008: Differential Validation Against QuantLib

| Field | Value |
|---|---|
| ADR identifier | ADR-0008 |
| Title | Differential validation against QuantLib |
| Status | **Proposed** |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M2 ([OQ-13](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

SabiQuant must show that it agrees with QuantLib's behavior (ADR-0003). QuantLib
must stay an external reference only (C-04), the crate must stay
standard-library-only (ADR-0002), and QuantLib must not be presumed correct.
Python wheels of QuantLib 1.43 exist on PyPI, which makes it possible to
generate references reproducibly without building C++.

## Decision proposal

1. **Two kinds of oracle.**
   - An *independent oracle*: exact rational or high-precision values, and
     literature examples, which give mathematical ground truth.
   - *QuantLib 1.43*, which gives behavioral parity evidence.

   A test may use either or both. Correctness questions are settled by the
   independent oracle and primary sources.
2. **Committed fixtures.** Reference results are committed to the repository
   as data files. `cargo test` reads them with a std-only test parser. QuantLib
   never runs at test time.
3. **External generator.** A generator outside the crate uses the QuantLib
   Python package **pinned to 1.43**, through its public interface only.
   - Proposed location (OQ-13): a top-level `validation/` directory in this
     repository, **excluded from the published crate package**.
   - Alternative: a separate repository.

   The generator records its environment (QuantLib version and commit, Python
   version, platform, generator revision, timestamp).
4. **Fixture schema.** As in [TESTING_STRATEGY §7.3](../TESTING_STRATEGY.md#73-reference-fixture-specification).
   Every record names its tolerance policy and its expected error behavior.
5. **Proposed encoding (OQ-13).** UTF-8 text: a `key: value` metadata header,
   then tab-separated records under a declared column list. Dates in ISO 8601.
   Floats as shortest round-trip decimal text.
6. **Independent scenarios.** Scenarios come from SabiQuant's specifications,
   not from QuantLib's test suite. QuantLib's test data is **not copied
   wholesale**. Literature examples that QuantLib also uses are cited from
   their original sources.
7. **Discrepancy handling.** Classes D1 to D9 and the resolution procedure in
   [TESTING_STRATEGY §7.5–§7.7](../TESTING_STRATEGY.md#75-discrepancy-classification),
   with a permanent discrepancy log.
8. **Re-baselining** only at milestone boundaries
   ([QUANTLIB_REFERENCE §9](../QUANTLIB_REFERENCE.md#9-re-baselining-policy-proposed)).
9. **Fixture provenance and attribution.** Fixture files say that their values
   were computed by running QuantLib (version and commit). Fixtures hold
   numbers only, never QuantLib code.

## Alternatives considered

| Alternative | Assessment |
|---|---|
| Call QuantLib at test time through FFI | Breaks C-04 and ADR-0002, and needs a C++ toolchain. Rejected |
| Copy expected values from QuantLib's test suite | Mixes provenance, copies QuantLib material, and inherits its gaps. Rejected as a method; original sources are cited instead |
| Generate references with a C++ program | Viable fallback if the Python wheels are unavailable. Requires recording the compiler and flags |
| JSON fixtures | Needs a hand-written JSON parser (no `serde`). The proposed text format is simpler. Remains an option under OQ-13 |
| No differential testing | Loses the main behavioral benchmark. Rejected |

## Advantages

- Tests are hermetic: no network, no QuantLib, std only.
- References are reproducible and reviewable as diffs.
- Mathematical and behavioral evidence are kept apart, so QuantLib defects can
  be detected.

## Disadvantages

- Fixture files add repository volume.
- The generator is a second codebase to maintain (in Python).
- Regenerating fixtures for a new baseline takes effort.

## Consequences

- M1 delivers a std-only fixture reader (F-CORE-06) once OQ-13 is decided.
- M2 delivers the first generator and fixtures.
- Every fixture change is reviewed like code.

## Risks

- R-16: the Python wheels for the pinned version stop being installable.
  Mitigated by recording wheel hashes and keeping the C++ fallback.
- R-05: false discrepancies against QuantLib's naive formulas. Mitigated by
  the oracle and classification D5/D8.

## Reconsideration triggers

- Fixture volume becomes unwieldy. Options are a separate fixtures repository
  or compression in a std-decodable format.
- The QuantLib Python package stops being published.
- The owner wants the generator in a separate repository.

## Related requirements and documents

C-04, C-08, NFR-12; [TESTING_STRATEGY §7](../TESTING_STRATEGY.md#7-differential-testing-against-quantlib);
[NUMERICAL_CORRECTNESS](../NUMERICAL_CORRECTNESS.md); [ADR-0003](ADR-0003-quantlib-compatibility.md).

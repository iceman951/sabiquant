# Contributing Guidelines and Governance

| Field | Value |
|---|---|
| Milestone | M0; adopted at the start of M1 |
| Document status | **Adopted** on 2026-10-10, together with the licence decision ([OQ-02](IMPLEMENTATION_READINESS.md#5-decision-register), [OQ-03](IMPLEMENTATION_READINESS.md#5-decision-register)) |
| Related | [ADR-0010](adr/ADR-0010-licensing.md), [LICENSING_AND_ATTRIBUTION](LICENSING_AND_ATTRIBUTION.md), [ADR-0009](adr/ADR-0009-api-stability.md) |

> **Current status.** SabiQuant is licensed under `MIT OR Apache-2.0`, and
> implementation of M1 is authorized. Code contributions are accepted under
> the rules below and in the root `CONTRIBUTING.md`.

This document sets practical, lightweight rules for a project that starts
with a single maintainer. The root `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` and
`SECURITY.md`, in the places GitHub looks for them, are derived from it.

---

## 1. Ways to contribute

| Contribution | Welcome now? |
|---|---|
| Comments on the M0 documents (pull request reviews, issues) | Yes |
| Research input: primary sources for conventions, holiday authorities, corrections | Yes |
| Reports of errors in specifications | Yes |
| Code | Yes, for authorized milestones (currently M1), following `CONTRIBUTING.md` |
| Calendar data | From M3, under the data policy (§5.4) |

## 2. Issue reporting

Each issue type needs specific information:

| Type | Required information |
|---|---|
| Specification error | Document and section; what is wrong; the **primary source** that shows it |
| Convention research | Convention name; authoritative source; examples with values |
| Calendar data correction (M3+) | Calendar; date(s); official source URL with its publication date |
| Bug (M1+) | Version; minimal inputs; expected and actual results; the conventions involved; platform |
| Numerical discrepancy (M1+) | Inputs; SabiQuant and reference values; reference version; magnitude in ULPs or relative terms |
| Feature request | Financial purpose; QuantLib reference if any; sources |

Security-sensitive reports **must not** be filed as public issues (§10).

## 3. Pull request expectations (from M1)

1. One focused change per pull request, linked to an issue, an OQ, or a
   milestone task.
2. Specifications are updated **before or together with** code. A change in
   behavior needs an approved specification change.
3. Include the tests the change requires (§5.2), the documentation, and a
   changelog entry. Use a **"Behavior changes"** entry for any change in
   numerical output.
4. Never add a dependency. The manifest must still declare none
   ([ADR-0002](adr/ADR-0002-standard-library-only.md)).
5. Never add `unsafe` (NFR-06).
6. Never add global state, clock access, or I/O to the library
   ([ADR-0011](adr/ADR-0011-explicit-state.md)).
7. **Provenance attestation** in the pull-request description: the change does
   not copy or translate code, comments, or test tables from QuantLib or any
   other third party, and the sources used are cited.
8. Every commit carries a [Developer Certificate of Origin](https://developercertificate.org/)
   sign-off (`Signed-off-by:` trailer, added by `git commit -s`). OQ-03
   adopted it on 2026-10-10.

## 4. Independence and provenance rules

- Contributors **may** read QuantLib to understand behavior and find edge
  cases.
- Contributors **must** write SabiQuant code from SabiQuant's specifications
  and primary sources. They must **never** paste, transliterate, or closely
  paraphrase QuantLib code, comments, or test-data tables.
- Literature examples are cited from the **original** publication, even when
  QuantLib's tests also use them.
- Licensed publications (for example the ISDA Definitions) are paraphrased,
  never reproduced.
- Details: [LICENSING_AND_ATTRIBUTION §5](LICENSING_AND_ATTRIBUTION.md#5-independence-and-provenance-policy).

## 5. Financial correctness requirements for contributions

### 5.1 Specification

- Every financial computation traces to a specification section and to cited
  sources (NFR-01).
- Conventions are named exactly. Ambiguous names are rejected (FR-CORE-06).
- Ambiguities are recorded as OQs. They are not settled silently in code.

### 5.2 Evidence

A change to financial behavior includes, where applicable:

| Evidence | Applies to |
|---|---|
| Exhaustive or windowed enumeration | Discrete domains (dates, calendars, day counts) |
| Property tests with recorded seeds | Invariants |
| Independent-oracle fixtures | Every real-valued output |
| QuantLib differential fixtures | Features QuantLib has |
| Boundary and error tests | Every error variant and edge |
| Tolerance justification | Every comparison of reals (exactness class and policy ID) |
| Discrepancy classification | Every mismatch with a reference |

### 5.3 Review checklist

- [ ] The specification exists or is updated; sources are cited.
- [ ] Requirement and scenario IDs are referenced in tests.
- [ ] Errors are typed. There are no panics on input and no NaN output.
- [ ] Numerics follow NP-01 to NP-12; tolerances are justified.
- [ ] No dependency, no `unsafe`, no global state, no I/O.
- [ ] Public items are documented, including the QuantLib correspondence.
- [ ] Changelog updated, with a "Behavior changes" entry if needed.
- [ ] The provenance attestation is present.

### 5.4 Calendar data contributions (from M3)

Every holiday entry or rule cites an official source (a government gazette, a
central bank, an exchange, or a payment-system operator), with URL and
retrieval date. The change updates the calendar's provenance and coverage
record. Corrections to past years are flagged as behavior changes.

## 6. Code review principles

- **Correctness before style.** Review the finance, the numerics, and the
  edge cases first.
- **Specifications are the contract.** If code and specification disagree,
  one of them must change, explicitly.
- **Solo-maintainer practice.** While the owner is the only reviewer:
  - every non-trivial change goes through a pull request, even the owner's;
  - the author works through the checklist (§5.3);
  - CI must pass;
  - for changes to conventions or numerics, a second reading on a later day
    (a cooling-off review) is recommended before merging.
- External reviewers with domain expertise are invited explicitly for
  convention-heavy milestones (M3, M4).

## 7. Documentation standards

- English, using the terms of [DOMAIN_MODEL §2](DOMAIN_MODEL.md#2-ubiquitous-language).
- Verified facts, proposals, and assumptions are kept apart and labelled.
- External claims cite a source (URL or publication).
- Mermaid diagrams where they help understanding.
- No reproduction of copyrighted or licensed texts.
- Documents carry a status header (milestone, status, related documents).
- Rationale is recorded for every significant decision, in an ADR when the
  decision is consequential.

## 8. Release management

As in [ADR-0009](adr/ADR-0009-api-stability.md). **Release checklist** (from
M5):

1. All tests pass on every primary platform.
2. API diff reviewed against the previous release.
3. "Behavior changes" reviewed, including calendar-data corrections.
4. Changelog complete. Version number follows SemVer.
5. MSRV verified.
6. Documentation builds with no missing items.
7. The owner approves publication.

**Releases with severe correctness defects:** fix, consider yanking the
affected versions, and publish an advisory describing the affected
computations and how to upgrade.

**Deprecation:** deprecated items name their replacement and stay for at
least one minor release before removal (ADR-0009).

## 9. Governance model

| Aspect | Proposal |
|---|---|
| Decision authority | The project owner decides scope, architecture, releases, and licensing |
| Consequential decisions | Recorded as ADRs and approved by the owner ([ADR index](adr/README.md)) |
| Lightweight RFCs | A pull request that adds a Proposed ADR. No separate RFC process |
| Open decisions | Kept in the [decision register](IMPLEMENTATION_READINESS.md#5-decision-register) |
| Adding maintainers | The owner may invite maintainers for specific areas (for example calendars or numerics), recorded in a maintainers list together with their review rights |
| Conflict resolution | Discussion in the pull request or issue. The owner decides. The rationale is recorded |
| Code of conduct | [Contributor Covenant 3.0](https://www.contributor-covenant.org/version/3/0/), adopted on 2026-10-10 (OQ-03), in the root `CODE_OF_CONDUCT.md` |

This governance is deliberately light. It can be revisited when more
maintainers join (assumption A-01).

## 10. Security reporting

- **Channel ([OQ-26](IMPLEMENTATION_READINESS.md#5-decision-register), decided
  2026-10-10):** GitHub's private vulnerability reporting for this repository,
  as described in the root `SECURITY.md`. Enabling it in the repository
  settings is an owner action.
- **In scope:** inputs that cause hangs or excessive resource use (for
  example, unbounded searches), panics reachable from the public API, and
  defects that produce silently wrong financial results in a way that could be
  exploited.
- Reports are acknowledged on a best-effort basis. There are no response-time
  commitments while the project is maintained by one person.
- Fixed issues are disclosed in the changelog and, where appropriate, in an
  advisory.

## 11. Maintenance responsibilities

| Responsibility | Frequency |
|---|---|
| Triage issues and review pull requests | Ongoing |
| Watch QuantLib releases and decide on re-baselining | At each milestone boundary ([QUANTLIB_REFERENCE §9](QUANTLIB_REFERENCE.md#9-re-baselining-policy-proposed)) |
| Annual review of calendar data against official announcements | Each year, per calendar |
| Review MSRV and toolchain | At each minor release |
| Enforce the dependency policy | Every pull request |
| Keep the decision and risk registers current | At each milestone review |

## 12. Compatibility expectations for contributors

Contributors do not claim QuantLib compatibility in code, documentation, or
pull requests without evidence that meets [ADR-0003](adr/ADR-0003-quantlib-compatibility.md).
Intentional divergences are documented in the specification.

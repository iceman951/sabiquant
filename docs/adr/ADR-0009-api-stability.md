# ADR-0009: Public API Stability and Versioning

| Field | Value |
|---|---|
| ADR identifier | ADR-0009 |
| Title | Public API stability and versioning |
| Status | **Accepted** on 2026-10-10 ([decision record](#decision-record)) |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1 (edition and MSRV, [OQ-04](../IMPLEMENTATION_READINESS.md#5-decision-register)); M5 (publication, [OQ-25](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

SabiQuant is meant to be a long-lived library with a stable API. Financial
users also depend on **numerical** stability: a release that silently changes
results can be as disruptive as one that breaks compilation. Cargo
implements semantic versioning, with special rules before 1.0
([Cargo SemVer guide](https://doc.rust-lang.org/cargo/reference/semver.html)).
The latest stable Rust on the research date was 1.99.0, and the 2024 edition
has been stable since Rust 1.85.

## Decision proposal

1. **Semantic versioning** following the Cargo SemVer guide.
2. **Pre-1.0 phase.** All releases until the foundations are validated and
   reviewed are 0.x. Breaking changes in 0.x bump the minor version and come
   with migration notes.
3. **First publication.** Nothing is published to crates.io before M5. The
   first release, 0.1.0, happens at M5 and only with the owner's explicit
   approval (OQ-25). No placeholder crate is published to reserve the name;
   crates.io's usage policy restricts name squatting.
4. **Criteria for 1.0:**
   - The foundational API has been reviewed.
   - The foundations have been in real use for a while.
   - Every foundational feature is Verified or Verified with divergences
     (ADR-0003).
   - Documentation is complete.
   - No open "Requires Decision" items remain for the foundations.
5. **Edition and MSRV (OQ-04).**
   - Rust **2024 edition**.
   - The MSRV is declared in the package manifest.
   - **Proposal:** at M1, set the MSRV to the stable release current at that
     time. Raise it only in minor releases, with a changelog entry, and never
     to a toolchain less than about six months old.
6. **Stability tools.**
   - Enumerations and structs that may grow are non-exhaustive.
   - Fields are private.
   - Traits that users are not meant to implement are sealed.
   - Error kinds are stable, but error messages are not (ADR-0007).
7. **Deprecation.**
   - Items are deprecated, naming their replacement, for at least one minor
     release before removal.
   - After 1.0, removal happens only in a major release.
8. **Numerical behavior changes are part of the contract.**
   - Every change in numerical output, including calendar-data corrections, is
     listed under **"Behavior changes"** in the changelog, with the affected
     conventions or calendars and typical magnitudes.
   - Correcting behavior to match the specification may ship in a patch
     release, clearly flagged.
   - An intended change of convention or specification is a minor release
     before 1.0 and a major release after 1.0.
9. **Changelog.** A human-written `CHANGELOG.md` (created in M1) in the "Keep
   a Changelog" style, with sections Added, Changed, Deprecated, Removed,
   Fixed, Security, and Behavior changes.
10. **Releases with severe correctness defects.**
    - Fix the defect.
    - Consider yanking the affected versions.
    - Publish an advisory that describes the affected computations and gives
      upgrade guidance.

## Alternatives considered

| Alternative | Assessment |
|---|---|
| Publish early and often (0.0.x) from M1 | Signals instability and invites use before validation. Rejected |
| Reach 1.0 quickly | Freezes the API before real usage. Rejected |
| Track the latest stable Rust only, with no declared MSRV | Hostile to users on pinned toolchains. Rejected |
| A very old MSRV | Blocks useful language features. Rejected |
| Treat numerical changes as invisible patch fixes | Unacceptable for financial users. Rejected |

## Advantages

- Users can trust both API stability and result stability.
- The pre-1.0 phase allows the API to be refined based on validation.

## Disadvantages

- Release discipline costs time: changelog entries, behavior-change notes,
  and deprecation cycles.
- Some users may want an earlier release.

## Consequences

- Every pull request that changes behavior updates the changelog (from M1).
- The release checklist covers API diff review, behavior-change review, MSRV
  check, documentation build, and the cross-platform test matrix.

## Risks

- R-14: someone else claims the `sabiquant` name on crates.io before M5. This
  is accepted. The owner may choose a different strategy under OQ-25.
- R-15: MSRV churn.

## Reconsideration triggers

- Strong user demand for an early pre-release, for example to evaluate the
  API. This could be a git-only pre-release, without crates.io.
- Ecosystem changes in the SemVer or MSRV tooling conventions.

## Related requirements and documents

NFR-10; [API_DESIGN_PRINCIPLES §9](../API_DESIGN_PRINCIPLES.md#9-api-stability);
[CONTRIBUTING_GUIDELINES §8](../CONTRIBUTING_GUIDELINES.md#8-release-management);
[ROADMAP](../ROADMAP.md).

## Decision record

| Date | Decision | Approval |
|---|---|---|
| 2026-10-10 | Accepted as proposed. OQ-04: Rust 2024 edition; the MSRV is **Rust 1.99**, the stable release current when M1 started, declared in the package manifest. OQ-25: nothing is published before M5; 0.1.0 is published at M5 only with the owner's explicit approval, and no placeholder crate is published. Until then the manifest marks the package as not publishable | Project owner, when authorizing M1 ([IMPLEMENTATION_READINESS §5](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

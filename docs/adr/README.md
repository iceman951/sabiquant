# Architecture Decision Records

Architecture Decision Records (ADRs) document SabiQuant's consequential
decisions: the context, the options considered, the decision, and its
consequences. Trivial decisions do not get ADRs.

## Status values

| Status | Meaning |
|---|---|
| **Accepted — owner constraint** | Records a constraint the project owner set explicitly in the M0 brief. It is approved, and changing it needs the owner's approval |
| **Proposed** | A recommendation awaiting the owner's decision. Not binding until accepted |
| **Accepted** | Approved by the project owner (with the approval recorded in the ADR) |
| **Rejected** | Considered and declined. Kept for the record |
| **Superseded by ADR-nnnn** | Replaced by a later ADR |

Some ADRs record an owner constraint *and* propose details for carrying it
out. In those ADRs, the constraint is marked **Accepted — owner constraint**,
and each added detail is marked **Proposed** in the text.

## Index

| ADR | Title | Status | Decision in one line | Must be decided before |
|---|---|---|---|---|
| [ADR-0001](ADR-0001-native-rust.md) | Native Rust implementation | Accepted — owner constraint (details Proposed) | Idiomatic Rust; no FFI, bindings, or C++ translation; no `unsafe` in H1 | — |
| [ADR-0002](ADR-0002-standard-library-only.md) | Standard Library-only dependency policy | Accepted — owner constraint (exception process Proposed) | No third-party crates of any kind; relaxing this needs its own ADR and owner approval | — |
| [ADR-0003](ADR-0003-quantlib-compatibility.md) | QuantLib functional compatibility philosophy | Accepted — owner constraint (definitions and claims policy Proposed) | Behavioral and numerical parity, not API compatibility; claims only with evidence | M2 |
| [ADR-0004](ADR-0004-project-structure.md) | Single-crate vs multi-crate architecture | Proposed | One library crate `sabiquant` with domain modules | M1 |
| [ADR-0005](ADR-0005-date-representation.md) | Date and calendar representation | Proposed | Serial-day date over 0001–9999; immutable, data-driven calendars with provenance | M2 (date), M3 (calendar) |
| [ADR-0006](ADR-0006-numeric-policy.md) | Floating-point and numerical precision policy | Proposed | Concrete `f64`; stable formulations; tolerances by exactness class; no global epsilon | M1 |
| [ADR-0007](ADR-0007-error-handling.md) | Error handling strategy | Proposed | `Result` with per-domain non-exhaustive errors plus a crate error; no panics on input | M1 |
| [ADR-0008](ADR-0008-validation-strategy.md) | Differential validation against QuantLib | Proposed | Committed fixtures from QuantLib 1.43 and independent oracles; external generator | M2 |
| [ADR-0009](ADR-0009-api-stability.md) | Public API stability and versioning | Proposed | SemVer; 0.x until validated; edition 2024; declared MSRV; numerical changes flagged | M1 |
| [ADR-0010](ADR-0010-licensing.md) | Open-source licensing and attribution | Proposed (**licence not selected**) | Recommend MIT OR Apache-2.0; no LICENSE file until the owner approves | M1 |
| [ADR-0011](ADR-0011-explicit-state.md) | Explicit state, no global mutable settings | Proposed | No global evaluation date, singletons, caches, or "today" | M1 |

## Process

1. **Propose.** Open a pull request that adds an ADR with status *Proposed*,
   numbered sequentially. Numbers are never reused.
2. **Review.** The project owner reviews it. Discussion happens in the pull
   request.
3. **Decide.** The owner accepts or rejects it. On acceptance, the status
   changes to *Accepted*, and the ADR records the date and a link to the
   approving pull request or comment.
4. **Change.** Accepted ADRs are not edited in substance. A new ADR supersedes
   the old one, and both are cross-linked.
5. **Trace.** Requirements, specifications, and the decision register
   ([IMPLEMENTATION_READINESS §5](../IMPLEMENTATION_READINESS.md#5-decision-register))
   link to the ADRs that govern them.

## Template

Every ADR uses these sections, in this order:

1. **ADR identifier**, **Title**, **Status**, **Date**, **Decision owner**
2. **Context**: the problem and the forces at play
3. **Decision proposal**: what is proposed (or recorded)
4. **Alternatives considered**
5. **Advantages**
6. **Disadvantages**
7. **Consequences**
8. **Risks**
9. **Reconsideration triggers**: conditions that would reopen the decision
10. **Related requirements and documents**

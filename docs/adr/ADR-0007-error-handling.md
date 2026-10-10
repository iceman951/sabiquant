# ADR-0007: Error Handling Strategy

| Field | Value |
|---|---|
| ADR identifier | ADR-0007 |
| Title | Error handling strategy |
| Status | **Accepted** on 2026-10-10 ([decision record](#decision-record)) |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1 ([OQ-05](../IMPLEMENTATION_READINESS.md#5-decision-register)); operator policy before M2 ([OQ-11](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

Invalid financial input must never produce a silent wrong answer or crash the
caller. Examples include 30 February, a compounding domain violation, and a
missing reference period. QuantLib signals failure with exceptions raised by
macros. Rust has no exceptions; it uses `Result` for recoverable errors and
panics for bugs. Derive-macro crates such as `thiserror` are excluded
(ADR-0002), so error types must be implemented by hand.

## Decision proposal

1. **`Result` for every fallible operation.** An operation that can fail for
   some input returns `Result`. Sentinels, NaN, and panics are never used to
   signal failure (FR-CORE-01).
2. **One error enumeration per domain**, for example date, period, calendar,
   day-count, and rate errors. Each is **non-exhaustive**. Each variant carries
   structured context, such as the offending components, the parameter name,
   the value, and the violated domain.
3. **A crate-level error** that wraps the domain errors, with conversions from
   each, so cross-domain code can propagate any failure with `?`.
4. **Standard traits.** Every error type implements `std::error::Error`,
   `Display`, `Debug`, `Clone`, and `PartialEq`. Wrapped errors expose their
   source through `Error::source`. All of this is written by hand.
5. **Stable kind identifiers.** Each variant has a documented, stable kind
   name. Fixtures use it to express expected errors (ADR-0008). **Message
   wording is not stable.**
6. **No panics on any input** (NFR-02). Panics are reserved for internal
   invariant violations, which are defects. A public function that can panic
   documents it in a "Panics" section, and the goal is to have none.
7. **No panicking arithmetic operators in the first release.** Date and
   period arithmetic is offered as checked operations that return `Result`.
   Operator overloads are reconsidered after real use (OQ-11).
8. **Optional values for absence, `Result` for invalidity.** "This period
   maps to no frequency" is an absent value. "This date does not exist" is an
   error.
9. **No logging, no global error state, no error codes stored in globals.**

## Alternatives considered

| Alternative | Assessment |
|---|---|
| One crate-wide error enumeration only | Simple. But each function's signature admits failures that cannot happen, so precision is lost. Partly kept: the crate-level error exists *in addition to* domain errors |
| Opaque error struct with a kind accessor (like `std::io::Error`) | Flexible and stable, but less precise to match on, and the context is less structured. Rejected for domain errors |
| Boxed trait-object errors | Loses static typing; allocates. Rejected |
| Panic on invalid input, with separate checked variants | Two APIs for everything, and panics in production. Rejected |
| String errors | Not inspectable. Rejected (NFR-15) |

## Advantages

- Callers handle exactly the failures that can occur.
- Tests can assert the exact kind of failure (TS-CORE-01).
- Fixtures can express expected failures that are stable across releases.

## Disadvantages

- Hand-written trait implementations add boilerplate.
- `Result` everywhere is more verbose than operators, especially for date
  arithmetic.
- Non-exhaustive enums force callers to handle unknown variants.

## Consequences

- Each domain module defines its error type next to its operations.
- Error kind names are part of the API surface (ADR-0009).
- Documentation lists every error variant an operation can return.

## Risks

- Error types sprawl across many domains. Mitigated by one type per domain
  module.
- Users find checked date arithmetic too verbose. Revisited under OQ-11 with
  usage evidence.

## Reconsideration triggers

- Usage evidence that checked arithmetic is a significant burden. Panicking
  or saturating operators might then be offered in addition, with clear
  documentation.
- An approved ADR-0002 exception for an error-derive crate. That would change
  the implementation technique only, not this policy.

## Related requirements and documents

FR-CORE-01, FR-CORE-02, NFR-02, NFR-15; [API_DESIGN_PRINCIPLES §5](../API_DESIGN_PRINCIPLES.md#5-error-semantics);
[ADR-0009](ADR-0009-api-stability.md).

## Decision record

| Date | Decision | Approval |
|---|---|---|
| 2026-10-10 | Accepted as proposed (OQ-05): `Result` for every fallible operation; one non-exhaustive error enumeration per domain plus a crate-level error; stable kind names; no panics on input. Accepting item 7 also settles OQ-11: the first release offers checked date and period arithmetic only, with no panicking operators | Project owner, when authorizing M1 ([IMPLEMENTATION_READINESS §5](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

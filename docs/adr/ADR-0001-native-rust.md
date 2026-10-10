# ADR-0001: Native Rust Implementation

| Field | Value |
|---|---|
| ADR identifier | ADR-0001 |
| Title | Native Rust implementation |
| Status | **Accepted — owner constraint** (from the M0 brief). Sub-decision D2 below is **Proposed** |
| Date | 2026-10-10 |
| Decision owner | Project owner |

## Context

SabiQuant aims, over time, for the functional breadth of QuantLib, a large
C++ library (1,440 headers in `ql/` at version 1.43). There are four broad
ways to bring QuantLib-like functionality to Rust: bind to QuantLib through
FFI; translate its C++ into Rust; build on another library; or design a native
Rust library that uses QuantLib only as a behavioral reference. The owner's
brief requires the last of these.

## Decision proposal

**D1 (owner constraint).** SabiQuant is implemented natively in Rust with
idiomatic abstractions: ownership and borrowing, value semantics, enums,
traits, explicit `Result`-based errors, immutability, composition, and clear
module boundaries. There is no FFI to QuantLib or any C/C++ library, no
QuantLib bindings, and no mechanical or manual C++-to-Rust translation.
Abstractions are not created just to imitate C++ inheritance.

**D2 (proposed).** No `unsafe` code in H1 (M1–M5). Any later use needs its
own ADR, a written justification, and a safety argument (NFR-06).

## Alternatives considered

| Alternative | Description | Why not chosen |
|---|---|---|
| A. FFI bindings to QuantLib | Wrap the C++ library | Breaks the owner constraint; needs a C++ toolchain and Boost; inherits QuantLib's global state; binding generators are third-party crates (ADR-0002) |
| B. Translate QuantLib to Rust | Port class by class | Breaks the owner constraint and C-03; reproduces C++ patterns that fight Rust (singletons, observer graphs); licensing and provenance burden |
| C. Build on another Rust finance crate | Extend an existing crate | Third-party dependency (ADR-0002); different design goals |
| **D. Native Rust design informed by QuantLib behavior** | Independent specifications; QuantLib for comparison | **Chosen (owner constraint)** |

## Advantages

- The architecture can use Rust's strengths: explicit state, immutable
  values, thread safety, and typed errors.
- Independent provenance makes licensing simple (ADR-0010).
- No C++ toolchain or runtime is needed by users.
- Behavior is specified and owned by SabiQuant, not inherited by accident.

## Disadvantages

- Much more implementation effort than bindings.
- SabiQuant does not benefit automatically from QuantLib's fixes and
  extensions.
- There is a risk of behavioral drift from QuantLib. Differential validation
  mitigates it (ADR-0008).

## Consequences

- Every feature needs an independent specification and validation
  (REQUIREMENTS, TESTING_STRATEGY).
- The architecture is designed from domain requirements (ARCHITECTURE), with
  QuantLib's structure used as an input, not a template
  ([QUANTLIB_REFERENCE §6](../QUANTLIB_REFERENCE.md#6-architecture-analysis-what-transfers-to-rust)).
- Language bindings *for* SabiQuant (Python, C) are Out of Scope until an ADR
  approves them (SCOPE).

## Risks

- R-11 (solo-maintainer capacity) and R-13 (scope growth) are made worse by
  the effort a native implementation needs. Mitigated by milestone gating.

## Reconsideration triggers

- The owner decides to offer SabiQuant to other languages (a C ABI or Python
  bindings). That needs a new ADR, and D1 is unaffected.
- A measured need for `unsafe` (for example, a performance-critical kernel
  with proven safety) reopens D2 only.

## Related requirements and documents

C-01, C-03, NFR-06; [VISION](../VISION.md); [ARCHITECTURE](../ARCHITECTURE.md);
[ADR-0002](ADR-0002-standard-library-only.md); [ADR-0003](ADR-0003-quantlib-compatibility.md).

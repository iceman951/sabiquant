# ADR-0011: Explicit State — No Global Mutable Settings

| Field | Value |
|---|---|
| ADR identifier | ADR-0011 |
| Title | Explicit state; no global mutable settings |
| Status | **Accepted** on 2026-10-10 ([decision record](#decision-record)) |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1 ([OQ-07](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

QuantLib 1.43 keeps important financial state process-wide (B-07, B-14, B-19):

- the evaluation date and event-inclusion flags in a `Settings` singleton;
- historical fixings in an `IndexManager` singleton;
- added and removed holidays in calendar implementations shared by all
  instances;
- Business/252 counts in process-wide caches keyed by calendar name.

Singletons can be made thread-local with a compile-time option. This design
lets dependent objects recalculate automatically. It also creates hidden
coupling, order-dependent results, and data races when threads share state.
Tests then need careful saving and restoring of global settings. The owner's
brief asks for an evaluation of global evaluation dates, pricing contexts,
shared state, and their effect on thread safety and reproducibility.

## Decision proposal

1. **No global mutable state** anywhere in the crate. No singletons, no
   process-wide caches, no thread-local settings, and no static mutable data.
2. **No implicit "today".** The library never reads the system clock.
   Valuation dates are explicit inputs. The standard library has no time-zone
   database, so a local "today" could not be defined reliably anyway.
3. **Immutable calendars.** Changes produce new values (ADR-0005).
4. **No observable caching.** Caching is allowed only inside explicit, owned
   values, must never change results, and must be thread-safe. H1 has no
   caching at all.
5. **Future valuation context (M9 and later; needs its own ADR).** An immutable
   value that bundles the valuation date, curves, fixing histories, and
   quotes, passed explicitly to pricing functions. Scenarios and bumps are new
   contexts derived from existing ones. There is no relinking and no
   notification.
6. **Event-inclusion conventions** become explicit parameters of the
   operations that need them (future cash-flow analytics). In QuantLib they
   are global flags, such as whether to include cash flows on the reference
   date.

## Alternatives considered

| Alternative | Assessment |
|---|---|
| QuantLib-style global settings | Convenient, but non-reproducible, racy, and hard to test. Rejected |
| Thread-local settings (like QuantLib "sessions") | Still hidden state. Breaks when work moves between threads. Rejected |
| Global default with explicit override | Two ways to do everything, and the default leaks into results silently. Rejected |
| Observer / observable graph for automatic recalculation | Hard to express safely in Rust; hidden control flow. Rejected for now; may be revisited for performance in M9+ through an ADR |

## Advantages

- **Reproducible**: outputs depend only on the inputs (NFR-03).
- **Thread-safe by construction**: nothing is shared and mutable (NFR-05).
- **Testable**: no global setup or teardown, and tests can run in parallel.
- **Auditable**: every result's inputs are visible at the call site.

## Disadvantages

- More parameters, and in later milestones explicit context objects to pass
  around.
- No automatic recalculation when market data changes. Callers rebuild or
  re-derive values.
- QuantLib users have to adjust their workflows.

## Consequences

- Foundational operations (M1–M4) are pure functions of their arguments.
- From M9, a valuation-context ADR designs context construction, sharing
  (cheap clones of immutable data), and derivation for scenarios.
- Differential fixtures set QuantLib's global evaluation date explicitly for
  each scenario and record it.

## Risks

- R-07: with many interdependent curves and quotes, rebuilding contexts may
  be slow or awkward. Mitigation: design the context around structural sharing
  and explicit dependency graphs, and measure before optimizing.

## Reconsideration triggers

- Measured performance problems in scenario-heavy workloads (M16) that
  explicit contexts cannot solve. Any redesign would still keep state
  explicit.
- Evidence from users that the explicit-context design is unworkable in
  practice.

## Related requirements and documents

NFR-03, NFR-05, NFR-07, FR-CORE-04, FR-DATE-09, FR-CAL-03;
[ARCHITECTURE §3.11](../ARCHITECTURE.md#311-financial-state-management-adr-0011);
[QUANTLIB_REFERENCE §6](../QUANTLIB_REFERENCE.md#6-architecture-analysis-what-transfers-to-rust).

## Decision record

| Date | Decision | Approval |
|---|---|---|
| 2026-10-10 | Accepted as proposed (OQ-07): no global mutable state, no implicit "today", immutable calendars, and no observable caching. The valuation-context design remains a separate ADR before M9 | Project owner, when authorizing M1 ([IMPLEMENTATION_READINESS §5](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

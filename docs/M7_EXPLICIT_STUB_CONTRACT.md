# Local M7 explicit coupon-date contract

Status: adopted for local implementation under the user's continuing development
instruction. This extends the saved core at `86d5bb8`; it does not complete M7 or
authorize publication. Specialized rules, automatic ICMA references and cash
flows remain separate work.

## Inputs and absence

`ExplicitStubDates::new(first: Option<Date>, penultimate: Option<Date>)` requires
at least one date. If both are present, first must not follow penultimate.
Equality is allowed: one interior coupon can play both roles, yielding two outer
intervals and no regular middle interval. This is a single contractual boundary,
not repair of a collision. Both dates must be strictly inside the builder's
original start/end; range checks belong to `build_explicit` because the value
does not contain that range. Reversed roles are rejected even if adjustment
would make them coincide.

`build_explicit(tenor, rule, dates, eom, max_boundaries)` accepts positive stored
tenors and Forward/Backward only. Preserve is incompatible with Days/Weeks.
It never applies an automatic None/Short/Long policy: `stub_policy()` is None,
`explicit_stub_dates()` retains Some(dates), and tenor/rule/EOM remain supplied
metadata. Automatic and Zero construction retain None explicit-date metadata.
Definition equality/hash include explicit-date roles and stored tenor units,
even when all resulting vectors are identical. Resource caps are not retained.

## Named contractual lattice

Let regular_start be first if supplied, otherwise the original start. Let
regular_end be penultimate if supplied, otherwise the original end. Forward
anchors at regular_start; Backward anchors at regular_end. All regular middle
dates are computed from that same anchor, never iteratively from a clamped
month. The opposite middle endpoint must be exactly on the anchored lattice;
otherwise `IrregularExplicitMiddle` reports direction, tenor, anchor, endpoint
and last regular boundary. No additional implicit internal stub is generated.
Equal first/penultimate means a zero-length middle, not an invalid schedule.

Preserve activates only if this named anchor is a calendar month end. Generated
regular candidates then roll to calendar month end. Original start/end and
supplied coupon dates are never rewritten. A business-only EOM cannot activate
the roll. `lattice_anchor()` exposes this seed (None for Zero).

Outer intervals are Regular only when their other endpoint equals the adjacent
quasi-boundary of THIS SAME named lattice. Otherwise they are Stub, including
multi-tenor outer spans. In Forward, the first outer comparison is step -1 and
the final outer comparison is step k+1. In Backward, the final outer comparison
is step +1 and the first outer comparison is step -(k+1), where k is the exact
middle step count. Never subtract a tenor from a clamped interior date to infer
the next quasi-boundary. For example, Forward anchored at February 28 with
explicit first February 28 classifies a January 31 start as Stub, although
January 31 plus one month also clamps to February 28. The flag expresses named
lattice provenance, not a claim that every conceivable date convention is
irregular. Future ICMA helpers must preserve this distinction.

An adjacent classification-only quasi-date outside Date::MIN/MAX means the
in-range outer endpoint cannot equal it, hence Stub. This does not permit
missing required ICMA reference periods; generation of required reference
extensions will need its own checked range and TARGET contract.

## Guards, precedence and resources

Build validation order: supported final cap; supported rule; positive tenor;
EOM compatibility; first then penultimate strictly-interior checks; complete
original TARGET span; generation; chronological adjustment. The standalone
explicit-date constructor checks absence before role ordering. Every candidate
adjustment retains the existing market guard and cause chain. Ordinary and
termination conventions remain separate; adjustment collision or reversal is
rejected without deleting a coupon or modifying its regularity.

The cap counts all final contractual boundaries including outer endpoints and
explicit coupons. A lower-bound failure reports the caller's original cap,
never an invalid derived inner cap. At most O(final boundaries) generation
steps and storage are allowed, with the existing fallible vector reservations.
Equal-role reuse consumes one boundary. Date-range prechecks may terminate an
unused middle candidate; they do not discard an unrelated date error. No
partially usable schedule is returned.

## Evidence and validation obligations

The [QuantLib v1.43 public header](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/time/schedule.hpp)
has separate first and next-to-last dates; it does not specify this stricter
contract. Existing 46 frozen probes contain five successful explicit cases,
previously deferred. Additional isolated, ten-second-bounded observations on
the established byte-attested 1.43 wheel found equal roles accepted as one
interior date in both directions, while reversed roles could produce unordered
dates. These are observations, not financial authority or copied algorithms;
freeze them separately before claiming durable reference coverage. Reversed
roles remain invalid here. This refines research's earlier suggested rejection
of all equal explicit dates.

Required tests: both directions; first-only, penultimate-only and both; equal
roles; empty/reversed/outside roles; exact and off-lattice middle; short/long and
one-step outer intervals; independent Gregorian spans over a 400-year cycle;
clamp recovery and calendar EOM; all four stored units and huge tenors; civil
range edges; exact cap and cap minus one; all adjustment convention pairs;
collision/reversal; complete TARGET coverage and visited-candidate escape;
immutable definition/hash, retained inactive options and concurrent rebuilds.
All existing numerical, calendar, day-count, rate and schedule tests remain.

Independent source review and numerical/date validation evidence must be
recorded after implementation. Native Linux/Windows execution and the existing
release governance gates remain unverified locally.

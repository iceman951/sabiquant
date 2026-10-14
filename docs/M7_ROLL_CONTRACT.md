# Local M7 civil roll contract

Adopted after `ffa7fe3` under continuing local-only authority. This implements
the four generic civil families proposed in [the research](M7_SPECIAL_RULES_RESEARCH.md).
These are explicit analytical definitions; the labels do not certify a market
calendar, financial product, or universal QuantLib equivalence. CDS/OldCDS,
CDS maturity selection and premium-leg conventions remain subsequent M7 work.

## API and retained definition

`ScheduleBuilder::build_roll(tenor, rule, max_boundaries)` accepts the closed
`SpecialRollRule` set ThirdWednesday, ThirdWednesdayInclusive, Twentieth and
TwentiethIMM. Exact canonical display/parsing has no aliases or trimming.
`DateGenerationRule::Roll(SpecialRollRule)` preserves the existing inspector's
return type while truthfully identifying rolls; its `ALL` now lists seven
labels. Recurrent/explicit builders reject Roll through `InvalidRule`, after
cap validation and before tenor, role, EOM or calendar validation.

Positive STORED Months/Years are required. Years convert to twelve months in
wide integer arithmetic. TwentiethIMM additionally requires a month stride
divisible by three. Days/Weeks, zero and negative lengths are rejected without
normalized-unit substitution. The original stored tenor remains inspectable.
EOM, automatic stub policy and explicit coupon roles have no arguments/defaults
in this method; those Schedule inspectors and `lattice_anchor()` return None.

`Schedule::roll_provenance()` borrows an immutable `RollProvenance`, or None
for existing builders. It records requested start/end, resolved contractual
start/end, rule, original tenor and anchor year/month. Both provenance and
Schedule Eq/Hash include stored tenor length/unit and original input range;
1Y and 12M remain different definitions despite normalized Period equality.
Caps are operation budgets, not definition identity. The anchor month is not
an ordinary clamped Date recurrence seed.

## Civil lattice and endpoint rules

Let W(m) be the third Wednesday of civil month m, and D(m) its twentieth.
Every candidate is derived from the retained anchor month and positive month
stride. Adjustments never determine or re-anchor contractual dates.

| Rule | Resolved endpoints and interior lattice |
|---|---|
| ThirdWednesday | Keep requested endpoints. Anchor month is the requested start month; interior W(anchor+k*stride), k>=1, strictly within the range |
| ThirdWednesdayInclusive | Replace endpoints by W of their requested months; reject equal/reversed resolved endpoints. Interior W(anchor+k*stride), k>=1, below resolved end |
| Twentieth | Keep requested start. Anchor is the first monthly D on/after start. End is the first monthly D on/after requested end, independently of stride. Keep congruent anchor+k*stride points strictly inside the resolved range |
| TwentiethIMM | As Twentieth, but eligible anchor/end months are March, June, September and December; require stride divisible by three |

Regular means both contractual dates are actual consecutive points of this
retained lattice, separated by exactly one stride. Every other interval is
Stub; no roll interval is NonPeriodic. Off-roll initial Wednesday boundaries
can differ from reference-library flags. Off-stride terminal rolls are stubs.
No long-stub removal, endpoint deduplication, sorting, EOM inference or
implicit market calendar is performed.

## Validation, resources and errors

The builder constructor first requires requested start<end. Build precedence:
valid final cap 2..=3,652,059; rule/tenor domain; original whole TARGET span;
required anchor/start/end resolution and resolved range; whole union of
requested and resolved spans; bounded lattice generation; adjustment and
strict adjusted order. Union coverage uses phase `roll coverage`; original
coverage retains `coverage`. All dates, including Unadjusted endpoints and
adjustment candidates, retain existing TARGET validity/policy guards.

Required roll-date failure preserves rule, stored tenor, input Date, role
(`anchor`, `start`, `end`) and its actual DateError through `RollDate`.
`ResolvedRollRange` retains both ranges, rule and tenor. `InvalidRollTenor`
retains rule/tenor. Stable kinds are `schedule.roll_date`,
`schedule.resolved_roll_range`, and `schedule.invalid_roll_tenor`.
Existing cap/allocation/calendar/adjusted-order errors are reused. Every
required endpoint is resolved before generation; it cannot be dropped to fit
the cap. A future month beyond the valid resolved end is unused, so no invalid
Date needs constructing for it, even with i32::MAX Years.

Final boundaries consume the supplied cap. Vectors reserve fallibly; no
partial Schedule escapes. Positive month progress bounds generation by
returned boundaries plus one terminal comparison, with existing calendar
coverage/adjustment costs. The next coordinate is at most a valid civil month
plus 12*i32::MAX, far below i64::MAX. No allocation-failure test forces OOM.

`build_icma_context` checks its cap then returns `RollIcmaUnsupported { rule }`
(`schedule.roll_icma_unsupported`) before frequency checks. Reusing the old
clamped Date lattice would give wrong Wednesday references. Same-roll ICMA
hulls remain required subsequent work; this temporary typed exclusion does
not complete M7. Explicit independently supplied accrual conventions can
already price generic raw-period fixed coupons; this does not establish a
standard CDS premium leg.

## Evidence required

Independent civil enumeration must cover every month of a 400-year cycle,
endpoint projection and exact lattice flags, stored aliases, large and invalid
tenors, civil endpoints, cap boundaries and provenance identity. Public tests
cover all adjustment pairs, TARGET original/expanded span precedence,
collisions, typed/root causes, composition and thread-safe immutable values.
New public reference probes must preserve exact returned dates/flags/errors
and explicitly classify local policy differences. Prior fixtures remain
unchanged. Dates/status/flags use exact E0, without tolerances. Full native
debug/release, strict lint/docs and artifact checks are reported separately;
cross-target compilation is not native platform execution. Existing release,
financial/source, owner and remaining M7 gates stay open.

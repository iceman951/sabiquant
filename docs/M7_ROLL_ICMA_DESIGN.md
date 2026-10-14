# Next M7 work: ICMA references on civil roll lattices

This is a proposed follow-on to [the adopted civil roll contract](M7_ROLL_CONTRACT.md),
not an implemented feature. The current public helper deliberately returns
`RollIcmaUnsupported`. The full objective still requires same-roll references,
three CDS rule families and the remaining acceptance/source/settlement work.
This proposal does not reinterpret civil roll output as a CDS premium leg.

## Required reference definition

Reuse the existing explicit-frequency [ICMA contract](M7_ICMA_REFERENCE_CONTRACT.md):
the supplied periodic frequency's Period must normalize equal to the retained
stored tenor. Keep annual event counts unchanged and never infer them from a
stub or projected endpoint. Roll builders accept only Months/Years, so the
matching-frequency domain follows from that checked tenor, including 12M/1Y
aliases and TwentiethIMM's quarter-multiple restriction.

Let `q(k)` be the exact third Wednesday or twentieth in
`anchor_month + k * month_stride`, for signed integer k. Both Wednesday rule
families use the same Wednesday point function; their endpoint policy remains
in the source Schedule. Both twentieth families use day 20; quarter membership
is guaranteed by the original IMM anchor and stride. A clamped shift can help
locate a civil MONTH, but its resulting day must never become the quasi-date.

The minimal reference hull is `q(L) <= raw_start < raw_end <= q(R)`, where
q(L) is greatest at/before raw start and q(R) is least at/after raw end.
Return every adjacent positive determination period from L through R−1.
Exact raw endpoints need no additional period. Genuine raw regularity is
useful metadata but must not be used as an alternative hull algorithm.

For a date d, select lattice MONTH indices by signed floor/ceiling division
of `(month(d)-anchor_month)/stride`, then compare the candidate's actual
weekday/day-20 date with d. If it lies on the wrong side, move one more signed
step. This independent coordinate description can serve as a proof and oracle;
implementation may instead retain the existing bounded extension structure.
Special cases include a ThirdWednesday one-month range entirely before its
anchor, entirely after it, or crossing it; a twentieth start before its first
eligible anchor; and a terminal eligible roll that is off the tenor stride.

Signed division must use floor/Euclidean semantics, not truncation toward
zero. For Monthly Wednesday with q(0)=2024-01-17, independent hand controls
are Jan01..Jan02 → Dec20..Jan17; Jan25..Jan26 → Jan17..Feb21; and
Jan16..Jan18 → Dec20..Jan17..Feb21. The crossing case proves two periods
before either extension. In year 0001, the analogous Jan16..Jan18 request
must fail cap1 before attempting year0; cap2 must preserve the required
checked predecessor Date error.

## Errors, precedence and resource bounds

Preserve cap validation before schedule/frequency work, Once rejection and
normalized frequency mismatch. A valid roll's absence of a clamped Date seed
must no longer cause NonPeriodicIcma. The existing Zero behavior remains.
Define required quasi-date failure with an actual checked Date cause and the
original roll anchor, signed offset and extension phase; no sentinel or
unused-candidate omission is acceptable. Required predecessor at year 0 or
successor at year 10000 must fail even if every coupon date is representable.

The final period cap is shared by both extension directions. Check proved
lower bounds before constructing a disallowed required quasi-date, retaining
existing cap-versus-date precedence. Evaluation must also preserve the
lower-extension-first order and the known upper
lower bound of one period exactly when q(0)<raw_end. The typed Date cause
should retain q(0) as its exact anchor Date, signed month offset and the
generation-position index/extension phase. A mismatched frequency fails
before date arithmetic, even for enormous retained tenors.

Month strides and allowed period counts
fit i64; positive points are strictly ordered by month. Reserve vectors
fallibly and keep no mutable cache. Retain the source calendar's complete
TARGET policy and validate the whole extended hull, not only quasi-dates.
The source Schedule already validated its entire requested/resolved union;
ICMA cannot waive either that provenance or new extension coverage.

Prefer a private explicit lattice abstraction if it cleanly shares the old
bounded hull construction. It must retain the old Date/EOM path exactly and
separately implement month rolls; a generic abstraction is not a reason to
change old date causes, count precedence or raw reference outputs.

The existing extension structure can start at q(0) because the current four
builders guarantee that anchor is inside the minimal reference hull. This
property was independently reviewed; it must not be generalized to arbitrary
future anchors. Generated ICMA equality continues to compare actual ordered
references/frequency, so different source Schedule provenance may legitimately
produce equal contexts. The source Schedule retains the additional definition.

## Independent evidence before changing the public exclusion

- Enumerate all civil months of a 400-year cycle and select q points by month
  congruence; independently count Wednesdays rather than call Rust Date logic.
- Cross before/on/after raw starts/ends with matching monthly, bimonthly,
  quarterly, four-month, semiannual and annual frequencies where the rule allows
  them, including stored aliases and actual initial/final stubs.
- Check minimality, every contiguous reference period and exact cap/cap−1;
  distinguish same-month spans entirely before/after/crossing the roll.
- Compute exact integer-overlap Fractions across the complete hull; compare
  ordinary, cross-stub, reversed and zero-length accrual queries under the
  unchanged four-ULP ICMA profile. No date/flag/status tolerance is permitted.
- Exercise MIN/MAX required predecessor/successor failures, huge unsupported
  frequency tenors, cap/frequency/date precedence and every typed source chain.
- Exercise strict/projected TARGET at inception, old coverage gaps and 2026/2028
  boundaries, null-member joints, overrides and all adjustment pairs. Payment
  changes cannot change the contractual reference hull.
- Compose the generated context with the generic fixed-rate coupon factory,
  retaining complete shared convention and checking raw coupon amounts and
  partial accrual independently. This supplies no standard-CDS product claim.
- Run the unchanged ordinary ICMA fixtures/properties as regression evidence,
  plus full tests, strict lint/docs, actual package extraction and native
  source-region coverage. Keep native-platform and financial gates explicit.

The historical unsupported-kind variant is already documented as part of an
unreleased local tranche. Decide its removal/deprecation and update reachable
tests/catalogs explicitly when all four same-roll paths actually work; do not
leave a false implementation limitation in current docs after adding support.

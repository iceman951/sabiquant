# Enclosure consumer bindings (B1 and B5)

`consumer-bindings.json` binds all 72 original EP planning obligations to the
adopted 18 FR-ENCL contract groups. Each row names an exact selection from
existing immutable inputs and a future assertion in an actual public consumer.
The assertion IDs and test paths are planned bindings. They are not claims that
the tests or four public producers already exist or have executed.

The original ledger and the sealed 72-to-18 mapping are identified by SHA256.
The manifest preserves their IDs, model scope, intended faults and assertions.
EP identifiers are not additional requirements, weights, or a progress measure.
The associated contract is `docs/M8_PRICE_ENCLOSURE_CONTRACT.md`; the readable
finite matrix remains `docs/M8_PRICE_ENCLOSURE_TEST_MATRIX.md`.

## Exact selections

The 73 selectors name explicit IDs or predicates on existing fields. Each has
the matching row count, literal example IDs and a digest of **every matching
ID in original order**, with one LF after each ID. A selector such as all
Generic price rows includes the whole existing family, not a sample. The
dimension census includes all 32 Generic configurations, all seven reachable
gilt accrued lifecycles, Treasury's four source policies, and Canada's complete
shape/frequency/selection/basis scope.

The numerical families are unchanged: S has 30,920 source plans; P has 173,448
full price targets; Q has 119,631 exact admission predicates; H has 19 literal
contexts with 43 basis targets; HC has 38 conditional cash controls; HYBRID has
144 public ICMA controls. They retain their existing independent author,
checker, certificate and chronology qualifications. This index derives no new
price, source, admission or cash expectation.

S/P/Q/H/HC/HYBRID are now present as lossless generator input assets under
`validation/enclosure/references/`, together with the original price and hand
certificates. Decompression reproduces the original bytes and logical SHA256
pins exactly; these are the same existing families. The manifest retains the
future S/P/Q destinations under `validation/fixtures/bonds/` for actual FS1
feature-case consumers, which remain pending. The four legacy settlement/named
cash corpora are referenced in place without recopying or rebuilding them.

`EP-G04` now has a specific existing witness, `ENC-SOURCE-002273`, and all
its original P descendants. Its retained 2027-01-01 horizon is
`3fef1f8fc7e3f1f9`. Adding RN(184/365) to its earlier retained horizon yields
`3fef1f8fc7e3f1fa`. The consumer must read the direct cached horizon.
No boundary supplement is needed for this discriminator. Other outstanding
boundary triggers belong only to the separately owned canonical B2 packet.

## Cash composition

The HC selectors bind the existing same-plan literal contexts to the future
originating finite-evidence consumers for all four models. Those consumers
must inspect the plan, quote, basis and retained source policy, then use the
evidence from that same actual producer call. A proof wrapper cannot be
fabricated from a hand value or moved between unrelated plans.

The legacy profile selectors bind signed seven-mode rounding, arbitrary
quanta, scales, both aggregation equations, continuous/discrete Exact, required
source stages, range rescue, ambiguous images and unchanged receipt guarantees.
Their predicates select all rows of each listed profile. These are conditional
cash expectations: they do not establish an ideal-price containment premise.
When an actual producer supplies a tighter or otherwise allowed enclosure,
the consumer must evaluate its exact declared input set; it must not force
an arbitrary manual interval's decision onto that different input set.

Generic Dirty uses `SettlementInstruction::dirty_per_100`; Generic Clean uses
`SettlementInstruction::clean_accrued_per_100` with independently exact
`100*cached_AI/face` evidence. The rounded per100 getter does not prove that
ingredient. Named plans use `dirty_cash` or `clean_accrued_cash`; Canadian
ModelDirty cannot enter a SettlementDirty factory. Coupon/dividend/accrued-only
recipes stay source-only and receive no synthetic analytic certificate. The
cash receipt guarantee stays `UniqueForDeclaredEnclosure`, independently of
the originating wrapper's analytical containment fact.

## Checking the bindings

Run the structural/index checker with Python 3.9+ and its standard library:

```sh
python3 validation/enclosure/check_bindings.py
```

It validates the complete ordered 72-ID inventory, all 18 group bindings,
unique assertion/selector IDs, pending consumer statuses and all committed
financial inputs/cash artifacts' hashes, exact selections and ID digests.
To require every numerical family to have its portable input transport:

```sh
python3 validation/enclosure/check_bindings.py --require-promoted
```

This check now passes without a private work path. It rejects any missing
transport. A successful index check does not execute a numerical oracle,
certificate checker, Rust baseline, public producer or cash integration.
Those statuses appear explicitly in its output.

B3 owns final logical outcomes and symbolic debit cutpoints. B4 owns the
producer-output protocol, rigorous containment comparison and ordinary
usefulness policy. These bindings do not freeze invented runtime counters,
new tolerances or universal cash-determination promises. E3/E4/E5 must execute
the named assertions on the final integrated implementation.

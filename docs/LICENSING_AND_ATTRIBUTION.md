# Licensing and Attribution

| Field | Value |
|---|---|
| Milestone | M0; licence decided at the start of M1 |
| Document status | Analysis and decision. **Licence selected: `MIT OR Apache-2.0`** (2026-10-10) |
| Related | [ADR-0010](adr/ADR-0010-licensing.md), [CONTRIBUTING_GUIDELINES](CONTRIBUTING_GUIDELINES.md), [ADR-0003](adr/ADR-0003-quantlib-compatibility.md) |

> **Not legal advice.** This is an engineering analysis to support the owner's
> decision. The owner may wish to consult a qualified lawyer.

---

## 1. Current status

- **SabiQuant is licensed under `MIT OR Apache-2.0`**, at the user's option.
  The owner selected it on 2026-10-10 ([OQ-02](IMPLEMENTATION_READINESS.md#5-decision-register)).
- The licence texts are in the root files `LICENSE-MIT` and `LICENSE-APACHE`.
  Source files carry no per-file licence headers.
- Copyright line: "Copyright (c) 2026 The SabiQuant Contributors"
  ([OQ-03](IMPLEMENTATION_READINESS.md#5-decision-register)).
- Contributions are licensed under the same terms ("inbound = outbound"),
  with a Developer Certificate of Origin sign-off on every commit and a
  provenance attestation in every pull request (`CONTRIBUTING.md`).

## 2. Decision

**`MIT OR Apache-2.0`** (dual licence), as recommended. The full options
analysis is in [ADR-0010](adr/ADR-0010-licensing.md). It is the Rust
ecosystem's convention. Apache-2.0 gives an explicit patent licence, MIT gives
simplicity and GPLv2 compatibility, and permissive terms encourage adoption in
the finance industry.

## 3. QuantLib licence review

| Item | Finding |
|---|---|
| Licence | Modified BSD, also known as BSD 3-Clause ([licence page](https://www.quantlib.org/license.shtml); `LICENSE.TXT` at v1.43; PyPI metadata `BSD-3-Clause`) |
| Condition 1 | Source redistributions must keep the copyright notice, the list of conditions, and the disclaimer |
| Condition 2 | Binary redistributions must reproduce the notice, conditions, and disclaimer in the documentation or other materials |
| Condition 3 (no endorsement) | The names of the copyright holders, the QuantLib Group, and its contributors may not be used to endorse or promote derived products without specific prior written permission |
| Copyright holders | Many individuals and organizations, from 2000 to 2026 (listed in `LICENSE.TXT`) |
| Third-party material inside QuantLib | Code from Peter Jäckel's book *Monte Carlo Methods in Finance*; software developed by the University of Chicago, as Operator of Argonne National Laboratory; direction numbers from Stephen Joe and Frances Kuo under a BSD-style licence |

**How this applies to SabiQuant.**

1. SabiQuant **does not redistribute** QuantLib source or binaries, so
   conditions 1 and 2 are not triggered. If QuantLib code were ever included,
   which is prohibited by C-03, they would apply.
2. **Running QuantLib** (QuantLib-Python wheels) to generate reference
   numbers is *use* of QuantLib. The generator is an external tool and is not
   distributed in the crate (ADR-0008). Fixture files contain numeric results,
   and their provenance states that they came from QuantLib, as a matter of
   good practice.
3. **No endorsement.** SabiQuant describes its relationship to QuantLib
   factually, for example "validated against QuantLib 1.43 for scenario set
   X". It never implies endorsement by, or affiliation with, the QuantLib
   project or its contributors. The README includes a statement that the
   project is not affiliated.
4. **Future features that touch QuantLib's third-party material.** For example,
   Sobol sequences with Joe–Kuo direction numbers (M13) must be implemented
   from the **original sources**, under the original licences, checked at that
   time. They are never taken from QuantLib's copy.

## 4. Attribution practices

| Where | What |
|---|---|
| README | A short acknowledgement that QuantLib is the behavioral reference, plus a statement that the project is not affiliated or endorsed |
| Specifications | Citations of primary sources. QuantLib file paths cited for traceability only |
| Fixtures | A provenance header: reference implementation, version, commit, generator, and date |
| Algorithms | Each in-house algorithm cites its publication (for example Hinnant's civil-date algorithms) |
| Licence files | `LICENSE-MIT` and `LICENSE-APACHE`, added at the start of M1 |

## 5. Independence and provenance policy

| Allowed | Not allowed |
|---|---|
| Reading QuantLib source and documentation to understand behavior | Copying or translating QuantLib code into SabiQuant |
| Citing QuantLib file paths and identifiers for traceability | Reproducing large sections of QuantLib documentation |
| Running QuantLib to generate reference numbers | Copying QuantLib's test-data tables wholesale |
| Citing literature examples from their original sources | Reproducing licensed texts such as the ISDA Definitions |
| Paraphrasing conventions in SabiQuant's own words | Copying copyrighted handbook text beyond short quotations |

Contributors confirm this in each pull request
([CONTRIBUTING_GUIDELINES §3](CONTRIBUTING_GUIDELINES.md#3-pull-request-expectations-from-m1)).

## 6. Reference data and third-party publications

| Source | Owner | Terms (as understood; to be confirmed) | Use in SabiQuant |
|---|---|---|---|
| ISDA 2006 Definitions; 2021 Interest Rate Derivatives Definitions | ISDA | Proprietary, licensed | Cite section numbers; paraphrase; never reproduce. Not consulted in M0 |
| *The Actual/Actual Day Count Fraction* (1999) | ISDA | Copyright ISDA; publicly downloadable | Cite; transcribe numeric examples (facts) with citation |
| *ICMA Primary Market Handbook, Appendix A5* (March 2022) | ICMA | Copyright ICMA; publicly downloadable | Cite and paraphrase the definition |
| ECB press releases and TARGET closing days | ECB | Official publication; reuse terms to be checked | Holiday facts with citation; provenance record |
| *chrono-Compatible Low-Level Date Algorithms* | H. Hinnant | "Donated to the public domain" | Algorithmic basis for civil-date conversion, cited |
| QuantLib and QuantLib-Python 1.43 | QuantLib contributors | BSD 3-Clause | External reference tool only (§3) |
| Python standard library (`datetime`, `fractions`, `decimal`) | Python Software Foundation | PSF licence | External oracle tool only. Not distributed |
| Wikipedia, vendor glossaries (Cbonds, MathWorks) | Various | CC BY-SA or proprietary | Secondary references, cited only. No text copied |
| Government, central-bank and exchange holiday announcements (future) | Each authority | Varies. Facts are generally not copyrightable, but compilations may be protected (for example by database rights in some jurisdictions) | Primary sources for calendar data. Terms recorded per calendar |

**Holiday data rules.**

1. Use **primary, official** sources.
2. Do not copy holiday tables from other libraries, QuantLib included.
3. Record each source's terms in the calendar's provenance record.
4. Prefer encoding holiday **rules** with citations over bulk lists. Use
   lists for one-off closures.
5. If a source's terms forbid redistribution, do not use it. Find an
   alternative or leave the calendar out.

## 7. Names and trademarks

- "QuantLib" is used only nominatively, to identify the reference library.
- **No trademark search for "SabiQuant" was performed in M0.** The owner may
  want one before the first public release. The crate name was unclaimed on
  crates.io on 2026-10-10.

## 8. Open items

| Item | Reference |
|---|---|
| ~~Select the project licence~~ Done: `MIT OR Apache-2.0` (2026-10-10) | [OQ-02](IMPLEMENTATION_READINESS.md#5-decision-register) |
| ~~Copyright line, DCO sign-off, code of conduct~~ Done (2026-10-10), including the code of conduct's private reporting channel | [OQ-03](IMPLEMENTATION_READINESS.md#5-decision-register) |
| Confirm the ECB's reuse terms before encoding TARGET data | M3 (with OQ-17) |
| Optional trademark check for "SabiQuant" | Before M5 publication (OQ-25) |

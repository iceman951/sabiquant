# ADR-0010: Open-Source Licensing and Attribution

| Field | Value |
|---|---|
| ADR identifier | ADR-0010 |
| Title | Open-source licensing and attribution |
| Status | **Proposed. No licence has been selected.** No LICENSE file exists, and none is created before the owner approves one |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1, before the first code commit ([OQ-02](../IMPLEMENTATION_READINESS.md#5-decision-register), [OQ-03](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

> This ADR is an engineering analysis, **not legal advice**. The project owner
> may wish to consult a lawyer before deciding.

## Context

SabiQuant is meant to be a public open-source library. Without a licence,
default copyright applies, and others cannot safely use or contribute to the
code. SabiQuant uses QuantLib, which is under the modified BSD (3-Clause)
licence, as a behavioral reference only, and copies none of its code.
Specifications draw on licensed publications (ISDA), copyrighted handbooks
(ICMA), official publications (ECB), and public-domain algorithms (Hinnant).
Details are in [LICENSING_AND_ATTRIBUTION](../LICENSING_AND_ATTRIBUTION.md).

## Decision proposal

1. **Recommended licence: dual `MIT OR Apache-2.0`**, at the user's option.
   It is the convention of the Rust ecosystem. Apache-2.0 gives an explicit
   patent licence, and MIT adds simplicity and GPLv2 compatibility. Both are
   permissive, which suits adoption in the finance industry.
2. **Contribution licensing ("inbound = outbound").** Contributions are
   licensed under the project licence. The proposal adds the
   [Developer Certificate of Origin](https://developercertificate.org/), with
   a sign-off line on each commit (OQ-03), plus a **provenance attestation**
   in pull requests: no QuantLib or other third-party code was copied or
   translated.
3. **Copyright line**, to be chosen by the owner (OQ-03). For example
   "Copyright (c) 2026 The SabiQuant Contributors", or the owner's name. Root
   licence files only; no per-file headers (proposal).
4. **Before a licence is chosen:**
   - No LICENSE file.
   - The README says the licence is pending.
   - External code contributions are not accepted. Issues and discussion are
     welcome.
5. **QuantLib attribution.**
   - SabiQuant acknowledges QuantLib as its behavioral reference.
   - It states that it is not affiliated with or endorsed by the QuantLib
     project.
   - It never uses QuantLib's or its contributors' names to endorse or promote
     SabiQuant. This follows the spirit of the BSD licence's no-endorsement
     clause.
   - Factual statements such as "verified against QuantLib 1.43 for scenario
     set X" are allowed (ADR-0003).
6. **No QuantLib material in the repository** apart from:
   - numeric reference values produced by running QuantLib (fixtures, with
     provenance);
   - short identifiers and file paths cited for traceability.

## Alternatives considered

| Licence | Kind | Patent clause | Notes | Assessment |
|---|---|---|---|---|
| **MIT OR Apache-2.0** | Permissive (dual) | Yes (Apache-2.0) | Rust convention; two licence files | **Recommended** |
| Apache-2.0 only | Permissive | Yes | Not compatible with GPLv2-only projects | Good alternative |
| MIT only | Permissive | No explicit grant | Simplest | Acceptable, weaker on patents |
| BSD-3-Clause | Permissive | No explicit grant | Same as QuantLib; no-endorsement clause | Acceptable. Matching QuantLib has no legal necessity because no code is shared |
| MPL-2.0 | Weak copyleft (per file) | Yes | Modified files must stay MPL | Less common in Rust; friction for some users |
| LGPL / GPL | Copyleft | GPLv3: yes | Strong obligations | Likely to block adoption in finance. Not recommended |

## Advantages (of the recommendation)

- Familiar to Rust users and compatible with almost all downstream licences.
- An explicit patent grant through Apache-2.0.
- Clear, low-friction contribution terms.

## Disadvantages

- Two licence texts to carry.
- Permissive licensing allows proprietary forks without giving changes back.
  That is a deliberate trade-off in favour of adoption.

## Consequences

- At M1, after approval: add the licence files, the licence field in the
  manifest, a licence section in the README, a root `CONTRIBUTING.md` with
  the inbound=outbound statement and DCO instructions, and a pull-request
  checklist with the provenance attestation.
- Reference-data provenance includes licence and terms notes
  ([LICENSING_AND_ATTRIBUTION §6](../LICENSING_AND_ATTRIBUTION.md#6-reference-data-and-third-party-publications)).

## Risks

- R-12: an unlicensed repository discourages contributors, and code may be
  copied from QuantLib by accident. Mitigated by deciding before M1 and by the
  provenance attestation and review.

## Reconsideration triggers

- Specific legal advice to the owner.
- An organization that wants to adopt SabiQuant has a licence requirement.
- A decision to accept code under another licence. Generally avoided; needs
  an ADR.

## Related requirements and documents

C-03; [LICENSING_AND_ATTRIBUTION](../LICENSING_AND_ATTRIBUTION.md);
[CONTRIBUTING_GUIDELINES](../CONTRIBUTING_GUIDELINES.md); [ADR-0003](ADR-0003-quantlib-compatibility.md).

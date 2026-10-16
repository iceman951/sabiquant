# M8 phase B binary boundary implementation review

Review completed 2026-10-16, Asia/Bangkok, on the local worktree based on
`0ae9168`, before the phase was committed. One read-only reviewer examined
the [binary boundary contract](M8_BINARY_BOUNDARY_CONTRACT.md) implementation,
the kernel generalization, the `round_count` extraction and the documentation.
The reviewer edited no repository file. The integrator verified each finding
before changing anything.

## Code review

No code defect was found.

- **Kernel generalization.** At eight limbs, the generic `Wide<N>` kernel
  only replaces constants (`512` by the width, `8` by `N`). The new `resize`
  changes no existing algorithm. A width below three limbs is rejected at
  compile time.
- **`round_count`.** The extraction keeps the phase A error mapping, and the
  phase A oracle and hand suites pass unchanged.
- **Differential harness.** A scratch harness used its own Python struct and
  Fraction oracle on about 700,000 cases that excluded every fixture input.
  Cases included random bit patterns with NaN payloads and signs, subnormals
  by leading bit, and every exponent field with thirteen fraction patterns
  and both signs. They also included scale 37/38/39 and i128 edges, random
  1–127-bit quanta at every scale under every mode, constructed half-quantum
  ties, and outgoing limits, dyadics and binade-carry midpoints. There were no
  mismatches in kind, context or bits, in debug and in release with overflow
  checks enabled.
- **Kernel at nineteen limbs.** A verbatim copy was checked against Python
  integers over 152,712 operations, including every shift from 0 to 1299.
  There were no mismatches and no panics.

That scratch work is not retained evidence. The retained suites are listed in
the [validation report](M8_BINARY_BOUNDARY_VALIDATION.md).

## Documentation findings

The reviewer reported two minor and five nit documentation findings. All
were resolved:

| Finding | Resolution |
|---|---|
| The NI-01 row both added the binary64 boundary and called it open | Only phase C adapter, currency and rounding-stage decisions remain open |
| The F-CORE-09 rows still called the binary64 boundary open | Both rows record the phase B conversions and link the contract and validation |
| The oracle report's consumer status was stale | Names `tests/amounts_binary_oracles.rs` and the validation report |
| The ARCHITECTURE module row still said phase A | Now "Local M8 phases A–B" |
| The readiness record lacked phase B evidence | Phase B evidence row added; baseline and design wording updated |
| ADR-0006 still listed the binary64 boundary as retained | Dated phase B note; only phase C decisions remain retained |
| The crate module table and `Error::Amount` rustdoc omitted conversions | Both mention binary64 conversions |

## Integrator findings

- **Error cause.** The contract first gave `NonFinite` a nested `ValueError`
  cause. Before any code depended on it, the integrator found that
  `ValueError` is not `Eq`, while the phase A contract requires
  `AmountError: Eq`. The variant therefore records the existing `NonFinite`
  class and has no nested source, like every phase A amount error. The
  contract and FR-BIN-02 were amended, and the oracle pins the amended hash.
- **Lint expectation.** One `#[expect]` on the biased-exponent cast was
  unfulfilled: Clippy proves the 11-bit mask fits. Strict Clippy reported it,
  and it was replaced by a comment.

## Remaining limits

The reviews concern the local phase B definition. They do not certify an
ideal financial value near a quantum boundary, settlement adapters, currency
policy, bond calculations, or owner, financial, platform and release
acceptance.

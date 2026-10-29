# Self-contained Generic HYBRID unit inputs

`enclosure_hybrid_unit.py` mechanically projects the fields consumed by the
private Generic selector control from the existing immutable
`validation/fixtures/bonds/enclosure-hybrid.fixture` into typed Rust literals
in `src/bonds/enclosure/generic_hybrid_cases.rs`.

This fixes an actual extracted-package failure: the prior unit control read
the repository's `validation/` directory, which the package intentionally
excludes. The typed cases keep all 144 original source constructions, actual
cached-horizon comparisons and selected-branch assertions. Package inclusion policy
and the frozen FS1 fixture remain unchanged; no test is skipped.

The input's SHA256 is checked before projection and recorded in the generated
source. Row identity/order and all copied field shapes are checked. The script
does no date, horizon, threshold, financial or oracle arithmetic. It authors no
new reference family and uses no producer output. Original period/date strings,
literal amounts, expected horizon bits and branch facts are copied unchanged.

Initial creation is create-only:

```sh
python3 -I validation/generator/enclosure_hybrid_unit.py
```

The normal reproducibility check writes nothing:

```sh
python3 -I validation/generator/enclosure_hybrid_unit.py --check
```

The generated case array has `rustfmt::skip` so its mechanical projection has
stable bytes. Ordinary Rust formatting still applies to its declarations and
the consuming unit test. The original packaging failure and the first purely
formatting-related projection check failure are retained in the local actual
implementation evidence; no numerical expectation was repaired.

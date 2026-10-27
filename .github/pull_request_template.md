## Summary

<!-- What does this pull request change, and why? -->

Related issue, open question (OQ) or milestone task:

## Checklist

See the review checklist in
[CONTRIBUTING_GUIDELINES §5.3](https://github.com/iceman951/sabiquant/blob/main/docs/CONTRIBUTING_GUIDELINES.md#53-review-checklist).

- [ ] The specification exists or is updated, and its sources are cited.
- [ ] Tests reference their requirement and scenario IDs (`FR-…`, `INV-…`, `TS-…`).
- [ ] Errors are typed. There are no panics on input and no NaN output.
- [ ] Numerics follow NP-01 to NP-12, and every tolerance names its policy.
- [ ] No runtime or build dependency, and any dev-dependency is allowlisted in ADR-0012 with owner approval.
- [ ] No `unsafe`, no global state, and no I/O in the library.
- [ ] Public items are documented, including the QuantLib correspondence.
- [ ] `CHANGELOG.md` is updated, with a "Behavior changes" entry if any numerical or financial output changes.
- [ ] Every commit is signed off under the Developer Certificate of Origin (`git commit -s`).

## Provenance attestation

- [ ] This change does not copy or translate code, comments or test-data
      tables from QuantLib or any other third party, and it reproduces no
      licensed text. The sources used are cited in this pull request or in
      the changed files.

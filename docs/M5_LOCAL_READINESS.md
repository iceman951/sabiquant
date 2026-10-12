# Local M5 review and implementation decisions

M4 is saved locally in signed-off commit `9aa59c2`, with 281 debug and 280
release tests, scoped independent/reference fixtures and native coverage.
The owner authorized continuous local work with broad tests; this batch uses
that delegated authority to make the foundations concrete and reviewable.
It does not accept M2–M4, sign off the owner's API review, authorize remote
writes or publish a release. Manifest version/publication remain unchanged.

Independent local reviews are [API](M5_LOCAL_API_REVIEW.md),
[package](M5_LOCAL_PACKAGE_REVIEW.md) and
[validation](M5_LOCAL_VALIDATION_AUDIT.md). Their original snapshots/findings
are retained and must be distinguished from post-fix evidence.

The agent adopts these local engineering resolutions:

- Calendar/TARGET EOM, endpoint and holiday-weekend input options become
  descriptive closed enums, as the adopted API policy requires. Remove
  TARGET's `Default`; explicit `TargetCalendar::new()` continues to select
  strict TARGET. Preserve all financial rules and endpoint behavior.
- Add definition-based `Hash` to discrete ICMA/accrual contexts. Quote equality
  compares input rate, owned context and declared compounding; it does not
  represent financial equivalence and real quotes have no ordering/hashing.
- Preserve normalized Period equality and the specified calendar dispatch:
  business Days versus calendar Weeks. Document the stored-unit requirement
  and add a dedicated regression; normalized Period alone is not a safe key
  for advancement results.
- Ship only the owned `tests/support` source needed by embedded library tests
  and `clippy.toml` needed by those lint expectations, alongside the library.
  This is a narrow test/config exception to the package allowlist. External
  integration fixtures, generators, QuantLib binaries and research stay out.
- Make packaged README links repository-qualified. Unpublished local changes
  must be available at their canonical remote targets before publication;
  local-only work does not satisfy that release gate.
- Catalog identifiers parse labels, not complete financial contexts. Required
  calendars, overrides, termination dates and ICMA periods must remain explicit;
  no invented defaults or arbitrary graph serialization are introduced. Resolve
  overly broad identifier/complexity/source/error wording and add useful compiled
  composition examples and reference search aliases.

The M4 computed range/guard and rounded hybrid-switch qualifications remain
explicit. An exact rational switch would be a separate financial/numerical
behavior change with independent evidence; this batch preserves the current
documented operational switch. Native primary-platform CI, remaining US/AFB
research, TARGET maintainer appointment, security-setting verification and
independent owner/release review remain gates. Publication stays on hold.

# Security Policy

## Supported versions

SabiQuant has not been released yet. The first release, 0.1.0, is planned for
milestone M5 ([roadmap](docs/ROADMAP.md)). Until then, security fixes are made
on the `main` branch only.

| Version | Supported |
|---|---|
| `main` (unreleased) | Yes |

## Reporting a vulnerability

**Do not report security problems in public issues, discussions or pull
requests.**

Report them privately through GitHub's private vulnerability reporting: open
the repository's **Security** tab and choose **Report a vulnerability**
(<https://github.com/iceman951/sabiquant/security/advisories/new>).

Please include:

- the affected commit or version;
- the inputs that trigger the problem, and the conventions involved;
- the expected and the actual behavior;
- your platform and Rust toolchain version.

## What counts as a security issue

SabiQuant is a computation library. It reads no files, opens no network
connections, and does not use the environment or the system clock. In this
project, security issues include:

- inputs that make a public operation hang or use excessive resources, for
  example an unbounded search;
- panics reachable from the public API, since the library must not panic on
  any input;
- defects that produce silently wrong financial results in a way that could be
  exploited.

Wrong results that cannot be exploited are ordinary bugs. Report them as
issues, as described in [CONTRIBUTING.md](CONTRIBUTING.md).

## What to expect

SabiQuant is maintained by one person. Reports are acknowledged on a
best-effort basis, without response-time commitments. Fixed issues are
disclosed in the [changelog](CHANGELOG.md) and, where appropriate, in a GitHub
security advisory.

# Supply-chain pilot verification — 2026-11-15

Scope: reviewable workflow changes only; no publication, tag, merge, or settings change.
Owner authorization: security-program assignment of 2026-11-15.

## Assumptions checked against upstream

The following 14 read-only GitHub API checks resolved the actual pinned commits
and inspected action metadata at those commits, rather than assuming tag names
or current documentation matched the selected versions. All passed.

| Upstream case | Observed result |
| --- | --- |
| actions/attest-build-provenance (v4.2.2) | GitHub commit API resolved exact 40-character pin |
| actions/attest-sbom (v4.1.0) | GitHub commit API resolved exact 40-character pin |
| actions/checkout (v6.1.0) | GitHub commit API resolved exact 40-character pin |
| actions/download-artifact (v8.0.2) | GitHub commit API resolved exact 40-character pin |
| actions/upload-artifact (v7.0.2) | GitHub commit API resolved exact 40-character pin |
| dtolnay/rust-toolchain (master (stable)) | GitHub commit API resolved exact 40-character pin |
| github/codeql-action/upload-sarif (v4.39.3) | GitHub commit API resolved exact 40-character pin |
| ossf/scorecard-action (v2.5.0) | GitHub commit API resolved exact 40-character pin |
| rust-lang/crates-io-auth-action (v1.0.6 (v1)) | GitHub commit API resolved exact 40-character pin |
| zizmorcore/zizmor-action (v0.8.1) | GitHub commit API resolved exact 40-character pin |
| actions/attest-build-provenance action contract | Pinned upstream action.yml accepts configured inputs/outputs |
| actions/attest-sbom action contract | Pinned upstream action.yml accepts configured inputs/outputs |
| zizmorcore/zizmor-action action contract | Pinned upstream action.yml accepts configured inputs/outputs |
| ossf/scorecard-action action contract | Pinned upstream action.yml accepts configured inputs/outputs |

Primary references: [Scorecard template](https://github.com/ossf/scorecard/blob/main/.github/workflows/scorecard-analysis.yml),
[Scorecard publishing restrictions](https://github.com/ossf/scorecard-action#publishing-results),
[zizmor action](https://github.com/zizmorcore/zizmor-action),
[cargo-cyclonedx](https://github.com/CycloneDX/cyclonedx-rust-cargo),
[provenance action](https://github.com/actions/attest-build-provenance),
[SBOM action](https://github.com/actions/attest-sbom),
[verification CLI](https://cli.github.com/manual/gh_attestation_verify).

## Executed local checks

- actionlint 1.7.12: all four workflows clean.
- zizmor 1.32.0, default regular persona, offline audits: zero findings, no
  repository suppressions/configuration. The PR job enables online audits with
  the read-only GitHub token; local offline success does not establish that result.
- Full CI equivalent: tests (all features / no defaults / serde), all three
  Rust 1.85.0 checks, clippy with warnings denied, fmt, rustdoc with warnings
  denied, and wasm32 no-default-features build passed. Stable Rust: 1.99.1.
- Package listing: 25 files. `cargo publish --locked --registry crates-io
  --dry-run` passed and aborted the upload as expected; Cargo warned that 0.1.0
  already exists. This change does not attempt to republish it.
- `cargo package` and publish-dry-run archives were byte-identical. A separate
  shallow checkout using the same toolchain produced identical archive bytes.
  `cmp` and SHA-256 rejected a deliberate one-byte archive mutation.
- cargo-cyclonedx 0.5.9 generated CycloneDX 1.5 from the extracted crate using
  all features and all targets. Its root was irig106-types 0.1.0; its seven
  dependency name/version pairs exactly matched the packaged lockfile. The
  actual output is `irig106-types.cdx.json` (the upstream README's `bom.json`
  description is stale); the workflow uses the observed path. Its locked tool
  installation warned about yanked xml-rs 0.8.19; no runtime crate dependencies
  were added or changed.

## Publication boundary

Cargo's CLI cannot upload a preexisting archive by path. The publish job uses
the exact verification toolchain, compares a fresh dry-run archive before
obtaining the registry token, and compares the crates.io download afterward.
Attestations occur only after this last equality check. These checks fail closed
for evidence, but cannot undo a registry upload. A later failure requires owner
investigation, never overwriting a version or moving a tag.

Actual OIDC issuance, environment approval, Scorecard publishing/SARIF upload,
registry upload, and signing were not executed locally. They require their
intended main/scheduled or approved tag events. Online/offline verification
commands were checked against `gh attestation verify --help` and
`gh attestation trusted-root --help`; no new signed bundle exists yet to verify.
Release evidence is retained as Actions artifacts for 90 days, not permanent
GitHub Release assets. No SLSA level or existing-release provenance is claimed.

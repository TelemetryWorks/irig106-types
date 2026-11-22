# Organization release-pipeline pilot — 2026-11-22

Scope: one adoption PR for irig106-types, no tag/publication/settings change.
Read the entire TelemetryWorks/.github `docs/TRUSTED-PUBLISHING.md` at merged
commit `9bc516d` before implementation. Origin is
`https://github.com/TelemetryWorks/irig106-types.git`; base main is
`bd8cd4115e1b7ec89edbcf7de03e5a4406d6fd2e`. The root package remains 0.1.0.

## Contracts and read-only evidence

- Crates.io 0.1.0 metadata: SHA-256
  `2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420`,
  publishing commit `3c517358694646652939c236968b202ebf76e28a`, run
  `43727921675`. The committed fixture matches that checksum, has the same
  `.cargo_vcs_info.json` commit, and contains the original 25 files/license.
- Attempt-specific GitHub API: historical publishing attempt 2 succeeded.
  This is historical evidence, not a new release or an attestation.
- Public crate API: `crate.trustpub_only == true`.
- Environment API: `crates-io` has the owner reviewer and
  `can_admins_bypass == false`. Deployment policy API: only `v*` tags.
  No settings were changed. The private registry publisher registration was
  not inspected; the owner must verify its filename/environment before tagging.
- CI API for exact base-main SHA: `.github/workflows/ci.yml`, event push,
  branch main, conclusion success. Gate tests also exercise a later API page
  and reject wrong workflow, SHA, event, branch, and conclusion.
- Docs.rs `/crate/irig106-types/0.1.0/status.json`: HTTP 200,
  `{"doc_status":true,"version":"0.1.0"}`. The initially probed
  `builds.json` endpoint returned 404; it is not used by the pipeline.
- Upstream tag refs resolve the five selected actions to their exact existing
  SHA pins: checkout v7.0.1, upload-artifact v7.0.2, download-artifact v8.0.2,
  attest-build-provenance v4.2.2, crates-io-auth-action v1.0.6. At-pin metadata
  confirms subject-path/bundle-path, artifact name/path and pattern/merge inputs.
- Native Rust 1.99.1 and MSRV 1.85.0 execute. cargo-cyclonedx 0.5.9 was
  installed with --locked; it warns about its locked yanked xml-rs 0.8.19.
  This is a build tool, not a new required crate dependency.

These are actual primary-endpoint/tool observations (over ten distinct API
and registry checks), not inferred publication success. No registry credentials,
OIDC issuance, signing or release mutation were exercised.

## Executed controls

- 32 Python-stdlib tests: checksum mutation/path/duplicate/symlink rejection;
  dated/committed/matching notes; intended exact-commit paginated CI; qualified
  annotated tag acceptance and failures; dry-run/manual-tag publish=false;
  exact Rust and non-compiling packaging; existing registry comparisons;
  numeric draft recovery and never-replace assets; pending docs.rs is not passed.
- Actual manual-from-tag dry gate: publish=false and a visible finding for
  absent `docs/releases/v0.1.0.md`. No record was created/backfilled.
- Actual known-good registry check passes against the committed fixture and
  registry checksum. It is not a comparison of two newly downloaded archives.
- Local full CI equivalent passed on Rust 1.99.1/MSRV 1.85.0, as did packaged
  source compilation and SBOM generation. Package audit caught a nested
  fixture README matching Cargo's basename include pattern; renaming it to
  PROVENANCE.md keeps repository-only fixture documentation out of the crate.
- actionlint 1.7.12 and zizmor 1.32.0 regular persona pass the workflow.
  The build/dry-run execution evidence and PR run ID are recorded in the PR.

## Decisions and feedback for the organization standard

1. **Immutable Actions artifacts cannot be appended to.** Build uploads
   `release-<version>` once; attest uploads the canonical JSONL bundle as a
   separate immutable `release-<version>-attestations` sidecar. GitHub release
   merges those same-run downloads into one logical bundle, without overwriting
   the build artifact. The standard should specify this transport explicitly
   rather than imply a later job can append to the original artifact.
2. **SBOM provenance differs from the earlier pilot.** The standard requires
   SLSA provenance for the SBOM file itself. The new single multi-subject bundle
   attests crate + SBOM; the earlier, never-published SBOM-predicate layout is
   superseded. Verification docs now verify both files, not extract a CycloneDX
   predicate from crate provenance. No SLSA level is claimed.
3. **Define pending docs.rs as non-success.** This implementation uses ten
   bounded status observations; absent/false status at the deadline is reported
   as pending/unsuccessful and fails verification, never a green docs build.
4. **Release-input PR filters include preparation inputs.** In addition to
   workflows/scripts, they cover manifests, sources/tests, README/CHANGELOG,
   and release notes so a version-bump PR also exercises the dry run.
5. **The new standard supersedes the former second environment approval.**
   Attestations now precede the only publication approval, and only publish has
   environment crates-io. GitHub release is not an additional deployment gate.
6. **Privileged jobs also check the immutable event context.** They require
   the gate output AND a push of a `v*` tag, independently of the gate script.
   PR/manual source can change the gate implementation, so its boolean alone
   should not be the sole condition for signing/publication. The standard
   should spell out this defense; its required gate-output condition remains.
7. **Checkout the immutable event SHA directly.** CodeQL identified the
   cross-job gate commit output as untrusted for manual dispatch (nine cache
   poisoning alerts). All job checkouts now use `github.sha`; the gate checks
   its dereferenced event commit against the annotated tag/main/CI. This also
   removes a gate-script-controlled source-ref surface without suppressing
   findings. The gate commit remains evidence, not the checkout authority.
8. Release-note derivation is concrete: first heading is title, remaining body
   equals the dated CHANGELOG entry excluding its heading and link definitions.
   Expanded workspaces fail explicitly until ordered family handling is adopted;
   this root library is the complete current release set.

Nothing in these checks authorizes re-tagging or re-publishing 0.1.0. The first
real use remains the next independently reviewed, owner-authorized release.


## Coordinator decision (a): amendment fcceb02, PR #21 follow-up

The amended standard was read in full. Publish and GitHub release now download
exactly `release-<version>{,-attestations}` with merge-multiple. Explicit
producer upload paths select three payload files and one disjoint sidecar
filename. The pinned download action uses minimatch 10.1.1; an isolated tool
check of 12 stable/pre-release/build-metadata name cases confirms the pattern
selects only the two intended artifacts. No npm/project dependency was added.

A shared verifier validates checksums and both payload signatures with bundle,
repository, signer workflow, source-ref and source-digest constraints. Publish
calls it before repackaging or emitting an upload decision; GitHub release
calls it before any mutation. Post-publication verification consumes both
frozen artifacts and compares every attached file with them before verification.
No real signature, publishing or attachment path was invoked during this change.

Latest metadata is selected from a fresh, paginated Release list using numeric
semantic-version precedence. Highest stable gets true; older/pre-release gets
false and pre-releases carry the pre-release flag. Draft creation uses false;
finalization applies the computed policy after a second tag/commit check.
The new second-check test changes the tag only on its second observation and
asserts no metadata mutation. Additional tests cover bad/missing signatures,
verification-before-repackaging, multi-digit version ordering, older uploads,
pre-release metadata and draft exclusion. All 49 control tests pass locally.

The independent tag-push conditions and immutable event-SHA checkouts remain;
the read-only gate checkout is now explicit too. File name, crates-io environment,
job graph and permission table are unchanged. Actionlint and regular-persona
zizmor pass. The updated PR's actual dry-run evidence is recorded in its body.

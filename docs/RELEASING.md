# Releasing irig106-types

Owner policy (2026-11-13): **crates.io Trusted Publishing (OIDC) from GitHub
Actions only**, for every TelemetryWorks repository. Never create or store a
long-lived `CARGO_REGISTRY_TOKEN`, run `cargo login` for releases, or run a
real `cargo publish` locally. Local `cargo publish --dry-run` is allowed;
it packages and verifies without uploading. Version 0.1.0 is the first real
release; 0.0.1 was a name placeholder. Do not reuse an existing version.

## One-time owner setup (before any release tag)

These are owner actions, not actions performed by the preparation PR.

1. On **GitHub → TelemetryWorks/irig106-types → Settings → Environments**,
   create **`crates-io`**.
2. Under **Required reviewers**, add the owner and save the protection rule.
   Do not enable “Prevent self-review” if the same sole owner will push the
   tag and approve it; that combination would prevent approval. Approval
   must still be explicit. Disable administrator bypass of protection rules.
3. Under **Deployment branches and tags**, choose **Selected branches and
   tags**. Add a **Tag** rule with pattern **`v*`**. Add no branch rule.
   Do not add registry-token secrets to the repository or environment.
4. On **crates.io → irig106-types → Settings → Trusted Publishing**, add a
   **GitHub** publisher with these exact fields:

   | Field | Value |
   |---|---|
   | Owner | `TelemetryWorks` |
   | Repository | `irig106-types` |
   | Workflow | `release.yml` |
   | Environment | `crates-io` |

5. Check the saved publisher and environment rules before creating a tag.
   GitHub can auto-create an unprotected environment when a workflow names
   a nonexistent one; naming `crates-io` in YAML alone does not protect it.
   The owner must configure it first. Do not substitute a stored token if
   Trusted Publishing fails.

### Recommended release-tag ruleset (owner setup)

In **GitHub → Settings → Rules → Rulesets → New tag ruleset**, create an
**Active** ruleset targeting tag names **`v*`**. Enable **Restrict creations**,
**Restrict updates**, and **Restrict deletions**. Grant bypass permission
only to the owner: in this organization repository, use a dedicated team
whose sole member is the owner. Do not grant bypass to broad roles such as
all repository administrators, other teams, or GitHub Apps. With these rules,
only the owner can create, update, or delete matching tags. Restrict access
to edit the ruleset itself as well; verify its effective bypass list.

This is a recommended owner configuration, not a settings change performed
by a release-preparation PR. The owner must still follow the policy against
moving or deleting release tags. The environment's `v*` deployment rule
controls deployment eligibility, not who can push tags; it does not replace
this ruleset. See [GitHub tag rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository).

See [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing)
and [GitHub environment setup](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments).

## Release procedure

1. Open a version-bump/release-preparation PR updating `Cargo.toml`,
   `Cargo.lock` if necessary, README, and CHANGELOG. Use a version that has
   never been published. Consolidate `Unreleased` into the version's
   changelog entry and set its date **in the release-prep PR** to the intended
   tag day; update it before merge if that day changes. Review the
   package list and run the [development checks](../README.md#development-checks),
   `cargo package --locked --list`, and
   `cargo publish --locked --registry crates-io --dry-run`.
2. Obtain independent review, pass CI, and **merge the PR into main**.
   Ensure the one-time setup above is complete. Confirm the intended version
   is not already on crates.io; versions cannot be overwritten.
3. From a clean local checkout of the reviewed main commit, the owner creates
   and pushes an **annotated tag**. For 0.1.0:

   ```sh
   git switch main
   git pull --ff-only origin main
   git status --short
   # Stop if the checkout is not clean. Inspect HEAD and Cargo.toml version.
   git log -1 --oneline
   git tag -a v0.1.0 -m "Release irig106-types 0.1.0"
   git push origin refs/tags/v0.1.0
   ```

   For later releases, substitute the merged version `vX.Y.Z`. Tag the
   reviewed commit on main, never an unmerged PR branch. Do not move or
   force-update a release tag.
4. Open **Actions → Release** for that tag. The `verify` job checks that the
   event is a tag push, the tag equals `v` plus `Cargo.toml`'s version, and
   the checked-out event commit is an ancestor of `origin/main`. It then runs
   the full CI equivalent: all-features/no-default/serde tests, Rust 1.85.0
   checks in all three configurations, clippy with warnings denied, fmt,
   docs with warnings denied, wasm32, package listing, and publish dry run.
   This job has only `contents: read`, with no OIDC permission or registry token.
5. When `verify` succeeds, the `publish` job waits for the owner's
   **`crates-io` environment approval**. Follow the independent approver
   checklist below; a green `verify` job alone is insufficient. Inspect the
   exact tag, commit, logs, and package contents. The job checks out the
   verified commit SHA, not a mutable main/tag reference. Main ancestry in
   the job is checked against the snapshot fetched during verification.
6. After approval, the publish job obtains a short-lived token with
   `rust-lang/crates-io-auth-action` (full-SHA pin for v1.0.6). Only that job
   has release permissions `id-token: write`, `attestations: write`, and
   `contents: read`. Only its publish step receives
   `CARGO_REGISTRY_TOKEN` from `steps.auth.outputs.token`. The action revokes
   the temporary token on job completion. There are no other secrets.
   Before authentication, it checks the downloaded verification artifact's
   checksums and reproduces its crate with the exact Rust version recorded
   by `verify`. Cargo packages again during publishing; the job downloads
   the crates.io archive and requires byte identity with the verified crate
   before creating provenance and SBOM attestations. A mismatch fails the
   release evidence steps; it cannot undo an upload already accepted by
   crates.io. Stop and investigate rather than retrying a different payload.
7. The separate `release-evidence` job then waits for the same protected
   `crates-io` environment approval. Inspect the successful publish/attestation
   results before approving. Only this job has `contents: write`, with no OIDC
   permission or registry credentials. It downloads and checks the evidence,
   creates a **draft GitHub Release for the existing tag** (`--verify-tag`),
   uploads the assets, and publishes the Release after uploads succeed.
   It does not change the tag or the repository's Latest release selection.
8. Wait for the workflow to finish, then verify the result on crates.io as
   described below. Do not call a release complete merely because a tag exists.

The workflow triggers **only** on tag pushes matching `v*.*.*`; it has no
manual dispatch or branch/PR publish path. All third-party actions are pinned
to full commit SHAs. Rust toolchains are installed with the runner's `rustup`.
The `crates-io` environment and publisher configuration are owner prerequisites,
not resources created/configured by this workflow.

## Approver checklist before crates-io deployment

Before approving the pending deployment, the owner must independently:

- Get the run's actual commit SHA and tag from the GitHub Actions run page,
  not only the `verify` job's output. Confirm the annotated release tag
  points to that commit and the version matches the reviewed release PR.
- Fetch current main in a trusted checkout and confirm the run's commit is
  on main. Do not rely on the ancestry assertion in the tag's workflow.
- Identify the reviewed `release.yml` on main by its immutable commit SHA
  (for example, the independently reviewed merge commit). Confirm that the
  run commit's `.github/workflows/release.yml` is **byte-identical** to that
  reviewed file. Do not trust a tag's altered workflow to verify itself.
- Inspect verification results and package contents. If ancestry or workflow
  identity differs, **do not approve**, even if `verify` is green. Resolve
  the difference through independent review first; do not bypass protection.

Example independent checks (replace both placeholder SHAs using the run page
and reviewed main history; do not take them from untrusted workflow output):

```sh
set -eu
release_commit=REPLACE_WITH_RUN_COMMIT_SHA
reviewed_main_commit=REPLACE_WITH_REVIEWED_MAIN_COMMIT_SHA
git fetch origin main
git merge-base --is-ancestor "$release_commit" origin/main
git merge-base --is-ancestor "$reviewed_main_commit" origin/main
release_workflow_blob=$(git rev-parse "$release_commit:.github/workflows/release.yml")
reviewed_workflow_blob=$(git rev-parse "$reviewed_main_commit:.github/workflows/release.yml")
test "$release_workflow_blob" = "$reviewed_workflow_blob"
```

Matching Git blob IDs establish identical file bytes. These checks complement
the environment approval and tag ruleset; they are not delegated to `verify`.

## Verify the published crate

- Open [irig106-types on crates.io](https://crates.io/crates/irig106-types).
  Confirm the new version (first real release: **0.1.0**), Apache-2.0 license,
  repository link, rendered README, features, and Rust 1.85 requirement.
- Confirm the matching Release workflow succeeded on the reviewed tag commit.
  Inspect the downloaded crate's normalized `Cargo.toml`, `Cargo.toml.orig`,
  and `.cargo_vcs_info.json`; its Git SHA must match that verified commit.
- Fetch the crate from the registry in a separate temporary consumer project,
  rather than through a path or git dependency. For 0.1.0, a minimal manifest is:

  ```toml
  [package]
  name = "irig106-types-release-check"
  version = "0.0.0"
  edition = "2024"
  rust-version = "1.85"

  [dependencies]
  irig106-types = { version = "=0.1.0", default-features = false }

  [features]
  serde = ["irig106-types/serde"]
  ```

  Create `src/lib.rs` containing `#![no_std]` and a small use of `Rtc`, then
  run `cargo +1.85.0 check`, `cargo +1.85.0 check --features serde`, and
  `cargo +stable check --target wasm32-unknown-unknown`. Ensure the lockfile
  resolves `irig106-types` from crates.io at exactly 0.1.0. Install that target
  with rustup if needed. This checks consumption; it does not publish anything.
- Optional read-only metadata check:
  `cargo info irig106-types@0.1.0 --registry crates-io`.

## Release evidence and attestations

For releases made with this workflow, `verify` packages the crate and runs
all checks, then generates `sbom.cdx.json` with **cargo-cyclonedx 0.5.9**
(`--locked` installation). The SBOM describes the extracted packaged
manifest with all features and all target dependencies; it is an inventory
of the source crate, not a claim about a particular consumer's binary.
The same run's immutable `release-package` Actions artifact transfers the
crate, SBOM, and `SHA256SUMS` to the environment-gated publish job. Only
that release job can write attestations or request a release OIDC token.

After confirming the registry serves exactly that crate, the job creates
[SLSA build provenance](https://github.com/actions/attest-build-provenance)
and a [CycloneDX SBOM attestation](https://github.com/actions/attest-sbom)
with the crate's SHA-256 as subject. Both are stored with GitHub and their
Sigstore bundles are copied to the **`release-evidence` Actions artifact**,
alongside the crate, SBOM and checksums covering all four payload files.
The separately gated `release-evidence` job copies all five files to the
matching **GitHub Release** as assets. GitHub Release assets are not subject
to Actions artifact expiry, making them the durable download location for
sites preparing offline verification months or years later. They persist
until an authorized user deletes the assets, Release, or repository; sites
should still archive their own copy. The Actions artifacts are a temporary
transfer/fallback, expiring after 90 days (or sooner under repository policy).
No existing publication is retroactively attested,
and no SLSA assurance level or badge is claimed by this change.

### Online verification

Use a current [GitHub CLI with attestation support](https://cli.github.com/manual/gh_attestation_verify).
Choose the version and reviewed commit independently from
main history and the tag; do not accept arbitrary values from a downloaded
checksum file. For example (replace both values):

```sh
set -eu
version=X.Y.Z
commit=REPLACE_WITH_REVIEWED_TAG_COMMIT_SHA
gh release download "v$version" -R TelemetryWorks/irig106-types -D evidence
cd evidence
sha256sum -c SHA256SUMS
curl --fail --location "https://crates.io/api/v1/crates/irig106-types/$version/download" -o registry.crate
cmp "irig106-types-$version.crate" registry.crate
sha256sum "irig106-types-$version.crate" registry.crate
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/release.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/release.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners --predicate-type https://cyclonedx.org/bom
```

If the Release-assets job failed after publication, the same-run Actions
artifact remains a temporary fallback while retained:
`gh run download RUN_ID -R TelemetryWorks/irig106-types -n release-evidence -D evidence`.
Obtain that run ID independently from the reviewed tag's workflow run.

`SHA256SUMS` detects transfer errors; the signed attestations establish
identity. The first verification requires the SLSA provenance predicate by
default; the second requires the SBOM predicate. To inspect the signed
SBOM, add `--format json` and examine the verified statement's `predicate`;
compare its JSON content with `sbom.cdx.json`. The SBOM predicate is bound
to the crate hash, not to a separate signature over the JSON file's bytes.

### Offline verification

While still online, obtain trusted roots independently using
`gh attestation trusted-root > trusted_root.jsonl`. Save that trusted file,
the entire evidence directory (including both `.sigstore.json` bundles),
and the independently verified tag/commit identity. Transfer them securely
to the offline machine. Do not replace your trusted roots with an
untrusted artifact's supplied trust configuration.

On the offline machine, set `version` and `commit` as above, enter the
evidence directory, and run:

```sh
set -eu
sha256sum -c SHA256SUMS
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types --bundle provenance.sigstore.json \
  --custom-trusted-root ../trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/release.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types --bundle sbom.sigstore.json \
  --custom-trusted-root ../trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/release.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners --predicate-type https://cyclonedx.org/bom
```

The local bundles and trusted roots remove the need for network lookups.
An offline check authenticates the saved crate; comparing it with the
live crates.io download must happen online. A failed attestation step
**after publication** means the version may already exist: inspect the
registry and run logs, preserve the evidence, and resolve through review.
Never move the tag or republish under the same version to repair evidence.

The durable-assets job fails rather than overwriting an existing Release or
its assets. If an upload fails, a draft may remain. Inspect the existing
Release/draft and preserve the original same-run artifact; resolve recovery
through owner review. Do not delete or replace previously published evidence
or bypass the environment gate just to make a retry pass. No settings change
is needed: both gated jobs use the existing protected `crates-io` environment.

## Package contents and failures

`Cargo.toml` explicitly includes source, tests, manifest/lockfile, LICENSE,
README, and CHANGELOG. Cargo adds normalized manifest/VCS metadata. Repository
CI, release instructions, and standard-review documents stay on GitHub; README
links to those documents use GitHub URLs so the packaged README remains useful.

If verification fails, correct it through a reviewed PR. Do not bypass the
checks or environment protection, force-push a tag, store a fallback token,
or publish locally. If authentication fails, the owner checks the four
publisher fields, the environment approval/rules, and `id-token: write`.
If publishing reports that the version exists, inspect crates.io first: an
upload may have completed before a later workflow failure. Never overwrite
or assume it is safe to publish a different payload under the same version.

References: [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html),
[Cargo publish dry run](https://doc.rust-lang.org/cargo/commands/cargo-publish.html),
[auth action v1.0.6](https://github.com/rust-lang/crates-io-auth-action/tree/3268eb413133e2960fa4ae0e963ca76380c54cd9).

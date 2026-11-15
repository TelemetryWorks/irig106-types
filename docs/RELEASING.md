# Releasing irig106-types

Owner policy (2026-11-13): **crates.io Trusted Publishing (OIDC) from GitHub
Actions only**, for every TelemetryWorks repository. Never create or store a
long-lived `CARGO_REGISTRY_TOKEN`, run `cargo login` for releases, or run a
real `cargo publish` locally. Local `cargo publish --dry-run` is allowed;
it packages and verifies without uploading. Version 0.1.0 is the first real
release; 0.0.1 was a name placeholder. Do not reuse an existing version.

## Historical publication: 0.1.0 (2026-11-15)

Version **0.1.0 was published by attempt 2 from `.github/workflows/release.yml`**.
The [crates.io version record](https://crates.io/api/v1/crates/irig106-types/0.1.0)
identifies GitHub run **`43727921675`** and commit
`3c517358694646652939c236968b202ebf76e28a` in `trustpub_data`;
the [successful publishing attempt (2)](https://github.com/TelemetryWorks/irig106-types/actions/runs/43727921675/attempts/2)
records that workflow path. [Attempt 3](https://github.com/TelemetryWorks/irig106-types/actions/runs/43727921675/attempts/3)
was a later rerun that failed because 0.1.0 already existed, so the run's
landing page shows failure rather than attempt 2's successful publication.
See the [organization SECURITY-PROGRAM attempt history](https://github.com/TelemetryWorks/.github/blob/8c3fd1b50ada9195bddd810ecc3a83ad756ab27c/SECURITY-PROGRAM.md#verified-release-and-attempt-history).
This predates the supply-chain pilot: that workflow did not produce the
new SLSA or SBOM attestations. Renaming the workflow does not change the
publication record or retroactively attest 0.1.0.
The verification examples below apply to future releases made by
`trusted-publishing.yml` after the pilot is merged.

## Workflow filename migration (owner action before the next tag)

Owner org standard (2026-11-15): every TelemetryWorks publishing workflow
uses **`.github/workflows/trusted-publishing.yml`**. The displayed Actions
workflow name remains **Release**.

**Required owner step, dated 2026-11-15 — complete and verify BEFORE the
next tag:**

1. Under **crates.io → irig106-types → Settings → Trusted Publishing**,
   update the existing publisher's historical filename **`release.yml`** to
   **`trusted-publishing.yml`**. Keep owner `TelemetryWorks`, repository
   `irig106-types`, and environment `crates-io`.
2. Save, reopen the publisher settings, and independently verify all four
   saved fields against the table below **before creating or pushing the
   next release tag**. Do not assume the workflow rename updates crates.io.
3. Confirm **`trustpub_only` remains enabled**. A filename mismatch fails
   authentication and safely blocks the release. Do not substitute a stored
   token or local publication to bypass it.

This is an owner settings action, not performed by this PR.

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
   | Workflow | `trusted-publishing.yml` |
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
   Ensure the one-time setup and dated publisher migration/verification
   above are complete before tagging. Confirm the intended version
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
7. After `publish` completes, `release-assets` becomes a **second pending
   `crates-io` deployment requiring its own explicit approval**. The first
   approval does not approve this job. Follow the
   [second approval checklist](#second-approval-release-assets-deployment)
   before approving it. Only this job has `contents: write`, with no OIDC
   permission or registry credentials. It downloads and checks the evidence,
   rechecks that the remote tag resolves to the verified commit, then creates
   a **draft GitHub Release for that existing tag** (`--verify-tag`) if needed.
   It uploads missing assets, compares existing assets byte-for-byte without
   overwriting, and publishes a draft only after all five files are present.
   It does not change the tag or the repository's Latest release selection.
8. Wait for the workflow to finish, then verify the result on crates.io as
   described below. Do not call a release complete merely because a tag exists.

The workflow triggers **only** on tag pushes matching `v*.*.*`; it has no
manual dispatch or branch/PR publish path. All third-party actions are pinned
to full commit SHAs. Rust toolchains are installed with the runner's `rustup`.
The `crates-io` environment and publisher configuration are owner prerequisites,
not resources created/configured by this workflow.

## Approver checklist before crates-io deployment

Before approving the first pending deployment (`publish`), the owner must
independently:

- Get the run's actual commit SHA and tag from the GitHub Actions run page,
  not only the `verify` job's output. Confirm the annotated release tag
  points to that commit and the version matches the reviewed release PR.
  Record the run ID, tag, commit SHA, and reviewed workflow Git blob ID for
  comparison at the second approval.
- Fetch current main in a trusted checkout and confirm the run's commit is
  on main. Do not rely on the ancestry assertion in the tag's workflow.
- Identify the reviewed `trusted-publishing.yml` on main by its immutable commit SHA
  (for example, the independently reviewed merge commit). Confirm that the
  run commit's `.github/workflows/trusted-publishing.yml` is **byte-identical** to that
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
release_workflow_blob=$(git rev-parse "$release_commit:.github/workflows/trusted-publishing.yml")
reviewed_workflow_blob=$(git rev-parse "$reviewed_main_commit:.github/workflows/trusted-publishing.yml")
test "$release_workflow_blob" = "$reviewed_workflow_blob"
```

Matching Git blob IDs establish identical file bytes. These checks complement
the environment approval and tag ruleset; they are not delegated to `verify`.

## Second approval: release-assets deployment

After publication, the owner must separately review and approve the second
pending `crates-io` deployment for `release-assets`. Before approving:

- Compare the pending deployment with the first approval's recorded evidence:
  it must be the **same workflow run ID, tag, commit SHA, and reviewed
  `.github/workflows/trusted-publishing.yml` Git blob ID**. Repeat the
  independent blob comparison above; a green publish job does not replace
  this identity check.
- In that run's `publish` job, confirm **Confirm crates.io serves the exact
  verified archive** passed. This is the byte comparison against the actual
  crates.io download; a successful upload alone is insufficient.
- Confirm **both** SLSA provenance and CycloneDX SBOM attestations exist for
  that exact crate's **SHA-256 digest**. Either inspect the successful
  **Attest published crate provenance** and **Attest CycloneDX SBOM for the
  same crate** steps and their subject digests in that run, or run both
  signer/tag/commit-constrained `gh attestation verify` commands in the
  verification paths below against the crates.io crate. Require both
  predicates to bind to the same digest as the byte-verified crate.

If any evidence is missing, failed, or differs from the first approval,
**do not approve**. Investigate through owner review; do not bypass the gate
or rerun publication just to obtain release assets. Only after these checks
should the owner approve `release-assets` to retain the five evidence files.

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
The separately gated `release-assets` job copies all five files to the
matching **GitHub Release** as assets. GitHub Release assets are not subject
to Actions artifact expiry, making them the durable download location for
sites preparing offline verification months or years later. They persist
until an authorized user deletes the assets, Release, or repository; sites
should still archive their own copy. The Actions artifacts are a temporary
transfer/fallback, expiring after 90 days (or sooner under repository policy).
No existing publication is retroactively attested,
and no SLSA assurance level or badge is claimed by this change.

### Path A: permanent Release set and offline verification

Use a current [GitHub CLI with attestation support](https://cli.github.com/manual/gh_attestation_verify).
These shell examples assume a POSIX system with `sha256sum`, `curl`, and
Python 3. Choose the version and reviewed commit independently from main
history and the tag; never trust a downloaded checksum file to choose them.
While online, download the permanent Release set, fetch trusted roots
independently, and compare the Release crate with the crates.io download:

```sh
set -eu
version=X.Y.Z
commit=REPLACE_WITH_REVIEWED_TAG_COMMIT_SHA
gh release download "v$version" -R TelemetryWorks/irig106-types -D evidence
gh attestation trusted-root > trusted_root.jsonl
cd evidence
sha256sum -c SHA256SUMS
curl --fail --location "https://crates.io/api/v1/crates/irig106-types/$version/download" -o registry.crate
cmp "irig106-types-$version.crate" registry.crate
```

Save the entire evidence directory, `trusted_root.jsonl` in its parent, and
the independently verified version/commit identity. Transfer them securely
to the disconnected site. Do not accept an artifact supplier's replacement
trust configuration in place of independently fetched trusted roots.
On the offline machine, set `version` and `commit` to those trusted values,
enter the evidence directory, and run:

```sh
set -eu
sha256sum -c SHA256SUMS
cmp "irig106-types-$version.crate" registry.crate
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types --bundle provenance.sigstore.json \
  --custom-trusted-root ../trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/trusted-publishing.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners
gh attestation verify "irig106-types-$version.crate" \
  -R TelemetryWorks/irig106-types --bundle sbom.sigstore.json \
  --custom-trusted-root ../trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/trusted-publishing.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners --predicate-type https://cyclonedx.org/bom \
  --format json > verified-sbom.json
python3 - <<'PYCHECK'
import json
from pathlib import Path
results = json.loads(Path('verified-sbom.json').read_text())
boms = [r['verificationResult']['statement']['predicate'] for r in results]
local = json.loads(Path('sbom.cdx.json').read_text())
if local.get('bomFormat') != 'CycloneDX' or not boms or any(b != local for b in boms):
    raise SystemExit('Release SBOM differs from the verified predicate')
PYCHECK
```

The bundles and trusted roots avoid network lookups. The offline `cmp`
compares a saved crates.io download, not the live registry; obtaining and
checking that download must happen while connected. `SHA256SUMS` detects
transfer errors, including changes to the bundles. Attestation verification
establishes identity and binds the SBOM predicate to the crate's hash; the
checksum manifest alone does not authenticate any file. To authenticate
`sbom.cdx.json` content, the Python check above compares its parsed JSON with
the verified predicate, ignoring formatting. No online download is needed.

### Path B: crates.io and durable online attestation lookup

This path does not depend on either Actions artifacts or Release assets.
It uses the crates.io archive and GitHub's stored attestations. From a new
empty directory, set the independently checked `version` and `commit`, then:

```sh
set -eu
version=X.Y.Z
commit=REPLACE_WITH_REVIEWED_TAG_COMMIT_SHA
crate="irig106-types-$version.crate"
curl --fail --location "https://crates.io/api/v1/crates/irig106-types/$version/download" -o "$crate"
gh attestation trusted-root > trusted_root.jsonl
mkdir provenance-bundles sbom-bundles
(cd provenance-bundles && gh attestation download "../$crate" \
  -R TelemetryWorks/irig106-types --predicate-type https://slsa.dev/provenance/v1)
(cd sbom-bundles && gh attestation download "../$crate" \
  -R TelemetryWorks/irig106-types --predicate-type https://cyclonedx.org/bom)
gh attestation verify "$crate" \
  -R TelemetryWorks/irig106-types --bundle provenance-bundles/*.jsonl \
  --custom-trusted-root trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/trusted-publishing.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners
gh attestation verify "$crate" \
  -R TelemetryWorks/irig106-types --bundle sbom-bundles/*.jsonl \
  --custom-trusted-root trusted_root.jsonl \
  --signer-workflow TelemetryWorks/irig106-types/.github/workflows/trusted-publishing.yml \
  --source-ref "refs/tags/v$version" --source-digest "$commit" \
  --deny-self-hosted-runners --predicate-type https://cyclonedx.org/bom \
  --format json > verified-sbom.json
python3 - <<'PYBOM'
import json
from pathlib import Path
results = json.loads(Path('verified-sbom.json').read_text())
boms = [r['verificationResult']['statement']['predicate'] for r in results]
if not boms or any(b != boms[0] for b in boms):
    raise SystemExit('No verified SBOM or conflicting verified SBOM predicates')
if boms[0].get('bomFormat') != 'CycloneDX':
    raise SystemExit('Verified predicate is not a CycloneDX SBOM')
Path('sbom.cdx.json').write_text(json.dumps(boms[0], indent=2) + '\n')
PYBOM
```

`gh attestation download` writes digest-named `.jsonl` files; each new
subdirectory above contains one downloaded set for its predicate type.
Always extract the SBOM from **successful verification output**, not from
an unverified bundle's payload. The signer workflow, tag, commit, predicate,
and hosted-runner checks apply to both paths. Keep the crate, downloaded
bundles, trusted roots, and verified identity for later offline use. The
SBOM is authenticated as a predicate about the crate, not a separate
signature over the JSON file's bytes.

### Retention, gating, and recovery

Owner decision (2026-11-15): every release retains all five files as GitHub
Release assets. We use the **existing protected `crates-io` environment**
for the separate `release-assets` job, rather than a new environment whose
reviewers/tag restrictions might be missing. It needs only `publish`, which
forwards the verified commit/version. Its only permission is `contents: write`;
OIDC and attestations permissions are absent. The independently reviewed
workflow and remote tag-to-commit check control which tag it may update.
No environment or repository settings change is performed by this PR.

If asset retention fails after publication, the same-run Actions artifact
is a temporary fallback while retained:
`gh run download RUN_ID -R TelemetryWorks/irig106-types -n release-evidence -D evidence`.
Obtain the run ID independently from the reviewed tag's run. Path B remains
available independently of that artifact's expiry. Release assets persist
until deletion by an authorized user; disconnected sites should archive
both the files and their independently obtained trusted roots.

Retries add missing assets only. Identical existing files are retained;
a byte mismatch, including a different attestation bundle from a later run,
fails rather than replacing evidence. A failed initial upload leaves a draft;
updating an already published Release can leave a partial asset set until
recovery finishes. Inspect the Release and original run artifact before
owner-reviewed recovery. Never overwrite published evidence, move the tag,
bypass environment approval, or republish the crate to repair assets.
A failed attestation or asset step **after publication** does not undo an
upload already accepted by crates.io.

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

# Releasing irig106-types

Owner policy: crates.io Trusted Publishing (OIDC) from
**`.github/workflows/trusted-publishing.yml`**, named **Trusted publishing**.
The [organization standard](https://github.com/TelemetryWorks/.github/blob/fcceb02/docs/TRUSTED-PUBLISHING.md)
was adopted by this pilot on 2026-11-22. It supersedes the earlier pipeline's
post-publication signing and second `release-assets` environment approval.
Only **Publish to crates.io** uses the protected **`crates-io`** environment;
GitHub release retention follows automatically after successful publication.
Never store a registry token or run a real `cargo publish` locally. Dry runs
are permitted and never authenticate, sign, enter an environment, or publish.
No version is authorized for publication by merging this workflow adoption.

## Historical publication: 0.1.0 (2026-11-15)

Version **0.1.0 was published by attempt 2 from `.github/workflows/release.yml`**.
The [crates.io version record](https://crates.io/api/v1/crates/irig106-types/0.1.0)
identifies run **43727921675** and commit
`3c517358694646652939c236968b202ebf76e28a` in `trustpub_data`.
[Attempt 2](https://github.com/TelemetryWorks/irig106-types/actions/runs/43727921675/attempts/2)
published; [attempt 3](https://github.com/TelemetryWorks/irig106-types/actions/runs/43727921675/attempts/3)
was a later failed rerun because the version already existed. See the
[SECURITY-PROGRAM attempt history](https://github.com/TelemetryWorks/.github/blob/8c3fd1b50ada9195bddd810ecc3a83ad756ab27c/SECURITY-PROGRAM.md#verified-release-and-attempt-history).
That publication predates the new attestations. Do not re-release, re-tag,
replace or retroactively attest it. No `docs/releases/v0.1.0.md` is backfilled;
the committed-notes gate applies to the next authorized release.

## Owner configuration before the next tag

The owner must save, reopen and verify the crates.io Trusted Publisher:

| Field | Required value |
| --- | --- |
| Owner | `TelemetryWorks` |
| Repository | `irig106-types` |
| Workflow | `trusted-publishing.yml` |
| Environment | `crates-io` |

Owner decision dated **2026-11-15**: update the historical publisher filename
`release.yml` to `trusted-publishing.yml` and verify it **before the next tag**.
The private publisher registration is not verified by this PR. Retain
`crate.trustpub_only == true`; a filename/environment mismatch must block
publication, never trigger a stored-token fallback.

In GitHub, `crates-io` must require the owner reviewer, disallow administrator
bypass, and allow **`v*` tags only**, with no branch deployment rule. Verify
this protection explicitly: naming an absent environment can create an
unprotected one. This adoption does not change settings. Configure an active
`v*` tag ruleset restricting creation, updates and deletion to the owner;
never move or delete a release tag even when the owner can do so.

## Release preparation

1. Open a release-prep PR from main for an **unpublished** version. Update
   `Cargo.toml`, `Cargo.lock`, and any applicable exact pins together.
   This pilot supports one publishable root library. Adding a workspace must
   also adopt ordered family packaging/publication; the gate rejects an
   unexpected workspace rather than silently omitting crates.
2. Consolidate Unreleased changes into a dated
   `## [X.Y.Z] - YYYY-MM-DD` CHANGELOG entry. Set the intended tag-day date in
   this PR, correcting it before merge if the day changes. Commit
   `docs/releases/vX.Y.Z.md`: its first line is the release-title `# Heading`,
   followed by the exact CHANGELOG entry body, without its dated heading or
   trailing link definitions. The GitHub Release body is this committed file.
3. Pass the normal CI and the workflow's **pull_request dry run**. Its filters
   include workflows, release scripts/fixtures, manifests, packaged sources,
   tests, README, CHANGELOG, and release documentation. The gate reports
   release-readiness findings but emits `publish=false` on PR/manual runs,
   even when a manual dispatch is launched from a tag. Missing 0.1.0 notes
   during adoption are expected findings, not authorization to backfill them.
4. Obtain independent review and merge. Wait for the intended `ci.yml`
   workflow's **push-to-main run for the exact merged commit** to succeed.
   The tag gate checks all pages of that workflow's runs, exact SHA/path,
   event and branch; an unrelated green workflow does not qualify.
5. Complete the owner configuration above. From a clean checkout, inspect
   main, manifest, CHANGELOG and committed notes, then the owner creates and
   pushes one **annotated** `vX.Y.Z` tag on the reviewed main commit. For example,
   replacing the version below with the newly approved version:

   ```sh
   git switch main
   git pull --ff-only origin main
   git status --short
   git log -1 --oneline
   # Stop if dirty or if this is not the reviewed, CI-qualified commit.
   git tag -a vX.Y.Z -m "Release irig106-types X.Y.Z"
   git push origin refs/tags/vX.Y.Z
   ```

   Never substitute 0.1.0 or move an existing tag. The tag must exactly match
   the manifest version, be annotated and reachable from main, and have
   committed, matching release notes plus a valid dated CHANGELOG entry.

## Jobs and approval boundary

| Job | Name | Dependencies | Additional permission / environment |
| --- | --- | --- | --- |
| `gate` | Release gate | — | `actions: read` |
| `build` | Build | `gate` | — |
| `publish-path-check` | Publish path (dry run) | `gate`, `build` | read only; no environment |
| `attest` | Checksums and attestations | `gate`, `build` | OIDC + attestations write |
| `publish` | Publish to crates.io | `gate`, `build`, `attest` | OIDC; **`crates-io`** |
| `github-release` | GitHub release | `gate`, `publish` | `contents: write` only |
| `verify` | Verify publication | `gate`, `github-release` | read only |

Every job explicitly grants `contents: read`, except `github-release` replaces
it with write. Jobs attest/publish/github-release/verify run only when the
gate emits `publish=true` for a tag push. PR/manual routes run gate, build
and publish-path-check only. All base runners are Ubuntu 24.04. Actions are
SHA-pinned; checkout does not persist credentials. Two Cargo workers bound
CPU use, and job timeouts and network waits are bounded.

**Build** runs the full CI equivalent (three test configurations, MSRV 1.85.0
checks, clippy/fmt/rustdoc/wasm), release-control tests and `cargo package
--locked`, including compilation from the extracted package. Rust is pinned
to **1.99.1**, recorded as `build.outputs.rust-version`; downstream packaging
installs and checks that exact version. CycloneDX generator **0.5.9** is
installed with `--locked`. The SBOM describes the extracted source crate's
all-features/all-target dependency graph, not a consumer's compiled binary.

**Publish path (dry run)** strictly validates the candidate artifact checksums,
repackages using `cargo package --locked --no-verify`, and compares bytes with
the frozen build. It separately compares the registry's published **0.1.0**
with a committed expected archive, pinned at SHA-256
`2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420`.
See [fixture provenance](../scripts/release/fixtures/PROVENANCE.md). This is not
a comparison of two fresh downloads and is not signature verification;
dry-run candidates and the old fixture are unsigned.

**Attest** checks checksums before producing SLSA provenance for **both the
crate and SBOM file**. It finishes before publication approval. The standard's
new provenance layout supersedes the pilot's earlier SBOM-predicate bundles;
no release ever used that earlier pilot layout. No SLSA level or badge is
claimed. SHA256SUMS and the provenance bundle are evidence; neither is claimed
to have its own attestation.

**Before approving Publish to crates.io**, independently record/check the
run ID, annotated tag, exact commit, intended CI result and artifact digests.
Fetch current main in a trusted checkout and require the commit on main.
Identify the independently reviewed workflow by immutable main commit, then
compare its Git blob with the run commit's workflow blob:

```sh
set -eu
release_commit=REPLACE_WITH_RUN_COMMIT_SHA
reviewed_main_commit=REPLACE_WITH_REVIEWED_MAIN_COMMIT_SHA
git fetch origin main
git merge-base --is-ancestor "$release_commit" origin/main
git merge-base --is-ancestor "$reviewed_main_commit" origin/main
test "$(git rev-parse "$release_commit:.github/workflows/trusted-publishing.yml")" = \
     "$(git rev-parse "$reviewed_main_commit:.github/workflows/trusted-publishing.yml")"
```

Confirm both crate and SBOM attestations bind the frozen artifact digests to
that run/tag/commit, using its attestation step/summary or the constrained
verification below. A green gate alone never substitutes for independent
workflow identity review. Do not approve discrepancies or bypass protection.

**Nothing compiles in publish.** It downloads both the immutable payload
and provenance sidecar, validates SHA256SUMS, and runs `gh attestation verify
--bundle` on the crate and SBOM before any upload/authentication decision.
Both checks require the expected signer workflow, release tag and exact
`--source-digest`. It installs the build's exact Rust and reproduces the
archive with `--no-verify` before comparing bytes. Existing registry versions skip uploading only after checksum AND byte
comparison. A fresh OIDC action invocation supplies the single-crate wave's
short-lived token only to `cargo publish --locked --no-verify`. No stored
registry secret exists. New and existing served archives must equal the
frozen crate and registry checksum. The short-lived credential is revoked by
the auth action on job completion.

**GitHub release** has no registry credentials, OIDC, or approval environment.
It downloads and merges the same payload/sidecar and verifies both signatures
with the same signer/tag/commit constraints **before creating a Release or
attaching files**. It rechecks the tag/commit, uses the committed notes' first
heading as title and whole file as body, and creates/resumes a draft. Existing
drafts are resumed through the releases list. New drafts are created once via
the API, after verifying the existing annotated tag/commit, with the committed
notes and explicit draft/pre-release flags. The returned numeric `.id` is
retained and read through `GET /releases/{id}`; creation never depends on an
immediate post-create list match. Only a 404 on that ID lookup is retried:
up to five GETs, 30-second request timeouts, and 1/2/4/8-second backoff
(15 seconds total scheduled delay). Create is never retried. Exhaustion,
permission errors, invalid IDs, or mismatched tag/draft/title/body metadata
fail before attachment. Existing-draft discovery through the list does not
poll or sleep; the retained-ID reads use the bounded lookup rules above.

Missing assets may be added; existing assets must compare byte-for-byte and
are never clobbered.
A mismatched title/body or asset fails. After transfer it refreshes the Release
list and compares semantic versions, not upload time: the highest published
stable version (including the candidate) gets `make_latest: "true"`; older
stable versions and pre-releases get `"false"`. Pre-releases also set the
pre-release flag. Draft creation always uses latest=false because a draft
cannot be Latest; the policy is applied explicitly when publishing its metadata.
A second tag/commit check blocks un-drafting or metadata updates if the tag
moved. Immediately before the finalizing PATCH, re-fetch the retained numeric
Release ID and require the same ID, tag, title, and committed notes. A lookup
failure or changed metadata blocks finalization, even if earlier list/ID
checks and asset transfers succeeded. Metadata selection never permits asset
replacement.

**Verify publication** downloads both frozen artifacts as well as the actual
attached files, compares all four files byte-for-byte, validates all checksums,
compares the registry, and verifies both
attested files' identity, and waits at most ten docs.rs status observations.
If success is not observed, it reports **pending/unsuccessful and fails**;
that is not a passed documentation build. Only a library is shipped, so a
binary `cargo install` smoke test is not applicable.

## Durable evidence and verification

The immutable Actions payload is `release-X.Y.Z`: crate, `sbom.cdx.json` and
`SHA256SUMS`. Since an artifact cannot be appended to, `attest` uploads a
separate immutable `release-X.Y.Z-attestations` sidecar. Together they are
one logical release bundle. Later jobs consume the unchanged payload and
frozen sidecar; no artifact overwrite or substitute bytes is permitted.
Artifact selection uses `pattern: release-X.Y.Z{,-attestations}` with
`merge-multiple: true`, selecting exactly the two names rather than other
prefix matches. Build uploads an explicit three-file payload list and attest
uploads only the canonical bundle file, so their filenames cannot collide.
Both consumers verify the bundle before uploading or attaching; an absent or
invalid sidecar fails closed. Dry-run candidates remain unsigned and use only
the payload, without entering these tag-only verification/publication paths.

The permanent GitHub Release attaches these four files:

- `irig106-types-X.Y.Z.crate`
- `sbom.cdx.json`
- `SHA256SUMS` (exactly the two payload hashes)
- `irig106-types-X.Y.Z.crates.intoto.jsonl` (provenance for both payloads)

Release assets are not subject to Actions' 90-day artifact expiry, but remain
deletable by authorized users. Disconnected sites must archive the files,
independently fetched trusted roots, and the independently verified tag/commit.
Never accept a supplier's substituted trust configuration.

While online (replace the version/commit with independently reviewed values):

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

On the disconnected site, set `version` and `commit`, enter the saved evidence
directory, and verify using the saved roots in its parent:

```sh
set -eu
sha256sum -c SHA256SUMS
cmp "irig106-types-$version.crate" registry.crate
for file in "irig106-types-$version.crate" sbom.cdx.json; do
  gh attestation verify "$file" -R TelemetryWorks/irig106-types \
    --bundle "irig106-types-$version.crates.intoto.jsonl" \
    --custom-trusted-root ../trusted_root.jsonl \
    --signer-workflow TelemetryWorks/irig106-types/.github/workflows/trusted-publishing.yml \
    --source-ref "refs/tags/v$version" --source-digest "$commit" \
    --deny-self-hosted-runners --predicate-type https://slsa.dev/provenance/v1
done
```

This compares a previously fetched registry archive, not a live offline
registry. SHA256SUMS detects transfer errors; successful attestation checks
establish the files' identity. For online verification without local bundles,
run the same per-file `gh attestation verify` commands without `--bundle` and
`--custom-trusted-root` so GitHub supplies attestations. `gh attestation
download FILE -R TelemetryWorks/irig106-types` can retain digest-named bundles
for later offline use. The new SBOM is a directly attested file, not an SBOM
predicate extracted from the crate's SLSA provenance.

## Recovery and owner checklist

A failed job after upload does not undo a published crate. Check crates.io
before recovery, even if the run landing page says failure. Recovery under
this new pipeline skips only checksum/byte-matching versions; it never
replaces a version or changes the legacy 0.1.0 workflow's retry behavior.
Do not rerun a successful release merely to obtain a green display. Preserve
the original frozen artifacts/bundle when retrying missing Release assets;
a newly generated SBOM or bundle can differ and must not overwrite evidence.
Never force-push a tag, delete/replace assets, bypass review, store a fallback
token or publish locally to repair a partial failure.

Before tagging, the owner verifies the registry publisher's exact owner,
repository, filename and environment; only-mode; environment reviewer,
no-bypass and tag-only deployment protection; independently reviewed main,
CI and records; and the complete single-library release set. Read-only
checks during adoption do not certify the private publisher registration.


## Draft-discovery hardening (2026-11-23)

The [core release incident and fix](https://github.com/TelemetryWorks/irig106-core/pull/14)
exposed the base pipeline's one-shot list lookup after successful draft creation
(run 45118531760). Read-after-write visibility lag is consistent with that
failure; historical response bodies were not logged. Types shared that lookup.
The types follow-up retains the API creation response's ID and retries only
reading that specific ID, while preserving list-based resume. Logs retain the
new ID so an owner can identify a draft even if lookup exhausts its deadline.
An uncertain/failed create is never automatically repeated within the job.
The ID is job-local, not durable state shared with a rerun. In the rare case
that POST succeeds on GitHub but its response times out before the ID is
retained, a rerun relies on list-based resume. If that list is still stale,
the rerun can attempt a second create. Once duplicate matching Releases are
list-visible, resume fails closed with `Multiple Releases for the tag` and
requires manual cleanup; the pipeline does not choose one or delete drafts
automatically. This is a known cross-run recovery edge, not a guarantee that
a POST timeout means no draft was created. Inspect server state before any
owner-reviewed rerun. This change does not authorize a release rerun, tag
change, registry operation, or automated cleanup.

The [core parity follow-up](https://github.com/TelemetryWorks/irig106-core/pull/15)
also re-fetches by ID immediately before finalization. Types performs that
second read with its existing bounded getter and checks ID, tag, title and
committed notes; the expected notes are captured before transfer.

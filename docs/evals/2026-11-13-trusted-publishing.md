# Trusted Publishing readiness, 2026-11-13

Scope: preparation PR only. No settings, tags, publication, local credentials,
or OIDC token exchange. Owner authorization selects Trusted Publishing only;
the configured environment and publisher remain one-time owner actions.

## Verified external contracts

- Read-only crates.io API check for `irig106-types`: only version 0.0.1.
- GitHub upstream refs resolve checkout v6 to
  `d23441a48e516b6c34aea4fa41551a30e30af803` and crates-io-auth-action v1.0.6
  to `3268eb413133e2960fa4ae0e963ca76380c54cd9`.
- Inspected the pinned auth action metadata: token output, Node 24 entrypoint,
  and post-job cleanup; upstream README documents temporary-token revocation.
- Checked GitHub environment documentation: required-reviewer gate precedes
  job execution; tag rules are distinct from branch rules; an absent
  environment can be implicitly created without protection. Hence owner
  configuration must precede tag creation, as RELEASING.md requires.
- Checked Cargo documentation: publish dry run packages/verifies without
  uploading; include allowlists control archive contents. No dependency added.

Sources: [auth action at the pinned commit](https://github.com/rust-lang/crates-io-auth-action/tree/3268eb413133e2960fa4ae0e963ca76380c54cd9),
[crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing),
[GitHub environments](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments),
[Cargo publish](https://doc.rust-lang.org/cargo/commands/cargo-publish.html).

## Executed guard cases

Executed the Python guard extracted verbatim from release.yml with simulated
GitHub event metadata. The successful case used the actual checked-out main
commit, actual manifest, and actual Git ancestry. No tag was created.
The off-main case mocked Git's ancestry rejection; other Git operations ran
against this repository. Rejections produced no commit output for publication.

| Case | Result |
|---|---|
| Matching v0.1.0 tag metadata and real main ancestry | Accept |
| pull_request event | Reject |
| workflow_dispatch event | Reject |
| schedule event | Reject |
| Branch push | Reject |
| Version without v prefix | Reject |
| Wrong version v0.1.1 | Reject |
| Placeholder v0.0.1 | Reject |
| Incomplete v0.1 | Reject |
| Extra component v0.1.0.1 | Reject |
| Path suffix v0.1.0/extra | Reject |
| Shell metacharacters in tag | Reject |
| Empty tag name | Reject |
| Checkout differs from event commit | Reject |
| Unknown event commit | Reject |
| Off-main commit (Git rejection mocked) | Reject |

All 16 expectations passed. Static checks also confirmed no manual dispatch,
only one id-token permission grant, and no stored-secret references.
Actionlint v1.7.12 passed release.yml; its downloaded archive checksum was
verified against the upstream release checksums. No release action was run.

Actual environment enforcement and OIDC issuance cannot be exercised by this
preparation PR without the owner setup and an authorized release tag. This
record does not claim a live publication or an end-to-end authentication test.
The release procedure makes those remaining checks explicit for the owner.

Executed the full local CI equivalent on stable rustc 1.100.0 and MSRV
1.85.0 (x86_64), with two build workers. All ten commands passed: three
test configurations (41 unit + eight integration + one doctest each), three
MSRV checks, clippy with warnings denied, fmt, docs with warnings denied,
and no-default wasm32 build. Package listing and publish dry-run output
are reported on the PR after the clean preparation commit.

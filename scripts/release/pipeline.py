#!/usr/bin/env python3
"""Single-crate release controls. Dry-run commands never authenticate or publish."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request

NAME = "irig106-types"
REPO = "TelemetryWorks/irig106-types"
WORKFLOW = f"{REPO}/.github/workflows/trusted-publishing.yml"
KNOWN_VERSION = "0.1.0"
KNOWN_SHA256 = "2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420"
USER_AGENT = f"TelemetryWorks-release-pilot (https://github.com/{REPO})"


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs).strip()


def gh_api(path):
    return json.loads(run("gh", "api", path))


def output(**values):
    for key, value in values.items():
        print(f"{key}={value}")
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as stream:
            for key, value in values.items():
                stream.write(f"{key}={value}\n")


def summary(message):
    print(message)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as stream:
            stream.write(message + "\n")


def version():
    data = tomllib.loads(Path("Cargo.toml").read_text())
    package = data["package"]
    if "workspace" in data or package["name"] != NAME or package.get("publish") != ["crates-io"]:
        raise ValueError("Single-crate pilot: review release-set ordering before changing workspace/publish metadata")
    value = package["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", value):
        raise ValueError("Invalid package version")
    return value


def release_record(value):
    text = Path("CHANGELOG.md").read_text()
    match = re.search(rf"^## \[{re.escape(value)}\] - (\d{{4}}-\d{{2}}-\d{{2}})\s*$", text, re.M)
    if not match:
        raise ValueError(f"Missing dated CHANGELOG entry for {value}")
    datetime.date.fromisoformat(match[1])
    body = re.split(r"^## ", text[match.end():], maxsplit=1, flags=re.M)[0].strip()
    # Link definitions at the end of the changelog are not release notes.
    body = re.split(r"^\[[^\]]+\]:", body, maxsplit=1, flags=re.M)[0].strip()
    if not body:
        raise ValueError("Empty release changelog entry")
    path = Path(f"docs/releases/v{value}.md")
    notes = path.read_text()
    title, _, notes_body = notes.partition("\n")
    if not title.startswith("# ") or not title[2:].strip() or notes_body.strip() != body:
        raise ValueError("Release notes need a first heading and the exact CHANGELOG entry body")
    if run("git", "show", f"HEAD:{path}") != notes.strip():
        raise ValueError("Release notes must be committed at the release commit")
    return path, title[2:].strip()


def ci_succeeded(pages, commit):
    return any(
        item.get("head_sha") == commit
        and item.get("path") == ".github/workflows/ci.yml"
        and item.get("conclusion") == "success"
        and item.get("event") == "push"
        and item.get("head_branch") == "main"
        for page in pages for item in page["workflow_runs"]
    )


def tag_commit(tag):
    obj = gh_api(f"repos/{REPO}/git/ref/tags/{tag}")["object"]
    if obj["type"] != "tag":
        raise ValueError("Remote release tag must be annotated")
    for _ in range(8):
        if obj["type"] != "tag":
            break
        obj = gh_api(f"repos/{REPO}/git/tags/{obj['sha']}")["object"]
    if obj["type"] != "commit":
        raise ValueError("Tag does not resolve to a commit")
    return obj["sha"]


def gate():
    value = version()
    commit = run("git", "rev-parse", "HEAD")
    event = os.environ["GITHUB_EVENT_NAME"]
    publishing = event == "push" and os.environ.get("GITHUB_REF", "").startswith("refs/tags/v")
    if event not in ("push", "pull_request", "workflow_dispatch") or (event == "push" and not publishing):
        raise ValueError("Unexpected release event")
    findings = []
    try:
        release_record(value)
    except (ValueError, FileNotFoundError, subprocess.CalledProcessError) as error:
        findings.append(str(error))
    if publishing:
        tag = os.environ["GITHUB_REF_NAME"]
        if tag != f"v{value}":
            findings.append("Tag must exactly match the manifest version")
        else:
            if run("git", "cat-file", "-t", f"refs/tags/{tag}") != "tag":
                findings.append("Release tag must be annotated")
            if run("git", "rev-parse", f"refs/tags/{tag}^{{commit}}") != commit:
                findings.append("Checkout must equal the tagged commit")
            try:
                if tag_commit(tag) != commit:
                    findings.append("Remote annotated tag does not match the release commit")
            except ValueError as error:
                findings.append(str(error))
        if run("git", "rev-parse", os.environ["GITHUB_SHA"] + "^{commit}") != commit:
            findings.append("Checkout differs from the event commit")
        if subprocess.run(["git", "merge-base", "--is-ancestor", commit, "origin/main"]).returncode:
            findings.append("Release commit is not on main")
        pages = json.loads(run(
            "gh", "api", "--paginate", "--slurp",
            f"repos/{REPO}/actions/workflows/ci.yml/runs?head_sha={commit}&per_page=100",
        ))
        if not ci_succeeded(pages, commit):
            findings.append("No successful push-to-main CI workflow run for this exact commit")
    else:
        summary("Dry run: publish=false; tag/ancestry/CI requirements are enforced only on tag pushes.")
    for finding in findings:
        summary(f"- Release gate finding: {finding}")
    if publishing and findings:
        raise ValueError("Release gate rejected the tag before build")
    output(version=value, commit=commit, publish=str(publishing).lower(),
           crates=json.dumps([{"name": NAME, "version": value, "manifest": "Cargo.toml"}], separators=(",", ":")))


def expected_files(value):
    return {f"{NAME}-{value}.crate", "sbom.cdx.json"}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def validate_bundle(directory, value):
    root = Path(directory)
    manifest = root / "SHA256SUMS"
    if manifest.is_symlink():
        raise ValueError("Symlink checksum manifest")
    checksums = {}
    for line in manifest.read_text().splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.+-]+)", line)
        if not match or match[2] in checksums:
            raise ValueError("Malformed, unsafe or duplicate checksum entry")
        checksums[match[2]] = match[1]
    if set(checksums) != expected_files(value):
        raise ValueError("Checksum manifest must cover exactly the crate and SBOM")
    for name, checksum in checksums.items():
        path = root / name
        if path.is_symlink() or not path.is_file() or digest(path) != checksum:
            raise ValueError(f"Checksum/file mismatch: {name}")
    allowed = expected_files(value) | {"SHA256SUMS", f"{NAME}-{value}.crates.intoto.jsonl"}
    if {p.name for p in root.iterdir()} - allowed:
        raise ValueError("Unexpected release artifact files")
    return checksums


def http_get(url):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return None
            if error.code != 429 and error.code < 500:
                raise
            if attempt == 3:
                raise
        except urllib.error.URLError:
            if attempt == 3:
                raise
        time.sleep(2)
    raise RuntimeError("HTTP retry exhausted")


def registry_metadata(value):
    raw = http_get(f"https://crates.io/api/v1/crates/{NAME}/{value}")
    if raw is None:
        return None
    data = json.loads(raw)["version"]
    if data["num"] != value or not re.fullmatch(r"[0-9a-f]{64}", data["checksum"]):
        raise ValueError("Invalid registry metadata")
    return data


def registry_compare(value, archive):
    metadata = registry_metadata(value)
    if metadata is None:
        raise ValueError(f"Registry version not available: {value}")
    raw = http_get(f"https://crates.io/api/v1/crates/{NAME}/{value}/download")
    if raw is None or hashlib.sha256(raw).hexdigest() != metadata["checksum"]:
        raise ValueError("Registry archive does not match registry checksum")
    if raw != Path(archive).read_bytes():
        raise ValueError("Registry archive is not byte-identical to the frozen crate")
    summary(f"Registry byte identity and checksum passed: {NAME} {value}, sha256:{metadata['checksum']}")


def check_rust(toolchain):
    if not re.fullmatch(r"\d+\.\d+\.\d+", toolchain):
        raise ValueError("Expected exact Rust toolchain version")
    actual = run("rustc", f"+{toolchain}", "--version").split()[1]
    if actual != toolchain:
        raise ValueError(f"Expected Rust {toolchain}, got {actual}")


def repackage(directory, value, toolchain):
    validate_bundle(directory, value)
    check_rust(toolchain)
    subprocess.run(["cargo", f"+{toolchain}", "package", "--locked", "--no-verify"], check=True)
    candidate = Path(f"target/package/{NAME}-{value}.crate")
    if candidate.read_bytes() != (Path(directory) / candidate.name).read_bytes():
        raise ValueError("Repackaged crate differs from frozen build artifact")
    summary(f"Non-compiling candidate byte comparison passed with Rust {toolchain}")


def known_good():
    fixture = Path(__file__).parent / "fixtures" / f"{NAME}-{KNOWN_VERSION}.crate"
    if digest(fixture) != KNOWN_SHA256:
        raise ValueError("Pinned known-good fixture digest mismatch")
    registry_compare(KNOWN_VERSION, fixture)
    summary("Known-good check uses a committed fixture; candidate dry-run artifacts are unsigned.")


def verify_attestations(directory, value):
    root = Path(directory)
    validate_bundle(root, value)
    bundle = root / f"{NAME}-{value}.crates.intoto.jsonl"
    if bundle.is_symlink() or not bundle.is_file() or not bundle.stat().st_size:
        raise ValueError("Missing frozen provenance bundle")
    for name in sorted(expected_files(value)):
        subprocess.run(["gh", "attestation", "verify", str(root / name), "--repo", REPO,
                        "--bundle", str(bundle), "--signer-workflow", WORKFLOW,
                        "--source-ref", f"refs/tags/v{value}", "--source-digest", os.environ["RELEASE_COMMIT"],
                        "--deny-self-hosted-runners", "--predicate-type", "https://slsa.dev/provenance/v1"], check=True)


def release_list():
    pages = json.loads(run("gh", "api", "--paginate", "--slurp", f"repos/{REPO}/releases?per_page=100"))
    return [item for page in pages for item in page]


def release_by_id(release_id):
    """Poll only a newly created ID; never repeat a create during visibility lag."""
    if type(release_id) is not int or release_id <= 0:
        raise ValueError("Create response did not contain a valid release id")
    for attempt in range(5):
        try:
            release = json.loads(run("gh", "api", f"repos/{REPO}/releases/{release_id}",
                                     stderr=subprocess.PIPE, timeout=30))
        except subprocess.CalledProcessError as error:
            if not re.search(r"\bHTTP 404\b", str(error.stderr or "")):
                raise
            if attempt == 4:
                raise ValueError(f"Created draft {release_id} unavailable after 5 ID lookups") from error
            time.sleep(2 ** attempt)
            continue
        if not isinstance(release, dict) or type(release.get("id")) is not int or release["id"] != release_id:
            raise ValueError("Release lookup returned a different id")
        return release


def semantic_version(value):
    match = re.fullmatch(r"v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?", value)
    if not match:
        return None
    return tuple(int(match[i]) for i in (1, 2, 3)), match[4] is not None


def latest_policy(value, releases):
    """Compare SemVer, never upload time; unpublished drafts are not Latest."""
    target = semantic_version(value)
    if target is None:
        raise ValueError("Invalid semantic release version")
    core, prerelease = target
    stable_versions = [core]
    for release in releases:
        parsed = semantic_version(release["tag_name"])
        if parsed and not parsed[1] and not release.get("draft") and not release.get("prerelease"):
            stable_versions.append(parsed[0])
    return prerelease, not prerelease and core == max(stable_versions)


def prepare(directory, value, toolchain):
    verify_attestations(directory, value)
    repackage(directory, value, toolchain)
    existing = registry_metadata(value)
    if existing is not None:
        registry_compare(value, Path(directory) / f"{NAME}-{value}.crate")
    output(upload=str(existing is None).lower())


def registry_wait(directory, value):
    for attempt in range(10):
        if registry_metadata(value) is not None:
            registry_compare(value, Path(directory) / f"{NAME}-{value}.crate")
            return
        if attempt < 9:
            time.sleep(3)
    raise ValueError("Published crate did not appear within the bounded registry wait")


def release_assets(directory, value):
    root = Path(directory).resolve()
    verify_attestations(root, value)
    bundle = root / f"{NAME}-{value}.crates.intoto.jsonl"
    tag = f"v{value}"
    if tag_commit(tag) != os.environ["RELEASE_COMMIT"]:
        raise ValueError("Remote tag moved from the gate-verified commit")
    notes, title = release_record(value)
    # The list endpoint includes drafts that a tag lookup may not expose.
    matches = [item for item in release_list() if item["tag_name"] == tag]
    if len(matches) > 1:
        raise ValueError("Multiple Releases for the tag")
    if matches:
        release = matches[0]
    else:
        prerelease, _ = latest_policy(value, [])
        # The annotated remote tag/commit was verified above; creation never
        # retries and cannot rely on immediate visibility through the list.
        created = json.loads(run("gh", "api", "--method", "POST", f"repos/{REPO}/releases", "--input", "-",
                                 input=json.dumps({"tag_name": tag, "target_commitish": os.environ["RELEASE_COMMIT"],
                                                   "name": title, "body": notes.read_text(), "draft": True,
                                                   "prerelease": prerelease, "make_latest": "false"})))
        release_id = created.get("id") if isinstance(created, dict) else None
        if type(release_id) is not int or release_id <= 0:
            raise ValueError("Create response did not contain a valid release id")
        summary(f"Created draft {release_id}; reading by ID without list rediscovery")
        release = release_by_id(release_id)
        if release.get("tag_name") != tag or release.get("draft") is not True:
            raise ValueError("Created release tag/draft state differs from the request")
    if release["name"] != title or (release.get("body") or "").replace("\r\n", "\n").strip() != notes.read_text().strip():
        raise ValueError("Release metadata differs from committed notes")
    names = expected_files(value) | {"SHA256SUMS", bundle.name}
    assets = {asset["name"]: asset for asset in release["assets"]}
    if len(assets) != len(release["assets"]) or set(assets) - names:
        raise ValueError("Duplicate/unexpected existing Release assets")
    for name in sorted(names):
        if name in assets:
            raw = subprocess.check_output(["gh", "api", f"repos/{REPO}/releases/assets/{assets[name]['id']}",
                                           "-H", "Accept: application/octet-stream"])
            if raw != (root / name).read_bytes():
                raise ValueError(f"Existing Release asset differs: {name}")
        else:
            # POST without deleting/replacing an asset: name conflicts fail closed.
            subprocess.run(["gh", "api", "--method", "POST",
                            f"https://uploads.github.com/repos/{REPO}/releases/{release['id']}/assets?name={urllib.parse.quote(name, safe='')}",
                            "-H", "Content-Type: application/octet-stream", "--input", str(root / name)],
                           check=True, stdout=subprocess.DEVNULL)
    # Refresh after asset transfer: an older late upload must not take Latest.
    prerelease, latest = latest_policy(value, release_list())
    if tag_commit(tag) != os.environ["RELEASE_COMMIT"]:
        raise ValueError("Remote tag moved before publishing the release metadata")
    subprocess.run(["gh", "api", "--method", "PATCH", f"repos/{REPO}/releases/{release['id']}",
                    "--input", "-"], input=json.dumps({"draft": False, "prerelease": prerelease,
                                                     "make_latest": str(latest).lower()}),
                   text=True, check=True, stdout=subprocess.DEVNULL)


def verify_publication(directory, value):
    root = Path(directory)
    verify_attestations(root, value)
    registry_compare(value, root / f"{NAME}-{value}.crate")
    for attempt in range(10):
        raw = http_get(f"https://docs.rs/crate/{NAME}/{value}/status.json")
        if raw:
            data = json.loads(raw)
            if data.get("version") == value and data.get("doc_status") is True:
                summary(f"docs.rs {value}: passed (doc_status=true)")
                return
        if attempt < 9:
            time.sleep(10)
    summary(f"docs.rs {value}: pending/unsuccessful; no successful build observed before deadline")
    raise ValueError("docs.rs success is required; pending is not passed")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["gate", "validate", "repackage", "known-good", "prepare", "registry", "github-release", "verify"])
    parser.add_argument("--directory", default="release-artifacts")
    parser.add_argument("--version")
    parser.add_argument("--rust-version")
    args = parser.parse_args()
    if args.command == "gate":
        gate()
    elif args.command == "known-good":
        known_good()
    else:
        value = version()
        if value != args.version:
            raise ValueError("Gate version differs from checked-out manifest")
        if args.command == "validate":
            validate_bundle(args.directory, value)
        elif args.command == "repackage":
            repackage(args.directory, value, args.rust_version)
        elif args.command == "prepare":
            prepare(args.directory, value, args.rust_version)
        elif args.command == "registry":
            registry_wait(args.directory, value)
        elif args.command == "github-release":
            release_assets(args.directory, value)
        elif args.command == "verify":
            verify_publication(args.directory, value)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, FileNotFoundError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        print(f"Release control failed: {error}", file=sys.stderr)
        sys.exit(1)

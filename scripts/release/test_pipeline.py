"""Release controls: reject invalid evidence without registry mutations."""

import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("pipeline", Path(__file__).with_name("pipeline.py"))
pipeline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pipeline)


class ReleaseControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.previous = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, self.previous)
        self.addCleanup(self.temp.cleanup)
        self.env = patch.dict(os.environ, {"GITHUB_OUTPUT": "", "GITHUB_STEP_SUMMARY": ""})
        self.env.start()
        self.addCleanup(self.env.stop)
        self.value = "0.2.0"
        self.crate = f"irig106-types-{self.value}.crate"
        self.artifacts = self.root / "artifacts"
        self.artifacts.mkdir()
        (self.artifacts / self.crate).write_bytes(b"frozen crate")
        (self.artifacts / "sbom.cdx.json").write_text('{"bomFormat":"CycloneDX"}')
        self.manifest()

    def manifest(self):
        self.lines = [f"{pipeline.digest(self.artifacts / name)}  {name}" for name in sorted(pipeline.expected_files(self.value))]
        (self.artifacts / "SHA256SUMS").write_text("\n".join(self.lines) + "\n")

    def test_valid_manifest(self):
        self.assertEqual(set(pipeline.validate_bundle(self.artifacts, self.value)), {self.crate, "sbom.cdx.json"})

    def test_checksum_mutation_rejected(self):
        (self.artifacts / self.crate).write_bytes(b"changed")
        with self.assertRaises(ValueError):
            pipeline.validate_bundle(self.artifacts, self.value)

    def test_manifest_adversarial_entries(self):
        for bad in [self.lines[0], "\n".join(self.lines + [self.lines[0]]),
                    self.lines[0].replace(self.crate, "../escape") + "\n" + self.lines[1],
                    "g" * 64 + "  " + self.crate, "\n".join(self.lines) + "\n\n"]:
            with self.subTest(bad=bad):
                (self.artifacts / "SHA256SUMS").write_text(bad)
                with self.assertRaises(ValueError):
                    pipeline.validate_bundle(self.artifacts, self.value)

    def test_missing_file_rejected(self):
        (self.artifacts / "sbom.cdx.json").unlink()
        with self.assertRaises(ValueError):
            pipeline.validate_bundle(self.artifacts, self.value)

    def test_symlink_payload_rejected(self):
        outside = self.root / "outside"
        (self.artifacts / self.crate).rename(outside)
        (self.artifacts / self.crate).symlink_to(outside)
        with self.assertRaises(ValueError):
            pipeline.validate_bundle(self.artifacts, self.value)

    def test_unexpected_payload_rejected(self):
        (self.artifacts / "extra.crate").write_bytes(b"extra")
        with self.assertRaises(ValueError):
            pipeline.validate_bundle(self.artifacts, self.value)

    def test_manifest_symlink_rejected(self):
        path = self.artifacts / "SHA256SUMS"
        path.rename(self.root / "manifest")
        path.symlink_to(self.root / "manifest")
        with self.assertRaises(ValueError):
            pipeline.validate_bundle(self.artifacts, self.value)

    def make_record(self, date="2026-11-22", notes="Fixed a release control."):
        Path("CHANGELOG.md").write_text(f"## [{self.value}] - {date}\n\nFixed a release control.\n\n[0.2.0]: https://example.test\n")
        path = Path(f"docs/releases/v{self.value}.md")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"# irig106-types {self.value}\n\n{notes}\n")
        return path

    def test_committed_dated_notes(self):
        path = self.make_record()
        with patch.object(pipeline, "run", return_value=path.read_text().strip()):
            self.assertEqual(pipeline.release_record(self.value), (path, f"irig106-types {self.value}"))

    def test_missing_notes(self):
        self.make_record().unlink()
        with self.assertRaises(FileNotFoundError):
            pipeline.release_record(self.value)

    def test_invalid_changelog_date(self):
        self.make_record(date="2026-02-30")
        with self.assertRaises(ValueError):
            pipeline.release_record(self.value)

    def test_notes_must_derive_from_changelog(self):
        self.make_record(notes="Unrelated notes")
        with self.assertRaises(ValueError):
            pipeline.release_record(self.value)

    def test_notes_must_be_committed(self):
        self.make_record()
        with patch.object(pipeline, "run", return_value="different committed body"):
            with self.assertRaises(ValueError):
                pipeline.release_record(self.value)

    def test_ci_pages_and_identity(self):
        good = {"head_sha": "a" * 40, "path": ".github/workflows/ci.yml", "conclusion": "success", "event": "push", "head_branch": "main"}
        pages = [{"workflow_runs": []}, {"workflow_runs": [good]}]
        self.assertTrue(pipeline.ci_succeeded(pages, "a" * 40))
        for key, value in [("head_sha", "b" * 40), ("path", "unrelated.yml"), ("conclusion", "failure"), ("event", "pull_request"), ("head_branch", "other")]:
            with self.subTest(key=key):
                self.assertFalse(pipeline.ci_succeeded([{"workflow_runs": [dict(good, **{key: value})]}], "a" * 40))

    def test_dry_run_routes_never_publish_even_from_tag(self):
        for event in ["pull_request", "workflow_dispatch"]:
            with self.subTest(event=event), patch.dict(os.environ, {"GITHUB_EVENT_NAME": event, "GITHUB_REF": "refs/tags/v0.2.0"}), \
                 patch.object(pipeline, "version", return_value=self.value), patch.object(pipeline, "run", return_value="a" * 40), \
                 patch.object(pipeline, "release_record", side_effect=FileNotFoundError("missing future notes")), \
                 patch.object(pipeline, "output") as out, contextlib.redirect_stdout(io.StringIO()):
                pipeline.gate()
                self.assertEqual(out.call_args.kwargs["publish"], "false")

    def tag_run(self, kind="tag", tag="v0.2.0", ci=True, ancestry=0, notes=True):
        def fake_run(*args, **kwargs):
            if args[0] == "gh":
                return json.dumps([{"workflow_runs": [{"head_sha": "a" * 40, "path": ".github/workflows/ci.yml", "conclusion": "success" if ci else "failure", "event": "push", "head_branch": "main"}]}])
            if args[1] == "cat-file":
                return kind
            return "a" * 40
        with patch.dict(os.environ, {"GITHUB_EVENT_NAME": "push", "GITHUB_REF": "refs/tags/" + tag, "GITHUB_REF_NAME": tag, "GITHUB_SHA": "a" * 40}), \
             patch.object(pipeline, "version", return_value=self.value), patch.object(pipeline, "run", side_effect=fake_run), \
             patch.object(pipeline.subprocess, "run", return_value=subprocess.CompletedProcess([], ancestry)), \
             patch.object(pipeline, "release_record", side_effect=None if notes else FileNotFoundError("missing notes")), \
             patch.object(pipeline, "output") as out, contextlib.redirect_stdout(io.StringIO()):
            pipeline.gate()
            self.assertEqual(out.call_args.kwargs["publish"], "true")

    def test_qualified_tag_gate(self):
        self.tag_run()

    def test_tag_gate_rejections(self):
        for options in [{"kind": "commit"}, {"tag": "v0.3.0"}, {"ci": False}, {"ancestry": 1}, {"notes": False}]:
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.tag_run(**options)

    def test_unexpected_event_rejected(self):
        with patch.dict(os.environ, {"GITHUB_EVENT_NAME": "schedule"}), patch.object(pipeline, "version", return_value=self.value), patch.object(pipeline, "run", return_value="a" * 40):
            with self.assertRaises(ValueError):
                pipeline.gate()

    def test_existing_registry_mismatch_is_not_skipped(self):
        with patch.object(pipeline, "registry_metadata", return_value={"checksum": "a" * 64}), patch.object(pipeline, "http_get", return_value=b"different"):
            with self.assertRaises(ValueError):
                pipeline.registry_compare(self.value, self.artifacts / self.crate)

    def test_registry_byte_identity_and_checksum(self):
        raw = (self.artifacts / self.crate).read_bytes()
        with patch.object(pipeline, "registry_metadata", return_value={"checksum": hashlib.sha256(raw).hexdigest()}), patch.object(pipeline, "http_get", return_value=raw), contextlib.redirect_stdout(io.StringIO()):
            pipeline.registry_compare(self.value, self.artifacts / self.crate)

    def test_repackage_never_compiles(self):
        candidate = Path(f"target/package/{self.crate}")
        candidate.parent.mkdir(parents=True)
        candidate.write_bytes((self.artifacts / self.crate).read_bytes())
        with patch.object(pipeline, "check_rust"), patch.object(pipeline.subprocess, "run") as call, contextlib.redirect_stdout(io.StringIO()):
            pipeline.repackage(self.artifacts, self.value, "1.99.1")
            self.assertEqual(call.call_args.args[0], ["cargo", "+1.99.1", "package", "--locked", "--no-verify"])
        candidate.write_bytes(b"different candidate")
        with patch.object(pipeline, "check_rust"), patch.object(pipeline.subprocess, "run"):
            with self.assertRaises(ValueError):
                pipeline.repackage(self.artifacts, self.value, "1.99.1")

    def test_prepare_skips_only_matching_existing_registry_bytes(self):
        with patch.object(pipeline, "repackage"), patch.object(pipeline, "registry_metadata", return_value={}), patch.object(pipeline, "registry_compare") as comparison, patch.object(pipeline, "output") as out:
            pipeline.prepare(self.artifacts, self.value, "1.99.1")
            comparison.assert_called_once()
            self.assertEqual(out.call_args.kwargs, {"upload": "false"})

    def test_wrong_exact_rust_version_rejected(self):
        with patch.object(pipeline, "run", return_value="rustc 1.100.0 (wrong)"):
            with self.assertRaises(ValueError):
                pipeline.check_rust("1.99.1")
        with self.assertRaises(ValueError):
            pipeline.check_rust("stable")

    def test_known_good_fixture_is_pinned(self):
        fixture = Path(pipeline.__file__).parent / "fixtures/irig106-types-0.1.0.crate"
        self.assertEqual(pipeline.digest(fixture), "2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420")

    def release_case(self, *, moved=False, wrong_body=False, mismatch=False, missing=False, draft=False, new=False):
        bundle = f"irig106-types-{self.value}.crates.intoto.jsonl"
        (self.artifacts / bundle).write_text('{}\n')
        path = self.make_record()
        names = sorted(pipeline.expected_files(self.value) | {"SHA256SUMS", bundle})
        metadata = {"id": 10, "tag_name": f"v{self.value}", "name": f"irig106-types {self.value}",
                    "body": "wrong" if wrong_body else path.read_text(), "draft": draft or new,
                    "assets": [] if missing else [{"name": name, "id": i} for i, name in enumerate(names)]}
        reads = []
        def fake_run(*args, **kwargs):
            if args[0] == "gh":
                reads.append(True)
                return json.dumps([[]] if new and len(reads) == 1 else [[metadata]])
            return path.read_text().strip()
        def asset_bytes(args, **kwargs):
            index = int(args[2].rsplit("/", 1)[1])
            return b"changed" if mismatch else (self.artifacts / names[index]).read_bytes()
        with patch.dict(os.environ, {"RELEASE_COMMIT": "a" * 40}), \
             patch.object(pipeline, "tag_commit", return_value=("b" if moved else "a") * 40), \
             patch.object(pipeline, "run", side_effect=fake_run), \
             patch.object(pipeline.subprocess, "check_output", side_effect=asset_bytes), \
             patch.object(pipeline.subprocess, "run") as mutations:
            pipeline.release_assets(self.artifacts, self.value)
            return mutations.call_args_list

    def test_new_release_requires_existing_tag_and_committed_notes(self):
        calls = self.release_case(new=True)
        command = calls[0].args[0]
        self.assertEqual(command[:3], ["gh", "release", "create"])
        self.assertIn("--verify-tag", command)
        self.assertIn("--notes-file", command)
        self.assertIn("--draft", command)
        self.assertIn("PATCH", calls[-1].args[0])

    def test_single_crate_manifest_scope(self):
        Path("Cargo.toml").write_text('[package]\nname="irig106-types"\nversion="0.2.0"\npublish=["crates-io"]\n')
        self.assertEqual(pipeline.version(), self.value)
        for extra in ['\n[workspace]\nmembers=[]\n', '']:
            text = '[package]\nname="irig106-types"\nversion="0.2.0"\npublish=false\n' + extra
            Path("Cargo.toml").write_text(text)
            with self.assertRaises(ValueError):
                pipeline.version()

    def test_existing_release_assets_retained_without_writes(self):
        self.assertEqual(self.release_case(), [])

    def test_existing_asset_mismatch_rejected(self):
        with self.assertRaises(ValueError):
            self.release_case(mismatch=True)

    def test_existing_release_notes_mismatch_rejected(self):
        with self.assertRaises(ValueError):
            self.release_case(wrong_body=True)

    def test_moved_remote_tag_rejected(self):
        with self.assertRaises(ValueError):
            self.release_case(moved=True)

    def test_missing_release_assets_uploaded_without_clobber(self):
        calls = self.release_case(missing=True)
        self.assertEqual(len(calls), 4)
        self.assertTrue(all("POST" in call.args[0] and "--clobber" not in call.args[0] for call in calls))

    def test_resumed_draft_uses_numeric_api(self):
        calls = self.release_case(draft=True)
        self.assertEqual(len(calls), 1)
        self.assertIn("PATCH", calls[0].args[0])
        self.assertIn("releases/10", calls[0].args[0][-3])

    def test_docsrs_pending_never_passes(self):
        (self.artifacts / f"irig106-types-{self.value}.crates.intoto.jsonl").write_text('{}\n')
        with patch.object(pipeline, "registry_compare"), patch.object(pipeline.subprocess, "run"), \
             patch.object(pipeline, "http_get", return_value=b'{"version":"0.2.0","doc_status":false}'), \
             patch.object(pipeline.time, "sleep"), patch.dict(os.environ, {"RELEASE_COMMIT": "a" * 40}), contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(ValueError, "pending is not passed"):
                pipeline.verify_publication(self.artifacts, self.value)


if __name__ == "__main__":
    unittest.main()

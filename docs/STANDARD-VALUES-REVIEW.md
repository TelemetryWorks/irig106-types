# Shared standard values: T-5, T-6, T-7, T-9

Baseline inspected: origin/main `9f93dd5ebc1c2c29d6ba41ed85f4f8f61093adf9`.
The checkout was clean before branching. Version stays 0.1.0; no release is made.

## Authority and source inspection

Read `irig106-time`'s contract sections 8.2, 8.3, 8.6, ADR-0015,
ADR-0016, and STANDARD-REVIEW T-5/T-6/T-7/T-9. No time source files changed.

PDFs came from `TelemetryWorks/rcc-106-standards` at
`10719995d63426c2fe62d1548acb21ae0238b44c`, using its `scripts/fetch.py`
(with `python3`, since this host has no `python` alias). The script verified
size and SHA-256 against the manifest. Selected downloads:

```sh
python3 scripts/fetch.py --edition 106-24R1 --chapter 11
python3 scripts/fetch.py --edition 106-05 --edition 106-07 --edition 106-13 --edition 106-15 --chapter 10
python3 scripts/fetch.py --edition 106-17 --edition 106-22 --edition 106-23 --edition 106-24 --chapter 11
```

Inspected the extracted text, including the figure layouts and adjacent code
tables. The test fixtures are hand-transcribed/hand-calculated from those
sources, never produced by the crate's encoders. PDFs and full extracted
standard text are not committed.

| Finding | Primary evidence | Fixture derivation |
|---|---|---|
| T-5 | 106-24R1 §11.2.1.1 g, Figure 11-6, pp. 11-9–10 | 1 GHz gives 1 ns/tick; LSLW precedes MSLW. Explicit LE fixtures for 0, 1, 100, 2^32, 2^64−1; 100 ERTC ticks and 1 RTC tick both give 100 ns. |
| T-6 | §11.2.3.2 a, Figure 11-12, pp. 11-23–25 | SRC 0 internal, 1 external, 2 RMM, 3–14 reserved, 15 none. Same table in 106-05 Figure 10-16a, 106-07 Figure 10-22, 106-13/15 Figure 10-19 (§10.6.3.2). |
| T-7 | §11.2.3.2 a, Figure 11-12, pp. 11-23–25 | FMT 15 NONE; ITS 0–7 enumerated explicitly, 8–15 reserved. 106-15 Figure 10-19 reserves bits 31–12; 106-17 §11.2.3.2 defines ITS. |
| T-9 | §11.2.7.2, Figure 11-34, pp. 11-50–51; §11.2.1.1 e, Table 11-4, p. 11-6 | Two explicit edition lists and every byte value tested. Setup 7–14 vs header 1–10. All other bytes unknown. 106-07 §10.6.7.2/Figure 10-50 reserves setup 0–6 too. 106-22/23/24/24R1 keep setup 14 = 106-22. |

## Before/after executed evidence

Tests-only commit `d644de3` is based on current main with no library changes.
Each command was executed separately before implementation:

| Command (`cargo test --test …`) | Before on main's implementation | After |
|---|---|---|
| `t5_ertc` | FAIL: left `100`, right `1` | PASS |
| `t6_source` | FAIL: left `Gps`, right `Reserved(3)` | PASS |
| `t7_none` | FAIL: left `"Reserved(15)"`, right `"None"` | PASS |
| `t7_its` | Compile failure E0432: no `IrigTimeSource` in root | PASS |
| `t9_versions` | Compile failure E0432: no `Irig106Version` in root (and consequent unresolved variants) | PASS |

The last two are **missing-API compile failures**, not executed assertion
failures. They cannot run against main because main has neither API.
The NONE test uses the decoded Debug name so the identical assertion can
execute before the variant exists. Existing unit tests also assert the
new variant directly. Old ERTC/source unit assertions were corrected to the
standard after capturing the failures above.

## Every public API change

- `Ertc::to_nanos(self) -> u128` retains its signature, but now returns the
  raw tick count as nanoseconds (previously multiplied by 100). `as_raw`
  and byte parsing are unchanged.
- Remove `TimeSource::Gps`. `TimeSource::from_raw(3)` now returns
  `Reserved(3)`. Serde no longer accepts/emits the erroneous `Gps` source
  variant. `InternalRtc` remains named as before, documented correctly as
  internal from RMM; other SRC variants/signatures are unchanged.
- Add `TimeFormat::None`; `from_raw(15)` now returns it. Exhaustive matches
  need a new arm; serde gains `None`. It means invalid payload, not RTC.
- Add `time_enums::IrigTimeSource`, also re-exported at root, with variants
  `Freewheeling`, `TimeCommand`, `Rmm`, `ExternalIrig`, `ExternalGps`,
  `ExternalNtp`, `ExternalPtp`, `Embedded`, `Reserved(u8)` and
  `from_raw(u8) -> Self`. It preserves reserved/unrecognized values.
- Add public module `version`, containing `Irig106Version`, also re-exported
  at root. Variants: `V04`, `V05`, `V07`, `V09`, `V11`, `V13`, `V15`,
  `V17`, `V19`, `V22`, `Unknown(u8)`. Constructors:
  `from_setup_record(u8) -> Self`, `from_data_type_version(u8) -> Self`.
- Both new enums implement Debug, Clone, Copy, PartialEq, Eq, Hash;
  Serialize/Deserialize with the existing optional `serde` feature.
  There are no new required dependencies, allocations, features, or
  changes to `no_std`, edition 2024, or MSRV 1.85.

## What irig106-time must change

1. Update its git dependency from `e513af4` to the independently reviewed
   merge commit containing this change (or an owner-authorized release).
2. Remove `TimeSource::Gps` matches and source-3 version-specific GPS/None
   interpretation (`has_gps_time_source`); preserve/report reserved source.
   Do not confuse SRC with FMT's valid native GPS or ITS's external GPS.
3. Use corrected ERTC nanoseconds without an extra factor of 100. Revise
   conversion/correlation fixtures that assumed 100 ns ERTC ticks.
4. Handle `TimeFormat::None` as invalid payload before constructing a time
   reference. Decode ITS bits 15–12 via `IrigTimeSource::from_raw` only when
   applicable. Retain CSDW extraction and validation in time: reserved bits
   31–16 and 11–10 (mask 0xFFFF_0C00) for the ITS layout; before 106-17,
   bits 31–12 and 11–10 (mask 0xFFFF_FC00). Report ITS as not reported
   for older/undetermined editions, rather than freewheeling from zero.
5. Replace the local version code tables with the two shared constructors.
   Extract setup CSDW bits 7–0 for `from_setup_record`; pass the primary
   header data type version byte to `from_data_type_version`. Drop `V23`
   for code 15 and `Pre07` for code zero. Keep recording policy/capability
   decisions in time; unknown must not automatically enable capabilities.
   Network Time exists from 106-17 (a separate T-4 correction), not 106-22.
6. Update exhaustive matches, serde consumers, and downstream tests for the
   changed variants. Do not use data type version to select ITS semantics:
   Format 1 stays at 0x06 even after ITS was added.

## Source limitations and ambiguities

- 106-04 Chapter 10 is absent from the archive, as the finding records.
  No evidence supports the prototype's source-3 GPS/None rule. The checked
  editions reserve it; no special legacy interpretation is added here.
- ITS is edition-dependent: 106-15 §10.6.3.2 says “Bits 31-12 are reserved.”
  106-17 §11.2.3.2 defines “IRIG Time Source (ITS). Bits 15-12” while
  Table 11-4 still assigns Format 1 data type version 0x06. This library
  exposes the raw vocabulary; time must apply its reviewed reading policy.
- 106-24R1 §11.2.7.2 says “0x0E = RCC 106-22” and “0x0F through 0xFF = Reserved”.
  This cannot identify the exact later recording edition. The unchanged
  table through 106-24R1 is an observation, not a guarantee about the future.
- FMT 15 is explicitly “NONE (time packet payload invalid)”. Consumers
  must enforce that; a foundational enum does not parse/validate a body.

No blocking standard ambiguity for the shared mappings implemented here.

## Executed local CI equivalent

All ten workflow commands listed in README passed on this cloud host,
with `CARGO_BUILD_JOBS=2`. Stable was rustc 1.99.1; the MSRV checks used
rustc 1.85.0. Installed via rustup: stable, 1.85.0, stable clippy/rustfmt,
and stable wasm32-unknown-unknown. No project dependencies were added.

- Tests with all features, no default features, and serde: each passed
  41 unit tests, five standard regression tests, and one doctest.
- MSRV 1.85.0: default, no-default-features, and serde checks passed.
- Clippy, all targets/all features with `-D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- Documentation, all features with `RUSTDOCFLAGS="-D warnings"`: passed.
- wasm32-unknown-unknown build without default features: passed.

This is executed local validation, not a claim about GitHub Actions results.
Independent review and GitHub CI are required before merge; this assignment
opens a PR only, without merge, publication, tags, or settings changes.

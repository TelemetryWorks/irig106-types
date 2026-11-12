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
python3 scripts/fetch.py --edition 106-04 --kind full
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
| T-6 | §11.2.3.2 a, Figure 11-12, pp. 11-23–25 | SRC mapping from 106-05: 0 internal, 1 external, 2 RMM, 3–14 reserved, 15 none. 106-04 §10.6.3.3/Figure 10-15 defines EXT instead. Same SRC table in 106-05 Figure 10-16a, 106-07 Figure 10-22, 106-13/15 Figure 10-19 (§10.6.3.2). |
| T-7 | §11.2.3.2 a, Figure 11-12, pp. 11-23–25 | FMT 15 NONE from 106-05; 106-04 §10.6.3.3/Figure 10-15 reserves FMT 6–15. ITS 0–7 enumerated explicitly, 8–15 reserved. 106-15 Figure 10-19 reserves bits 31–12; 106-17 §11.2.3.2 defines ITS. |
| T-9 | §11.2.7.2, Figure 11-34, pp. 11-50–51; §11.2.1.1 e (version code list), Table 11-4 (current version per data type), p. 11-6 | Two explicit edition lists and every byte value tested. Setup 7–14 vs header 1–10. All other bytes unknown. 106-07 §10.6.7.2/Figure 10-50 reserves setup 0–6 too. 106-22/23/24/24R1 keep setup 14 = 106-22. |

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
  variant. Rename `InternalRtc` to `InternalFromRmm` for SRC 2; serde
  likewise uses `InternalFromRmm` instead of `InternalRtc`. Other SRC
  variants/signatures are unchanged. This mapping applies from 106-05;
  106-04 has EXT instead, handled by `irig106-time`.
- Add `TimeFormat::None`; `from_raw(15)` now returns it. Exhaustive matches
  need a new arm; serde gains `None`. It means invalid payload, not RTC,
  from 106-05 onward. 106-04 reserves FMT 6–15; edition handling stays in time.
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
   interpretation (`has_gps_time_source`); preserve/report reserved SRC
   values from 106-05 onward. Rename `InternalRtc` matches and serde data to
   `InternalFromRmm`. For 106-04, decode EXT bit 0 and validate reserved bits
   3–1 instead of using the SRC mapping. Its packet data type version is
   0x01 (Initial Release; called Header Version in 106-04 §10.6.1.1.5).
   Do not confuse SRC with FMT's valid native GPS or ITS's external GPS.
3. Use corrected ERTC nanoseconds without an extra factor of 100. Revise
   conversion/correlation fixtures that assumed 100 ns ERTC ticks.
4. Handle `TimeFormat::None` as invalid payload from 106-05 onward before
   constructing a time reference; for 106-04, FMT 0x6–0xF is reserved.
   Decode ITS bits 15–12 via `IrigTimeSource::from_raw` only when
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

- 106-04 Chapter 10 **is archived** inside `files/106-04/106-04.pdf`, headed
  “SOLID STATE ON-BOARD RECORDER STANDARD”. Verified §10.6.3.3, Figure 10-15,
  pp. 10-30–31: the layout is RESERVED | DATE | FMT | EXT. “Bit 0 indicates
  if the external time source is present”; “Bits 3-1 are reserved”; FMT
  “0x6 thru 0xF = Reserved.” The SRC table and FMT NONE first appear in
  106-05 Figure 10-16a. The prototype's claim that 3 means None in 106-04
  is incorrect. Edition-aware EXT/SRC and FMT handling belongs to time.
  The 106-04 header uses 0x01, “Initial Release Header Version”
  (§10.6.1.1.5, p. 10-19), named 106-04 by the later data type version list.
- ITS is edition-dependent: 106-15 §10.6.3.2 says “Bits 31-12 are reserved.”
  106-17 §11.2.3.2 defines “IRIG Time Source (ITS). Bits 15-12” while
  Table 11-4 still assigns Format 1 data type version 0x06. This library
  exposes the raw vocabulary; time must apply its reviewed reading policy.
- 106-24R1 §11.2.7.2 says “0x0E = RCC 106-22” and “0x0F through 0xFF = Reserved”.
  This cannot identify the exact later recording edition. The unchanged
  table through 106-24R1 is an observation, not a guarantee about the future.
- From 106-05 onward, FMT 15 is explicitly “NONE (time packet payload invalid)”. Consumers
  must enforce that; a foundational enum does not parse/validate a body.

No blocking standard ambiguity for the shared mappings implemented here.

## Executed local CI equivalent

All ten workflow commands listed in README passed again after the review
corrections and `InternalFromRmm` rename on this cloud host,
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


## Chapter 4 binary time layout (T-10)

Baseline: main `c59e0042669f69900e05dd6f23a987d30218ebd6`, clean checkout,
origin `https://github.com/TelemetryWorks/irig106-types.git`. Tests-only
commit `4a2499341ab085876aaf245f1548ce4842ff5e9e` adds hand-derived fixtures
without implementation changes. This follow-up fixes types first under
accepted `irig106-time` ADR-0015; no files in time are changed.

### Verified page images and edition coverage

Used the same manifest-verified standards archive described above, with:

```sh
python3 scripts/fetch.py --edition 106-04 --kind full
python3 scripts/fetch.py --edition 106-05 --edition 106-07 --edition 106-09 --edition 106-11 --edition 106-13 --edition 106-15 --edition 106-17 --edition 106-19 --edition 106-20 --edition 106-22 --edition 106-23 --edition 106-24 --edition 106-24R1 --chapter 4 --kind chapter
python3 scripts/fetch.py --edition 106-05 --edition 106-07 --edition 106-09 --edition 106-11 --edition 106-13 --edition 106-15 --chapter 10 --kind chapter
python3 scripts/fetch.py --edition 106-17 --edition 106-19 --edition 106-20 --edition 106-22 --edition 106-23 --edition 106-24 --edition 106-24R1 --chapter 11 --kind chapter
pdftoppm -f 18 -l 18 -scale-to 1600 -png -singlefile files/106-24R1/chapter11.pdf figure-11-4
pdftoppm -f 14 -l 15 -scale-to 2000 -png files/106-24R1/chapter4.pdf chapter4-figures
```

Rendered and visually inspected the baseline page and the following edition
pages with `pdftoppm` and the image viewer; inspected the IPTS/flag text in
each edition as well. Images/PDFs remain external to this Git repository.

| Edition | Printed page / PDF page (1-based) | Evidence inspected |
|---|---|---|
| 106-04 | 10-19–21 / 277–279 (full PDF) | §10.6.1.1.7: bits 3–2 = 00 select Chapter 4 secondary time, bit 6 selects it for IPTS; §10.6.1.2.1 and §10.6.1.3.2 refer to those flags. No layout figure. |
| 106-05 | 10-25 / 29 | §10.6.1.2 a, §10.6.1.3 b: secondary and IPTS times selected by packet flags; no dedicated Chapter 4 secondary-time layout figure in this edition. |
| 106-07 | 10-36 / 40 | Same sections and linkage; no dedicated Chapter 4 secondary-time layout figure. |
| 106-09 | 10-34 / 44 | Same sections and linkage; no dedicated Chapter 4 secondary-time layout figure. |
| 106-11 | 10-34 / 44 | §10.6.1.2 a, Figure 10-15: microseconds above reserved, high above low in the two 32-bit rows; §10.6.1.3 b links IPTS to flag-selected format. |
| 106-13 | 10-33 / 45 | Figure 10-12: same layout. |
| 106-15 | 10-30 / 42 | Figure 10-12: same layout. |
| 106-17, 106-19, 106-20, 106-22, 106-23, 106-24, 106-24R1 | 11-10 / 18 in each | Figure 11-4: same layout. |
| 106-04 Chapter 4 | 4-8–9 / 63–64 (full PDF) | §4.7.4 states 655.36 s; Figure 4-3 draws two low-word zeros and no microsecond-word zeros. |
| 106-05 Chapter 4 | 4-8–9 / 10–11 | Same §4.7.4 wording and Figure 4-3 zero placement as 106-04. |
| 106-07, 106-09, 106-11 Chapter 4 | 4-9 / 11 in each | Figure 4-3 matches the later pattern: one low-word zero, two microsecond-word zeros. |
| 106-13, 106-15 Chapter 4 | 4-9 / 13 in each | Figure 4-3: same later pattern. |
| 106-17, 106-19, 106-20, 106-22, 106-23, 106-24 Chapter 4 | 4-10 / 14 in each | Figure 4-4: same later pattern. |
| 106-24R1 Chapter 4 | 4-10 / 14 and 4-11 / 15 | §4.7, Figures 4-4 and 4-5: binary weighting, microsecond leading zeros, PCM insertion distinct from packet storage. |

For **106-04 through 106-09**, the packet word order used here is **inferred
from the 106-11-and-later layout figure**, not explicitly drawn in those
older editions. Their Chapter 4 packet-flag descriptions only specify that
the two LSBs of the 64-bit secondary/IPTS time are zero-filled; that wording
does not establish the four-word order. The shared flag selection alone is
not independent evidence of that order.

Figure 11-4 labels the first 32-bit row's high half **Micro-Seconds Word**
and low half **Reserved**; the second row's high half **High Order Time
Word** and low half **Low Order Time Word**. Section 11.1 defines least
significant byte first. Thus the byte intervals are:

| Bytes (half-open) | Little-endian 16-bit word |
|---|---|
| 0..2 | Reserved |
| 2..4 | Microseconds |
| 4..6 | Low-order time |
| 6..8 | High-order time |

For IPTS, §11.2.1.1 g bit 6 chooses RTC or secondary-header time; bits 3–2
select Chapter 4 for the latter. Section 11.2.1.3 b specifies the flag-selected
8-byte timestamp. The same correspondence is in the checked earlier editions
(Chapter 10 §10.6.1.1 g / §10.6.1.3 b in 106-05 through 106-15;
§10.6.1.1.7 / §10.6.1.3.2 in 106-04). This establishes format selection;
the pre-106-11 byte order remains the inference stated above.

### Binary weighting and figure caveat

Chapter 4 §4.7 explicitly says the high word's “lsb, bit 16, has a value of
655.36 seconds” when binary weighted; the low word resolves 0.01 s. The ratio
655.36 / 0.01 = 65536 gives a full 32-bit binary count across both words.
There is no day-of-year partition or 17-bit-only counter in this binary
representation. `low = 0x8000`, `high = 0` means 32768 × 0.01 = 327.68 s.

The page images differ by edition. **106-04 and 106-05 Figure 4-3**
(printed 4-9; full-PDF page 64 and chapter-PDF page 11 respectively) draw
**two** zeros on the low word and **none** on the microsecond word. Their
Figure 4-4 on the same page already draws two microsecond zeros. From
**106-07 onward**, the standardized 16-bit figure matches 106-24R1:
one low-word zero and two microsecond-word zeros (Figure 4-3 through
106-15, Figure 4-4 from 106-17). These are observed drawing differences,
not a change to the binary weighting used here: 106-04 and 106-05 §4.7.4
also state the high-word resolution as 655.36 seconds (printed 4-8).
The no-mask decision therefore stands for those editions too.

In the 106-24R1 baseline, Figures 4-4/4-5 show a zero in the low word's
leading position alongside BCD/binary annotations. Applying that drawn zero as a binary mask would
contradict the explicit 655.36-second high-word weighting; it is not forced
to zero by this implementation. This preserves every high/low bit as requested.
The baseline microsecond word separately shows **two** leading zero positions;
§4.7 defines 1 µs resolution, resetting at 10 ms, maximum 9999.

Chapter 4's PCM insertion order (high, low, microseconds, with zero filler
for a 12-bit PCM word size in Figure 4-5) is not the Chapter 11 packet-byte
order. The parser handles the latter and retains raw words. As before, it
validates length only, ignores the reserved word, and does not validate or
mask the microsecond word. Consumers enforce conformance and absolute-time
interpretation; no new validation policy or conversion API is introduced.

### Hand-derived regression fixtures and executed before/after evidence

`tests/t10_ch4_layout.rs` has no generated encodings or round-trip oracle.
The distinct-word fixture is `[00 00 | 34 12 | 01 80 | 02 00]`: reserved=0,
microseconds=4660, low=32769, high=2. All four words differ, and the valid
microsecond word has both leading bits clear. Expected count is
2 × 65536 + 32769 = 163841; expected duration is 1638.414660 s.

Other literal fixtures cover low=0x8000 at 327.68 s, high=1 plus low=0x8000
at 983.04 s, and high=low=0xFFFF at 42949672.95 s. Both parser entry points
are checked against literal expected fields/counts, independently of each
other. Arithmetic in the test expresses §4.7; this crate supplies no converter
from which expected values could be copied.

Executed `cargo test --test t10_ch4_layout` before the fix, on the tests-only
commit's main implementation: **0 passed, 3 failed**.

| Test | Before | After |
|---|---|---|
| `secondary_fields_from_figure_11_4` | Actual high=4660/usec=2; expected high=2/usec=4660 | PASS |
| `intra_packet_fields_from_figure_11_4` | Same independent literal-field failure | PASS |
| `binary_weighting_preserves_all_32_bits` | Actual 32768 ticks; expected 98304 (high=1, low=0x8000) | PASS |

The isolated high=0/low=0x8000 fixture already passed before the fix; it
remains a regression guard against adding a 15-bit low-word mask. The test
as a whole failed at the next fixture, which distinguishes the swapped words.

### API effect and irig106-time adoption

Public type, fields, signatures, serde shape, and error behavior are unchanged.
The behavior of `Ch4BinaryTime::from_secondary_bytes` and
`from_intra_packet_bytes` changes: high now reads bytes 6..8 instead of 2..4,
and microseconds now reads bytes 2..4 instead of 6..8. Low stays at 4..6.
No bits of either count word are discarded.

After independent review and merge, `irig106-time` must advance its pinned
`irig106-types` dependency, remove workarounds for the old swap, and update
secondary-header/IPTS byte fixtures. Its Chapter 4 conversion must use the
full binary count (cast to u32 before shifting high by 16), then 10 ms per
tick plus the microsecond fraction. Replace the prototype's 17-bit/day-of-year
split; do not mask low to 15 bits. Validation of reserved and microsecond
bits/ranges, calendar/year assumptions, and flag selection remain in time.

Executed full local CI equivalent for this follow-up, with two build
workers and the unchanged Rust stable / 1.85.0 toolchains: all ten commands
in README passed. Each test configuration (all-features, no-default-features,
serde) passed 41 unit tests, eight integration tests, and one doctest.
MSRV checks (default, no-default, serde), clippy with `-D warnings`, fmt,
docs with `-D warnings`, and the no-default wasm32 build passed. These are
executed local checks, separate from source inspection and GitHub Actions.

# PacketFacts implementation, evidence and adoption

Base: clean `origin/main` merge `701e9f7` (approved design PR #25).
Origin: `https://github.com/TelemetryWorks/irig106-types.git`.
Version remains 0.1.0; no tag, release, publication or sibling code change.
The owner decides the next release, 0.2.0. Published 0.1.0 lacks these APIs.

## Constructor decision before tests/code

Commit `3c6f046` records the frozen `PacketFacts::new_v1` signature and updates
source pins before implementation. All nine observations are required; there
is no `Default`. Future observations must represent absence explicitly, and V1
must leave them absent. A new versioned constructor can supply them; an
always-required observation needs a reviewed API/type transition. The 32-byte
budget remains a separate review gate. No speculative future fields are stored.
See [the evolution contract](PACKET-FACTS.md#constructor-evolution-implementation-decision-2026-11-23).

## Tests first: missing API, then passing observations

Tests-only commits `1fbc8d0` and `39683c1` precede all library changes. The
latter corrects the serde map test adapter before recording the clean failure.
At `39683c1`, source/manifest/dependencies remain identical to main; only the
up-front design clarification and tests have changed.

| Command | Before implementation | After implementation, local aarch64 |
|---|---|---|
| `cargo test --locked --all-features --test packet_facts` | Exit 101, E0432: PacketFacts and KnownDataType absent | 6 passed |
| `cargo check --locked --target wasm32-unknown-unknown --no-default-features --test packet_facts_layout` | Exit 101, E0432: PacketFacts absent | Target-size const assertion evaluated successfully |
| `cargo test --locked --all-features --test packet_facts_layout -- --nocapture` | Not run separately before; the wasm check above captured the missing import | 1 passed; printed `PacketFacts: 32 bytes on aarch64` |

The before results are compile failures, not executed assertion failures.
Regression oracles are literal observations and hand-transcribed code/flag
truth tables from 106-24R1 §11.2.1.1 b–i / Tables 11-2 and 11-4. They do not
call crate encoders to generate expectations. Both new test files cite their
source or distinguish API layout/ownership requirements from wire rules.

Coverage: every field's zero/max domain; distinct observations to detect
swaps; a frozen constructor function-pointer signature; Copy/Send/Sync;
all 256 raw data-type bytes and all 256 flags; unknown and valid-but-unnamed
codes; contradictory flags/lengths retained; separate DATA borrows; serde
trait support and unvalidated raw deserialization, including the existing RTC
serde caveat. Rustdoc also checks external struct construction/exhaustive
known-code matching are unavailable.

## Target layout and bounded local checks

`tests/packet_facts_layout.rs` has a mandatory const assertion
`size_of::<PacketFacts>() <= 32` plus a native runtime assertion. CI:

- Existing Test job executes it on x86_64 alongside all feature modes.
- Native `ubuntu-24.04-arm` job confirms aarch64 and executes the layout test.
- WASM job checks that test on wasm32-unknown-unknown, evaluating the target
  layout assertion at compile time without claiming a wasm runtime execution.
  The release qualification build repeats that compile-time target check.

Local Rust stable 1.99.1 on aarch64: all-features targeted tests (6 + 1),
no-default targeted tests (5 + 1), wasm32 layout check, clippy all targets/all
features with warnings denied, rustdoc warnings/broken links denied, 12
rustdoctests (8 runnable + 4 expected compile failures), and guide examples
passed. MSRV 1.85.0 check with no default features and serde passed. Formatting,
actionlint and default-persona zizmor passed. Full feature-suite/package
qualification and x86_64/aarch64 CI evidence are reported on the PR at its exact
head, rather than claiming local execution on an unavailable architecture.

## Public API and integration work

Add public module `packet_facts` and root re-exports:

- `PacketFacts`: non-exhaustive, Debug/Clone/Copy/Eq/Hash; optional serde.
  `const new_v1(u64, Rtc, u32, u32, u16, u8, u8, u8, u8) -> Self`.
- Nine raw getters: `byte_offset() -> u64`, `rtc() -> Rtc`,
  `packet_length()/data_length() -> u32`, `channel_id() -> u16`,
  `data_type()/data_type_version()/sequence()/flags() -> u8`.
- `known_data_type() -> Option<KnownDataType>`; boolean
  `has_secondary_header()`, `intra_packet_uses_secondary_time()`,
  `rtc_sync_error()`, `data_overflow()`; `secondary_time_format() -> SecHdrTimeFormat`;
  `data_checksum_bytes() -> u8` (0/1/2/4). All methods except
  `secondary_time_format` are const, including construction.
- `KnownDataType`: non-exhaustive, Debug/Clone/Copy/Eq/Hash; optional serde;
  variants `ComputerGeneratedFormat1`, `TimeFormat1`, `TimeFormat2`.
  These names are not an enum-cast wire encoding or an edition/trust decision.

All existing APIs and dependencies are unchanged. No alloc, no heap members,
no core/TMATS dependency, no payload storage, no parser and no semantic policy.
Size/Copy tests do not establish end-to-end allocation counts or throughput.

Core must explicitly construct V1 from its header getters and packet offset.
The orchestrator borrows facts and DATA, preserves trust/capture/gap/session
and same-channel interruption evidence separately, and delivers in input order.
TMATS consumes these with INT-011/045 policy and its own budgeted assembly;
time can accept the shared facts without a core dependency. Those adapters,
sideband interfaces, closure/completeness semantics and reusable-buffer reader
are not implemented here. Consumers must wait for a reviewed merge/owner-timed
release before changing their dependency, and may not treat a named code or
raw header value as a trusted time marker or governing metadata.

## Refreshed authority

[The contract](PACKET-FACTS.md#sources-and-freshness) pins TMATS
`3bb197bb0bf846ed5e083e3a984873e834197f2a` (guarded INT-045 independently approved)
and final requested irig106-docs #4 head
`db6023940329e07549e30e2052083dcadf025100`. Source inspection is separate from
executed tests; no sibling repository build/test or edit was performed.

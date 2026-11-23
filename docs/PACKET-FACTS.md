# PacketFacts design and requirements

Status: **design approved in PR #25; implementation in progress**.
Owner boundary decision: [ADR-0001](adr/0001-shared-packet-header-facts.md),
2026-11-23. This adds shared vocabulary; the next release is **0.2.0**, when
the owner decides. Cargo stays at 0.1.0 in this implementation PR; no release is made.

## 1. Boundary and flow

Core produces a plain `PacketFacts`; an orchestrator passes `&PacketFacts`
and a separate borrowed DATA slice to selected consumers. TMATS and time
name these inputs through types, without depending on core. Facts contain
scalars by value, not ownership of packet storage. They need no payload
lifetime parameter; the separate slice has the backing buffer's lifetime.
A cheap scalar copy is permitted; the normal synchronous seam borrows both.

| Stage | Input → output | Responsibility |
|---|---|---|
| Core | Framed packet → facts, DATA borrow, separate integrity evidence | Structural reading, bounds and integrity evidence; no TMATS/time meaning |
| Orchestrator | Facts + bytes + external context → selected consumer | One header classification/dispatch per packet, ordered delivery and buffer lifetime |
| TMATS | Facts + setup DATA + labelled context → candidate records/findings | INT-011 joining, INT-045 closure/ordering, separate completeness/adoption gates |
| Time | Time packet + qualified context → time results | Payload interpretation and correlation; no dependency on core or TMATS |
| Decode | Selected data + qualified metadata/time answers → decoded values | Payload decoding; no general per-data-packet TMATS callback |

All new seams above remain planned. PacketFacts is neither core's `Header`
nor evidence that a packet is valid. It performs no parsing, I/O, checksum
verification, recovery, session tracking, policy selection or semantic decoding.

## 2. Proposed value contract

**PF-001 — Representation.** Add a fully documented, `#[non_exhaustive]`
`PacketFacts`, deriving `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` and `Hash`.
Retain `no_std`, no `alloc`, edition 2024, MSRV 1.85 and no new required
dependencies. Construction, copying, getters and code/flag views allocate
nothing. No `Vec`, `String`, smart pointer, embedded byte slice or owned payload.
Optional serde follows the crate's value-type feature convention; it is not
validation. No automatic trust or edition inference occurs at deserialization.

**PF-002 — Fields.** Store exactly the following observations once. All raw
integer domains are accepted and retained; contradictory lengths/flags remain
observable, not repaired. Construction must explicitly supply every field,
without defaults masquerading as observations. The constructor is the frozen,
versioned `PacketFacts::new_v1(byte_offset, rtc, packet_length, data_length,
channel_id, data_type, data_type_version, sequence, flags)`. It accepts all nine
observations explicitly and performs no validation or repair.

### Constructor evolution (implementation decision, 2026-11-23)

`#[non_exhaustive]` prevents downstream struct construction/exhaustive matching;
it does not make a growing public constructor source-compatible. Freeze the
V1 signature and its nine meanings. Do not add parameters to it, add a `Default`
implementation, or fabricate zero values for observations not supplied by V1.
Future observations must use `Option<...>` (or an equally explicit absence type),
and V1 must initialize them to `None`/not supplied. A new `new_v2` can require
those additional observations explicitly, including an explicit `None` when
appropriate. An always-required future observation needs a separately reviewed
API/type transition rather than silently changing V1. All such changes must
still satisfy, or explicitly re-review, the 32-byte budget. No speculative
future fields or absence slots are stored in this implementation.

The nine-argument constructor has a narrowly scoped clippy argument-count
exception: keeping the approved fields explicit and the versioned signature
stable is intentional. There is no defaulting builder or competing facts type.

| Field / getter | Type and range | Origin and meaning |
|---|---|---|
| `byte_offset` | `u64`, full range; bytes | Caller/core stream provenance: packet sync start, not a header field or session-start assertion |
| `rtc` | Existing `Rtc`; constructor-produced domain 0..2^48−1 ticks | §11.2.1.1 i; 100 ns/tick, no absolute epoch or unwrapping |
| `packet_length` | `u32`, full raw range; bytes | §11.2.1.1 c; declared whole packet length |
| `data_length` | `u32`, full raw range; bytes | §11.2.1.1 d; declared DATA length, excluding headers and trailer filler/checksum |
| `channel_id` | `u16`, full range | §11.2.1.1 b; raw channel, no source/session identity inferred |
| `data_type` | `u8`, full range | §11.2.1.1 h / Table 11-4; raw routing code |
| `data_type_version` | `u8`, full range | §11.2.1.1 e; raw per-data-type layout version, not recording edition or setup RCCVER |
| `sequence` | `u8`, full range | §11.2.1.1 f; per-channel modulo-256 counter, not global order |
| `flags` | `u8`, full range | §11.2.1.1 g; raw bits retained even for unsupported/contradictory combinations |

The source is [IRIG 106-24R1 Chapter 11](https://github.com/TelemetryWorks/rcc-106-standards/releases/download/106-24R1/chapter11.pdf),
section **11.2.1.1**, Table **11-2** (packet layout), printed pp. 11-3–10.
Offset is an API provenance quantity, not an invented wire field. Reuse `Rtc`
without silently masking/reconstructing it; its existing serde caveat still applies.

The scalar storage totals 30 bytes before padding: two 8-byte values, two
4-byte lengths, one 2-byte channel and four bytes of codes/flags. Target
`size_of::<PacketFacts>() <= 32` on x86_64, aarch64 and wasm32, verified in
implementation tests. Do not promise a stable Rust field layout, packed layout,
C ABI or raw-byte transmutation. Any future size increase needs explicit review.

**PF-003 — Raw code plus named view.** `data_type()` always returns the
original byte. A computed `known_data_type() -> Option<KnownDataType>` view
initially names `ComputerGeneratedFormat1` (0x01), `TimeFormat1` (0x11), and
`TimeFormat2` (0x12), per Table 11-4. Make `KnownDataType` non-exhaustive;
other codes remain raw and yield `None` in this deliberately limited view.
`None` means “not named by this API”, not “reserved in the standard”,
“corrupt” or “unsupported by every consumer”. Do not store the view redundantly.
In particular, 0x12 can have a named baseline view while remaining reserved
in a pre-106-17 recording. This API does not decide applicability or whether a
time marker is trusted. Existing `Irig106Version` mappings remain separate;
there is no automatic recording-edition selection.

**PF-004 — Mechanical flag views.** Compute these from the unchanged flag
byte (§11.2.1.1 g, printed pp. 11-8–9):

| Proposed accessor | Extraction | Meaning, without validation |
|---|---|---|
| `has_secondary_header()` | bit 7 | Declared presence; does not prove twelve secondary bytes exist |
| `intra_packet_uses_secondary_time()` | bit 6 | Declared IPTS source; bit 6 set with bit 7 clear is retained, not repaired |
| `rtc_sync_error()` | bit 5 | Reported RTC sync-error bit; not an absolute trust verdict |
| `data_overflow()` | bit 4 | Reported overflow bit; does not itself prove lost payload |
| `secondary_time_format()` | bits 3–2 | Reuse `SecHdrTimeFormat::from_packet_flags`, including reserved 0b11 |
| `data_checksum_bytes()` | bits 1–0 | 0b00→0, 0b01→1, 0b10→2, 0b11→4 bytes; width, not verification |

No accessor rejects or clears bits, tests packet contents, or turns a flag
into a policy decision. Older-edition applicability stays with the consumer.

**PF-005 — Borrowed payload, separate context.** Do not put CSDW, body bytes,
FRMT/RCCVER/SRCC, integrity grades, edition bindings, session assertions,
phase/record identities or completeness in PacketFacts. Conceptual seam:

```text
selected_consumer.observe(&facts, data: &[u8], separate_context)
```

This is design notation, not an implemented method. `data` means DATA only:
CSDW at its beginning where that packet format defines one, with no primary
or secondary header, trailer filler or packet data checksum (§11.2.1.2–1.4;
setup Format 1 §11.2.7.2 / Figure 11-34). TMATS checks for the four-byte CSDW
and interprets it, then uses the remaining setup text; types never extracts it.
A raw CSDW need not be stored twice or included among header facts.

The producer/orchestrator must distinguish full DATA from a captured prefix
using `data_length`, bounds and separate truncation/integrity evidence. Never
treat a short capture as a complete setup fragment. Checked offset arithmetic
locates text at packet offset + 24 + (12 if bit 7) + 4; reject/report overflow
and insufficient bytes in the appropriate reader/consumer, not by clamping in
types. Keep borrowed slices within buffer lifetime; retention requires separately
budgeted ownership in the assembler or moving a buffer outside this value.

## 3. Core adapter and ordered TMATS consumption

**PF-006 — Core production.** The inspected core already exposes all inputs:

| Facts input | Existing core API |
|---|---|
| Byte offset | `Packet::offset` |
| Channel / raw data type / data-type version | `Header::channel()`, `data_type()`, `data_type_version()` |
| Sequence / flags / RTC | `Header::sequence()`, `flags()`, `rtc()` |
| Declared lengths | `Header::packet_length()`, `data_length()` |
| Separate DATA/trust inputs | `Packet::body_prefix`, `ranges`, `secondary_integrity`, recovery evidence |

Core's future adapter fills facts from those observations after its own
structural checks. This PR changes neither core nor its public API. Current
core 0.1 uses an owned `Vec` DATA prefix and can allocate per capture; borrowing
`&packet.body_prefix` does not make that reader allocation-free or guarantee
full DATA. The planned reusable-buffer core API is a separate integration.

**PF-007 — Joining (INT-011).** TMATS owns independent per-channel runs;
channel, sequence, raw version/flags/RTC and offset retain provenance. TMATS
interprets each CSDW and checks same-channel sequence +1 modulo 256 and
matching FRMT/RCCVER. Other-channel packets never mix with or interrupt a
run. Preserve start/stop reasons and every raw fragment/CSDW observation;
channel-0 conformance and SRCC interpretation are separate checks.

Selective routing must not erase observed same-channel interruption evidence.
Modulo +1 alone can alias after 256 omitted packets. The producer/orchestrator
must retain/report that evidence separately using bounded state, without a
general per-data-packet TMATS callback after the initial phase. The exact
sideband interface is an integration requirement, not a new fact, silent
assumption of continuity or implementation in this PR.

**PF-008 — Closure and initial ordering (INT-045).** Deliver in physical
stream order; never sort by RTC, channel, sequence or worker completion.
The orchestrator supplies caller-asserted session start, explicit trust/gap
observations and a reviewed edition/format binding separately. Offset zero,
sync success or q=0 cannot imply a known session start.

TMATS observes all initial headers to enforce the static Format 1 prefix.
A trusted recognised time marker on any channel closes initial candidates
immediately before that marker; an untrusted first marker cannot be silently
replaced by the next trusted one. The later SRCC=1-first-fragment extension
closes an eligible update at the next trusted recognised time packet. It is
owner policy, not a universal terminator printed by the standard; SRCC=0-first
retransmissions do not acquire eligibility from a later fragment's bit.
Raw 0x11/0x12 names alone do not establish trust or edition eligibility (0x12
is defined from 106-17). Secondary timestamps and IPTS are not closure markers.
Source NONE does not by itself disqualify an otherwise trusted time packet.

Seal/assessment precedes time interpretation of that same marker packet.
TMATS never decodes its time payload or appends it to setup text. Later
otherwise joinable fragments cannot cross the sealed phase backwards. Closure
does not prove completeness, unique metadata or adoption: INT-034 and other
TMATS contracts own incomplete/ambiguous outcomes, possible premature-seal
revisions, checksums and adoption withholding. No such policy enters types.

**PF-009 — Performance.** Follow pipeline.md's owner-approved target:
one data-type classification/dispatch per packet; initial-header observations
and cheap marker notifications while a candidate is open are explicit
exceptions to selective semantic routing. Data otherwise goes only to decode.
Facts and views allocate nothing; payloads cross libraries as borrowed slices.
No per-packet allocation on the steady-state path after buffer capacity exists
is an integration target, not a claim about core 0.1 or setup assembly.
Do not add a hidden reader, payload copy, broadcast path or unbounded retention.
Buffer growth, fixed pool/queues and separately budgeted assembly belong outside
types; lifetime/ownership transfer and backpressure remain orchestrator duties.
Actual performance claims require the pipeline's later throughput/allocation
benchmarks; a small `Copy` type alone is not performance evidence.

## 4. Tests-first acceptance for the implementation PR

These are planned checks, not executed implementation results:

- PF-001/002: compile-time Copy assertion, target size budget, round-trip every
  field's boundaries, no defaulted observations; no-default/MSRV/wasm checks.
- PF-003: literal standard-derived named codes; all 256 raw values retained;
  distinguish unnamed standard codes from invalidity; version byte stays raw.
- PF-004: literal flags 0x80/0x40/0x20/0x10, all four checksum codes and
  reserved secondary format. Exhaust all 256 bytes; preserve bit 6 without
  bit 7. Expected values come from §11.2.1.1 g, not the implementation.
- PF-005: documented borrowed call pattern compiles; no payload ownership or
  CSDW/semantic fields; optional serde preserves raw facts, never validates.
- Cite each table/section in new tests and demonstrate their pre-implementation
  missing-API failure before implementing. Keep every public item documented,
  with compiled useful examples and broken links denied.
- Run the meaningful checks and required CI for implementation. Cross-crate
  joining/closure, 256-packet interruption, truncation, ordering, trust and
  buffer-reuse/allocation tests belong to their adapters/consumers; types unit
  tests cannot establish those integrations or complete IRIG conformance.

## Sources and freshness

Read-only source inspection on 2026-11-23; no sibling builds/tests or edits.
The following links pin the inspected inputs rather than assuming main is fixed:

- [Core header](https://github.com/TelemetryWorks/irig106-core/blob/3c12bcd33b1ae66cca2d6378f4bfe08f2fc8c9a1/crates/irig106-core/src/header.rs)
  and [reader](https://github.com/TelemetryWorks/irig106-core/blob/3c12bcd33b1ae66cca2d6378f4bfe08f2fc8c9a1/crates/irig106-core/src/reader.rs).
- [TMATS interpretation register, INT-011/045](https://github.com/TelemetryWorks/irig106-tmats/blob/3bb197bb0bf846ed5e083e3a984873e834197f2a/docs/INTERPRETATIONS.md):
  INT-011 and guarded INT-045 initial/mid-session extension have completed
  owner and independent review at this snapshot. Types supplies observations;
  it does not implement those TMATS semantics.
- [Pipeline performance design](https://github.com/TelemetryWorks/irig106-docs/blob/db6023940329e07549e30e2052083dcadf025100/src/architecture/pipeline.md#5-performance-design),
  final irig106-docs PR #4 head requested for this implementation (still open
  at inspection). It describes planned integrations, selective dispatch and
  the reviewed first-fragment SRCC guards; it is not a performance measurement.
- Standard PDF above: SHA-256
  `0f63792dcac39f6010df371ee0afc46a396017918f4155cba9ebab6c2d6d263e`,
  matched to the standards archive manifest; inspected header text and tables.
  Older-edition code/format applicability remains explicitly external.

Only the ownership, shape and performance direction are owner decisions in
ADR-0001. The concrete field/view requirements here are proposed for this PR's
exact-head review; no future API, benchmark or integration is claimed shipped.

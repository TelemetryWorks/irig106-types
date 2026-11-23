# 0001 — Shared packet-header facts

Status: accepted (owner, 2026-11-23); detailed requirements approved in PR #25 at `ae760b8`.

The shared plain packet-facts type lives in `irig106-types`. Core produces
these header facts, an orchestrator passes them on, and the TMATS setup
assembler consumes them. TMATS and time remain independent of core.

`PacketFacts` is a small `Copy` value targeting about 32 bytes, with no heap
storage. Consumers borrow the facts and receive payload bytes separately as
a borrowed slice; the facts do not own packet storage. It carries header
facts only, with no validation policy, TMATS interpretation or time semantics.
The wire-field authority is IRIG 106-24R1 Chapter 11 section 11.2.1.1.

Follow the pipeline performance design: one dispatch per packet and no
per-packet allocation on the steady-state path after buffer capacity exists.
Initial ordering observations and cheap time-marker notifications remain
part of the ordered orchestration path. These are design targets, not claims
about the current core reader or measured performance.

The addition belongs to the next release, 0.2.0, whose timing the owner
decides. This decision does not authorize a version bump, tag or release.

The contract and source references are in
[PacketFacts requirements](../PACKET-FACTS.md).

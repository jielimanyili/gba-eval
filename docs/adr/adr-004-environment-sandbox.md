# ADR-004: Package the candidate in a Docker sandbox with an oracle CLI and no internet

**Date:** 2026-11-18
**Status:** Accepted

## Context

The eval gives models a Docker container with the toolchain, an ABI spec, dev ROMs, and an oracle CLI (a black-box Mesen2 wrapper), with no internet and periodic checkpoints. Reproduce this shape for the portfolio environment.

## Decision

Provide a Docker image containing: the Rust + wasm32 toolchain, the ABI spec (task statement, `TASK.md`), dev ROMs, and an `oracle` CLI that runs a ROM + input sequence and returns Mesen2's frames/audio. The candidate emulator runs inside this sandbox with no internet. Checkpoints (git commits) are made every milestone.

## Rationale

- A reproducible environment = honest grading (no hidden shortcuts).
- The oracle CLI is the one allowed interaction with the reference.
- Mirrors the original's "no internet / black-box oracle" trust boundary.

## Alternatives considered

- **Grading on the host** — non-reproducible; rejected.
- **Exposing Mesen2 source in the container** — violates ADR-002; rejected.

## Consequences

**Positive**
- Reproducible, defensible environment; a clear "how models run" story.

**Negative / trade-offs**
- Docker + wasm toolchain image is heavy to build; oracle determinism must be pinned (fixed Mesen2 version).

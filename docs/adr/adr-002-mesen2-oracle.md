# ADR-002: Grade against Mesen2 as a black-box oracle

**Date:** 2026-11-18
**Status:** Accepted

## Context

A grading harness needs a reference to compare the candidate emulator against. GBA Eval uses Mesen2, "one of the most accurate software GBA emulators available." The candidate must not read Mesen2's source — accuracy must be demonstrated against observable behavior, not copied implementation.

## Decision

Use the Mesen2 binary as a black-box oracle: a CLI wrapper runs Mesen2 on a ROM + input sequence and emits reference frames + audio. The candidate emulator never has access to Mesen2's source; it only sees the oracle's outputs.

## Rationale

- Mesen2 is the accepted gold standard for GBA accuracy.
- A black-box oracle mirrors the eval's trust boundary (and this project's honesty boundary).
- Frames + audio are the natural comparison units for "is it playable."

## Alternatives considered

- **mGBA / VBA-M** — accurate but the eval standardizes on Mesen2; rejected.
- **Self-reference (our emulator grades itself)** — meaningless; rejected.
- **Reading Mesen2 source for implementation** — violates the spirit of the eval; rejected.

## Consequences

**Positive**
- Honest, reproducible grading; a clear trust boundary.

**Negative / trade-offs**
- Mesen2 (GPL) is used as a binary only — never vendored or modified; document this to avoid license confusion.

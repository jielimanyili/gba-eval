# ADR-003: Score via SSIM replay (60%) + audio spectrogram (20%) + procedural ROMs (20%)

**Date:** 2026-11-18
**Status:** Accepted

## Context

Reproduce GBA Eval's grading: a weighted average of replay, audio, and procedural sections. This is what "grading" means in the eval, and matching the methodology (not just the weights) is the point.

## Decision

- **Replay (60%)** — feed identical button inputs to candidate and oracle; compare every frame with SSIM (structural similarity). Best signal for "games are actually playable."
- **Audio (20%)** — compare sound frame-by-frame using log-mel spectrogram distance (covers the 4 PSG channels + 2 FIFO Direct Sound channels).
- **Procedural (20%)** — run open-source test ROMs that exercise ARM/Thumb instructions, memory mapping, timers, interrupts, and DMA; pass/fail.

## Rationale

- SSIM captures perceptual playability better than pixel diff.
- Spectrogram distance captures audio correctness perceptually.
- Procedural ROMs isolate hardware subsystems independently of game behavior.

## Alternatives considered

- **Pixel-exact diff** — too strict; penalizes benign timing differences; rejected.
- **Equal weights** — replay is the dominant signal; rejected (follows the eval).

## Consequences

**Positive**
- Comparable to the real eval; each signal independently testable.

**Negative / trade-offs**
- SSIM + spectrogram need a deterministic reference stream from Mesen2 — the oracle must emit frame-by-frame output pinned to a fixed input sequence.

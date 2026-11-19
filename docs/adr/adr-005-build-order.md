# ADR-005: Build in hardware-dependency order (CPU → memory/timers/DMA → PPU → APU → graders → sandbox)

**Date:** 2026-11-18
**Status:** Accepted

## Context

A GBA emulator is a large system; building it in the right order de-risks the project. Each stage depends on the previous, and each produces a testable artifact.

## Decision

Milestone order:

1. ARM7TDMI CPU (ARM + Thumb decode/execute) + register file + CPSR.
2. Memory map + I/O registers + timers + DMA + interrupt controller.
3. PPU (modes 0–5, backgrounds, sprites, scanline render) → boot a test ROM and emit frames.
4. APU (4 PSG channels + 2 FIFO) → emit audio.
5. wasm32 build + browser runner.
6. Grading harness (SSIM, spectrogram, procedural) against the oracle.
7. Docker sandbox + task statement + oracle CLI.
8. End-to-end eval + score.

## Rationale

- CPU first: everything else is driven by CPU writes to I/O registers.
- PPU before APU: visual output is the quickest "is it working" feedback.
- Grading last: needs both the emulator and the oracle to be stable.

## Alternatives considered

- **Big-bang (all modules at once)** — high risk, no incremental feedback; rejected.
- **PPU first** — nothing drives it without the CPU; rejected.

## Consequences

**Positive**
- Each milestone is demoable; risk is front-loaded on the CPU.

**Negative / trade-offs**
- PPU/APU come late; audio accuracy may need iteration after the initial cut.

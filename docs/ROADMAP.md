# Roadmap

## Now

- Phase 0 — docs + scaffold (this).
- Phase 1 — ARM7TDMI CPU core: the highest-risk, highest-value first step.

## Next

- Phase 2 — memory map + I/O + timers + DMA + interrupts.
- Phase 3 — PPU → first visible frames (the quickest "is it working" signal).
- Phase 4 — APU → audio.

## Later

- Phase 5 — wasm32 browser runner.
- Phase 6 — grading harness (SSIM replay, spectrogram audio, procedural ROMs).
- Phase 7 — Docker sandbox + Mesen2 oracle CLI + task statement.
- Phase 8 — end-to-end score + optional leaderboard.

## Ordering rationale

CPU → I/O → PPU → APU follows hardware dependency (ADR-005). Grading and sandbox come last because they depend on a stable emulator + oracle.

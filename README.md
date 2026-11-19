# GBA Eval — a miniature agentic-coding eval environment

A portfolio-scale clone of [Mechanize's GBA Eval](https://github.com/mechanize-ai/gba-eval): build a Rust **Game Boy Advance emulator** (compiled to WebAssembly), then grade it the way frontier coding agents are graded — **SSIM frame replay, audio spectrogram distance, and procedural hardware test ROMs** — against a black-box **Mesen2** oracle.

## Why

GBA Eval makes "environments" and "grading" concrete: a sandboxed environment, a reference oracle, and a multi-signal scoring harness for evaluating a coding agent's output. This project reproduces that *shape* at portfolio scale, demonstrating the systems/emulation + eval/grading skills behind it.

## Structure

- `emulator/` — Rust GBA emulator (ARM7TDMI CPU, PPU, APU, timers, DMA, interrupts), target `wasm32-unknown-unknown`.
- `grader/` — Python grading harness: `replay.py` (SSIM), `audio.py` (log-mel spectrogram), `procedural.py` (test ROMs), `score.py` (weighted aggregate).
- `environment/` — Docker sandbox, task statement (`TASK.md`), and the Mesen2 oracle CLI (`oracle.py`).
- `docs/` — architecture, ADRs, roadmap, phases.

## Status

Planning (Phase 0). See `docs/PHASES.md` and `docs/ROADMAP.md`.

## Docs

- `docs/ARCHITECTURE.md` — system shape + components + data model + honesty boundary.
- `docs/adr/` — immutable decision log (why each choice was made).
- `AGENTS.md` — instructions for AI agents working in this repo.

## Honesty boundary

Portfolio project — a working emulator + grading harness + sandbox. **Not** leaderboard-competitive, **not** cycle-accurate, and **not** affiliated with Mechanize. Mesen2 is invoked as a black-box binary oracle only; its source is never read, and it is not vendored or modified.

## License

MIT.

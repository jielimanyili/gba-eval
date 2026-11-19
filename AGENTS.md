# AGENTS.md

> Instructions for AI coding agents working in this repo.

## Project

A portfolio-scale clone of GBA Eval: a Rust GBA emulator (wasm32 target) graded by a Python harness (SSIM replay + spectrogram audio + procedural ROMs) against a black-box Mesen2 oracle, inside a Docker sandbox. See `README.md` and `docs/ARCHITECTURE.md`.

## Commands

- emulator build: `cd emulator && cargo build --release --target wasm32-unknown-unknown`
- emulator test:  `cd emulator && cargo test`
- grader test:    `cd grader && pytest`
- grader lint:    `cd grader && ruff check .`

## Conventions

- Emulator is dependency-light by design (ADR-001); add crates only with an ADR.
- Grading math lives in `grader/` as pure, unit-tested functions.
- Prose conventions: `docs/style-guide.md`.

## Boundaries (non-negotiable)

- Never read Mesen2 source — treat it as a black-box oracle (ADR-002).
- Never read or write `.env` or any secret.
- Don't change architecture — propose an ADR first (`docs/adr/`).
- Keep the honesty boundary in `docs/ARCHITECTURE.md` truthful; don't overclaim.

## Review gates

Every change must pass: architecture fit · correctness · security · test coverage · readability. See `docs/ai-assisted-development.md`.

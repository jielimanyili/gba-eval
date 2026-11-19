# ADR-001: Build the emulator in Rust targeting wasm32-unknown-unknown

**Date:** 2026-11-18
**Status:** Accepted

## Context

The reference (GBA Eval) tasks models with writing a GBA emulator in Rust compiled to WebAssembly. Reproducing that shape maximizes the portfolio signal (systems programming + wasm) and lets the artifact run in the browser as a live demo. Alternatives: C/C++ (classic emulator choice), or a higher-level language (wrong signal/performance).

## Decision

Write the emulator in Rust, targeting `wasm32-unknown-unknown`, with a thin JS/browser runner. Keep the core dependency-light (no heavyweight crates) so the systems work is visible rather than hidden behind a framework.

## Rationale

- Rust: memory safety + performance + explicit systems control; matches the frontier-agent toolchain we want to demonstrate.
- wasm32: browser demo without a server; mirrors the original's WebAssembly requirement.
- Dependency-light: the point is to show understanding of the GBA hardware, not to glue libraries.

## Alternatives considered

- **C/C++** — traditional, but less modern-signal and more UB risk; rejected.
- **Go/Python core** — wrong performance/abstraction level; rejected.
- **Rust + heavy emulation framework** — hides the learning; rejected.

## Consequences

**Positive**
- Browser-runnable demo; strong systems-programming signal.

**Negative / trade-offs**
- Hand-rolling CPU/PPU/APU is a lot of code; wasm has no std by default (allocator + no_std care needed).

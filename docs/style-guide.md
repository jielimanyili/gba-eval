# Style guide

Non-lintable prose + code conventions.

## Prose

- One idea per paragraph; link over duplicate (see `_template` linking rules).
- ADRs are immutable — supersede, never edit.

## Code

- Rust: `cargo fmt` + `cargo clippy` clean; one module per hardware unit (`cpu.rs`, `ppu.rs`, `apu.rs`, `timer.rs`, `dma.rs`, `irq.rs`, `memory.rs`).
- Python: `ruff` clean; grader math as pure functions with pytest.

## Naming

- Emulator modules mirror GBA hardware names (ARM7TDMI, PPU, APU, DMA, IRQ).
- Grader files mirror the eval's sections (`replay`, `audio`, `procedural`, `score`).

# Phases

| Phase | Goal | Deliverables | Depends on | Status |
|---|---|---|---|---|
| 0 | Plan + scaffold + docs | README, ARCHITECTURE, ADRs, PHASES, ROADMAP, AGENTS | — | 🟡 |
| 1 | ARM7TDMI CPU (ARM + Thumb) | `emulator/src/cpu.rs` + decode/execute tests | 0 | ⬜ |
| 2 | Memory + I/O + timers + DMA + IRQ | `emulator/src/memory.rs`, `timer.rs`, `dma.rs`, `irq.rs` | 1 | ⬜ |
| 3 | PPU (modes 0–5, BG, sprites) → frames | `emulator/src/ppu.rs`; boots a test ROM, renders | 2 | ⬜ |
| 4 | APU (4 PSG + 2 FIFO) → audio | `emulator/src/apu.rs` | 2 | ⬜ |
| 5 | wasm32 build + browser runner | `emulator/www/` + build pipeline | 3, 4 | ⬜ |
| 6 | Grading harness (SSIM/audio/procedural) | `grader/` + pytest | 3, 4 | ⬜ |
| 7 | Docker sandbox + oracle CLI + task statement | `environment/` | 3, 4 | ⬜ |
| 8 | End-to-end eval + score + polish | `grader/score.py`, README demo | 5, 6, 7 | ⬜ |

Status legend: ⬜ planned · 🟡 in progress · ✅ done

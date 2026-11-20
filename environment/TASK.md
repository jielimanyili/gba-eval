# TASK — Build a GBA emulator in Rust (wasm32)

You are given this sandbox. Build a Game Boy Advance emulator in Rust that
compiles to WebAssembly and passes the grading harness.

## Deliverable

A WebAssembly module exporting this exact ABI (all functions `extern "C"`,
`#[no_mangle]`):

| export           | signature                                     | notes                            |
|------------------|-----------------------------------------------|----------------------------------|
| `emu_create`     | `() -> *mut c_void`                           | allocate an emulator instance    |
| `emu_destroy`    | `(*mut c_void) -> ()`                         | free it                          |
| `emu_alloc`      | `(usize) -> *mut u8`                          | allocate `len` bytes in wasm mem |
| `emu_load_rom`   | `(*mut c_void, *const u8, usize) -> ()`       | load a ROM image                 |
| `emu_set_pc`     | `(*mut c_void, u32) -> ()`                    | set PC, reset to ARM state       |
| `emu_step`       | `(*mut c_void, u32) -> ()`                    | step `n` instructions            |
| `emu_render`     | `(*mut c_void, *mut u8) -> ()`                | render 240×160×3 RGB bytes       |
| `emu_fb_size`    | `() -> usize`                                 | = 240 × 160 × 3                  |

## Reference (the oracle)

You may call the `oracle` CLI to obtain the reference frame for a ROM — this is
a black-box wrapper around Mesen2. You may **not** read Mesen2's source.

    oracle run <rom> <out.ppm> --backend mesen2 --steps N

## Grading (ADR-003)

- **Replay (60%)** — SSIM of your frames vs the oracle's, over a fixed input replay.
- **Audio (20%)** — log-mel spectrogram distance (4 PSG + 2 FIFO channels).
- **Procedural (20%)** — pass/fail on bundled test ROMs.

## Dev ROMs

`/work/roms/*.gba` — hand-encoded mode-3 ROMs. Start with `fill_red.gba` (fills
the framebuffer red) and `fill_blue.gba` (fills it blue).

## Constraints

- No internet at runtime.
- Do not read or vendor Mesen2's source (GPL) — it is a binary oracle only.

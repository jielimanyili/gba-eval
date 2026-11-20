# GBA Eval — a miniature agentic-coding eval environment

A portfolio-scale clone of [Mechanize's GBA Eval](https://github.com/mechanize-ai/gba-eval): build a Rust **Game Boy Advance emulator** (compiled to WebAssembly), then grade it the way frontier coding agents are graded — **SSIM frame replay, audio spectrogram distance, and procedural hardware test ROMs** — against a black-box **Mesen2** oracle.

## Why

GBA Eval makes "environments" and "grading" concrete: a sandboxed environment, a reference oracle, and a multi-signal scoring harness for evaluating a coding agent's output. This project reproduces that *shape* at portfolio scale, demonstrating the systems/emulation + eval/grading skills behind it.

## Structure

- `emulator/` — Rust GBA emulator (ARM7TDMI CPU, memory map, PPU bitmap modes 3/4): a `wasm32` library + browser runner, plus a native `gba-run` CLI that dumps a framebuffer as PPM for the grader.
- `grader/` — Python grading harness: `replay.py` (SSIM), `oracle.py` (golden + Mesen2 reference backends), `score.py` (end-to-end), `roms/` (hand-encoded dev ROMs). Audio + procedural signals are planned.
- `environment/` — Docker sandbox (`Dockerfile`), task statement (`TASK.md`), and the Mesen2 oracle wrapper.
- `docs/` — architecture, ADRs, roadmap, phases.

## Status

Working end-to-end on the golden path: CPU → PPU → frames → SSIM grading against a known-correct reference. The Mesen2 oracle backend is written but unverified (no Mesen2 binary in CI). See `docs/PHASES.md`.

## Try it

    # 1. build the candidate frame-dump CLI
    (cd emulator && cargo build --release --bin gba-run)

    # 2. grade a dev ROM (candidate vs golden reference, SSIM)
    python3 grader/score.py grader/roms/fill_red.gba --oracle-backend golden

    # 3. run the harness tests
    python3 grader/test_grader.py

    # 4. browser demo (live emulator + interactive debugger)
    open https://gba-eval.vercel.app

## Grading against Mesen (the real oracle)

The golden path (above) grades against a *known-correct* reference. To grade
against the real **Mesen/MesenCE** oracle, the wiring is complete and verified;
it needs only two user-supplied pieces (both are Nintendo's copyrighted data,
so they're never vendored in this repo):

1. **A GBA BIOS** (`gba_bios.bin`) — Mesen needs the real BIOS firmware to
   boot GBA ROMs. Mount it into the firmware folder (see below).
2. **The Nintendo logo** in the dev-ROM header (`grader/roms/build_roms.py`)
   should be diffed against a known-good homebrew ROM before relying on boot.

    # build the sandbox (MesenCE 2.2.1 + Linux candidate + grader)
    docker build -t gba-eval -f environment/Dockerfile .

    # grade the candidate against Mesen (mount the BIOS)
    docker run --rm \
      -v "$PWD/gba_bios.bin:/root/.config/Mesen2/Firmware/gba_bios.bin" \
      gba-eval python3 /work/grader/score.py \
        /work/roms/fill_red.gba --oracle-backend mesen2 \
        --candidate /work/emulator/target/release/gba-run

The Mesen invocation is fully reverse-engineered and verified against MesenCE
2.2.1 (arg format, the `io`/`os` config gate, the callback API, framebuffer
capture, exit codes) — see the header comments in `grader/mesen2_capture.lua`.

## Docs

- `docs/ARCHITECTURE.md` — system shape + components + data model + honesty boundary.
- `docs/adr/` — immutable decision log (why each choice was made).
- `AGENTS.md` — instructions for AI agents working in this repo.

## Honesty boundary

Portfolio project — a working emulator + grading harness + sandbox. **Not** leaderboard-competitive, **not** cycle-accurate, and **not** affiliated with Mechanize. Mesen2 is invoked as a black-box binary oracle only; its source is never read, and it is not vendored or modified.

## License

MIT.

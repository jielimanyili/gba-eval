#!/usr/bin/env python3
"""Oracle CLI (ADR-002) — emit the reference frame for a ROM.

The grading harness compares the candidate emulator's frame against this
reference (ADR-003). Two backends:

  --backend golden : returns the known-correct frame for the bundled dev ROMs
                     (deterministic by construction — used when Mesen2 isn't
                     installed, and in CI).
  --backend mesen2 : runs the real Mesen2 emulator as a black-box binary
                     (GPL — invoked as an external process, never vendored or
                     modified) to capture the frame via mesen2_capture.lua.

Usage:
  oracle.py run <rom> <out.ppm> [--backend golden|mesen2] [--steps N]
"""

import argparse
import os
import subprocess
import sys

WIDTH = 240
HEIGHT = 160

# Bundled dev ROMs and their known-correct RGB fill color (known by
# construction — these ROMs literally fill the framebuffer with one color).
GOLDEN = {
    "fill_red.gba": (255, 0, 0),
    "fill_blue.gba": (0, 0, 255),
}

HERE = os.path.dirname(os.path.abspath(__file__))


def write_ppm(path: str, rgb) -> None:
    with open(path, "wb") as f:
        f.write(f"P6\n{WIDTH} {HEIGHT}\n255\n".encode())
        f.write(bytes(rgb))


def golden_frame(rgb) -> list:
    return list(rgb) * (WIDTH * HEIGHT)


def golden_run(rom: str, out: str, steps: int) -> None:
    name = os.path.basename(rom)
    if name not in GOLDEN:
        sys.exit(f"no golden reference for {name!r}; known: {sorted(GOLDEN)}")
    frame = golden_frame(GOLDEN[name])
    write_ppm(out, frame)
    print(f"golden oracle: {name} -> {out} ({WIDTH}x{HEIGHT}, {GOLDEN[name]})")


def mesen2_run(rom: str, out: str, steps: int) -> None:
    mesen = os.environ.get("MESEN", "Mesen")
    lua = os.path.join(HERE, "mesen2_capture.lua")
    raw = out + ".raw"  # Mesen writes raw RGB; converted to PPM below.
    env = dict(os.environ)
    env["MESEN_CAPTURE_OUT"] = raw
    # Static dev ROMs: capture a few frames past BIOS boot + fill.
    env["MESEN_CAPTURE_FRAMES"] = os.environ.get("MESEN_CAPTURE_FRAMES", "5")
    # Headless test-runner mode (NOTE: the flag is --testRunner, capital R):
    # Mesen loads the ROM and runs the Lua script at max speed; the script
    # captures the framebuffer and calls emu.stop().
    # (GPL binary, invoked as a subprocess only — never vendored or modified.)
    subprocess.run(
        [mesen, "--testRunner", rom, lua],
        env=env,
        check=True,
    )
    with open(raw, "rb") as f:
        rgb = f.read()
    expected = WIDTH * HEIGHT * 3
    if len(rgb) != expected:
        sys.exit(f"Mesen2 captured {len(rgb)} bytes, expected {expected}")
    write_ppm(out, rgb)
    os.remove(raw)
    print(f"mesen2 oracle: {os.path.basename(rom)} -> {out}")


def main() -> None:
    ap = argparse.ArgumentParser(description="GBA Eval oracle CLI")
    sub = ap.add_subparsers(dest="cmd", required=True)

    run = sub.add_parser("run", help="emit the reference frame for a ROM")
    run.add_argument("rom")
    run.add_argument("out")
    run.add_argument("--backend", choices=["golden", "mesen2"], default="golden")
    run.add_argument("--steps", type=int, default=200000)

    args = ap.parse_args()
    if args.cmd == "run":
        if args.backend == "golden":
            golden_run(args.rom, args.out, args.steps)
        else:
            mesen2_run(args.rom, args.out, args.steps)


if __name__ == "__main__":
    main()

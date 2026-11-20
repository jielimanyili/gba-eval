#!/usr/bin/env python3
"""End-to-end grading (Phase 8): candidate frame vs oracle reference frame.

  1. Run the candidate emulator (gba-run) on the ROM -> candidate frame.
  2. Run the oracle (oracle.py) on the ROM -> reference frame.
  3. SSIM-compare (ADR-003 replay signal, 60% weight).

Usage:
  score.py <rom> [--steps N] [--oracle-backend golden|mesen2] [--candidate BIN]
"""

import argparse
import os
import subprocess
import sys
import tempfile

import replay

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DEFAULT_CANDIDATE = os.path.join(ROOT, "emulator", "target", "release", "gba-run")

WIDTH = 240
HEIGHT = 160
PASS_THRESHOLD = 0.99


def read_ppm(path: str) -> list:
    with open(path, "rb") as f:
        data = f.read()
    parts = data.split(b"\n", 3)
    if parts[0] != b"P6":
        raise ValueError(f"{path} is not a P6 PPM")
    w, h = map(int, parts[1].split())
    maxval = int(parts[2])
    px = parts[3]
    if (w, h) != (WIDTH, HEIGHT) or maxval != 255 or len(px) != WIDTH * HEIGHT * 3:
        raise ValueError(f"unexpected PPM: {w}x{h} max {maxval} ({len(px)} bytes)")
    return list(px)


def grade(rom: str, steps: int, oracle_backend: str, candidate_bin: str):
    """Run candidate + oracle and return (rom_name, ssim_score)."""
    oracle = os.path.join(HERE, "oracle.py")
    with tempfile.TemporaryDirectory() as td:
        cand_path = os.path.join(td, "candidate.ppm")
        ref_path = os.path.join(td, "reference.ppm")

        subprocess.run(
            [candidate_bin, rom, str(steps), cand_path],
            check=True,
        )
        subprocess.run(
            [sys.executable, oracle, "run", rom, ref_path,
             "--backend", oracle_backend, "--steps", str(steps)],
            check=True,
        )

        candidate = read_ppm(cand_path)
        reference = read_ppm(ref_path)
        return os.path.basename(rom), replay.ssim(candidate, reference)


def main() -> None:
    ap = argparse.ArgumentParser(description="GBA Eval end-to-end grader")
    ap.add_argument("rom")
    ap.add_argument("--steps", type=int, default=200000)
    ap.add_argument("--oracle-backend", choices=["golden", "mesen2"], default="golden")
    ap.add_argument("--candidate", default=DEFAULT_CANDIDATE)
    args = ap.parse_args()

    name, ssim = grade(args.rom, args.steps, args.oracle_backend, args.candidate)
    verdict = "PASS" if ssim >= PASS_THRESHOLD else "FAIL"
    print(f"{name}: SSIM {ssim:.6f} -> {verdict} (threshold {PASS_THRESHOLD})")
    sys.exit(0 if verdict == "PASS" else 1)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Tests for the grading harness (Phase 6/8) — dependency-free.

Verifies the pipeline end-to-end via the golden oracle (deterministic
known-correct frames). The Mesen2 backend is exercised separately, when a
Mesen2 binary is present (see environment/Dockerfile).

Prerequisites: build the candidate binary first:
    cargo build --release --bin gba-run   (from emulator/)

Run:
    python3 grader/test_grader.py
"""

import os
import subprocess
import tempfile

import oracle
import replay
import score

HERE = os.path.dirname(os.path.abspath(__file__))
ROMS = os.path.join(HERE, "roms")
CANDIDATE = os.path.join(HERE, "..", "emulator", "target", "release", "gba-run")


def _candidate_frame(rom):
    if not os.path.exists(CANDIDATE):
        raise SystemExit(
            f"candidate binary not found: {CANDIDATE} "
            "(run: cargo build --release --bin gba-run in emulator/)"
        )
    with tempfile.TemporaryDirectory() as td:
        out = os.path.join(td, "c.ppm")
        subprocess.run([CANDIDATE, rom, "200000", out], check=True)
        return score.read_ppm(out)


def _golden(rgb):
    return list(rgb) * (240 * 160)


def test_ssim_identical_and_distinct():
    assert replay.ssim([255] * 100, [255] * 100) > 0.9999
    assert replay.ssim([255] * 100, [0] * 100) < 0.99


def test_golden_oracle_emits_reference():
    with tempfile.TemporaryDirectory() as td:
        out = os.path.join(td, "ref.ppm")
        oracle.golden_run(os.path.join(ROMS, "fill_red.gba"), out, 200000)
        assert score.read_ppm(out) == _golden((255, 0, 0))


def test_candidate_matches_golden_red():
    cand = _candidate_frame(os.path.join(ROMS, "fill_red.gba"))
    assert abs(replay.ssim(cand, _golden((255, 0, 0))) - 1.0) < 1e-6


def test_candidate_matches_golden_blue():
    cand = _candidate_frame(os.path.join(ROMS, "fill_blue.gba"))
    assert abs(replay.ssim(cand, _golden((0, 0, 255))) - 1.0) < 1e-6


def test_grader_discriminates():
    assert replay.ssim(_golden((255, 0, 0)), _golden((0, 0, 255))) < 0.99


def test_score_end_to_end():
    name, s = score.grade(
        os.path.join(ROMS, "fill_red.gba"),
        steps=200000,
        oracle_backend="golden",
        candidate_bin=CANDIDATE,
    )
    assert name == "fill_red.gba"
    assert s > 0.99


def main():
    tests = [v for k, v in sorted(globals().items()) if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
        print(f"PASS {t.__name__}")
    print(f"{len(tests)} tests passed")


if __name__ == "__main__":
    main()

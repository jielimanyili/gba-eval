#!/usr/bin/env python3
"""Generate the bundled dev ROMs (hand-encoded ARM, mode 3).

These are the deterministic test ROMs used by the grading harness. Each fills
the framebuffer with a solid BGR555 color and then spins forever, so "run N
steps then capture a frame" is fully deterministic:

  0x08000000  MOV r3, #0x04000000   ; DISPCNT
  0x08000004  MOV r4, #3
  0x08000008  STRH r4, [r3]         ; mode 3
  0x0800000C  MOV r0, #0x06000000   ; VRAM
  0x08000010  MOV r1, #0x9600       ; 38400 px
  0x08000014  MOV r2, #<color>      ; BGR555
  0x08000018  STRH r2, [r0], #2     ; loop
  0x0800001C  SUBS r1, r1, #1
  0x08000020  BNE 0x08000018
  0x08000024  B 0x08000024          ; spin
"""

import os
import struct

RED = 0x001F  # BGR555: R=31
BLUE = 0x7C00  # BGR555: B=31


def fill(color_instr: int) -> bytes:
    words = [
        0xE3A0_3404,  # MOV r3, #0x04000000
        0xE3A0_4003,  # MOV r4, #3
        0xE1C3_40B0,  # STRH r4, [r3]
        0xE3A0_0406,  # MOV r0, #0x06000000
        0xE3A0_1C96,  # MOV r1, #0x9600
        color_instr,  # MOV r2, #color
        0xE0C0_20B2,  # STRH r2, [r0], #2
        0xE251_1001,  # SUBS r1, r1, #1
        0x1AFF_FFFC,  # BNE loop
        0xEAFF_FFFE,  # B .
    ]
    return b"".join(struct.pack("<I", w) for w in words)


ROMS = {
    "fill_red.gba": fill(0xE3A0_201F),   # MOV r2, #0x001F
    "fill_blue.gba": fill(0xE3A0_2B1F),  # MOV r2, #0x7C00 (imm8 0x1F, ROR 22)
}


def main() -> None:
    out_dir = os.path.dirname(os.path.abspath(__file__))
    for name, data in ROMS.items():
        path = os.path.join(out_dir, name)
        with open(path, "wb") as f:
            f.write(data)
        print(f"wrote {path} ({len(data)} bytes)")


if __name__ == "__main__":
    main()

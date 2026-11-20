//! The full GBA system — CPU + memory, with the fetch-decode-execute loop.

use crate::cpu::Arm7tdmi;
use crate::memory::Memory;
use crate::ppu::Ppu;

pub struct System {
    pub cpu: Arm7tdmi,
    pub mem: Memory,
    pub ppu: Ppu,
}

impl System {
    pub fn new() -> Self {
        Self {
            cpu: Arm7tdmi::new(),
            mem: Memory::new(),
            ppu: Ppu::new(),
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.mem.load_rom(bytes);
    }

    /// Render the current frame into an RGB framebuffer (WIDTH*HEIGHT*3 bytes).
    pub fn render_frame(&self, fb: &mut [u8]) {
        self.ppu.render(&self.mem, fb);
    }

    /// Execute one instruction (ARM or Thumb, per CPSR.T).
    ///
    /// The ARM7TDMI PC pipeline convention: r15 = (address of the executing
    /// instruction) + 8 in ARM state, + 4 in Thumb state. The fetch loop keeps
    /// that invariant and advances the PC only when an instruction didn't
    /// itself write r15 (branch/BX).
    pub fn step(&mut self) {
        if self.cpu.cpsr.thumb {
            let addr = self.cpu.regs.pc() - 4;
            let first = self.mem.read16(addr);
            if (first & 0xF800) == 0xF000 {
                // BL — 32-bit Thumb instruction.
                let second = self.mem.read16(addr + 2);
                self.cpu.thumb_bl(first, second);
            } else {
                let old = self.cpu.regs.pc();
                self.cpu.step_thumb(&mut self.mem, first);
                if self.cpu.regs.pc() == old {
                    self.cpu.regs.set_pc(old + 2);
                }
            }
        } else {
            let addr = self.cpu.regs.pc() - 8;
            let instr = self.mem.read32(addr);
            let old = self.cpu.regs.pc();
            self.cpu.step_arm(&mut self.mem, instr);
            if self.cpu.regs.pc() == old {
                self.cpu.regs.set_pc(old + 4);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_fetches_and_executes_arm() {
        let mut sys = System::new();
        // MOV r0, #42 (0xE3A0002A) little-endian in ROM.
        sys.load_rom(&[0x2A, 0x00, 0xA0, 0xE3]);
        sys.cpu.regs.set_pc(0x0800_0008); // ARM: r15 = addr + 8
        sys.step();
        assert_eq!(sys.cpu.regs.r[0], 42);
        assert_eq!(sys.cpu.regs.pc(), 0x0800_000C); // advanced by 4
    }

    #[test]
    fn cpu_runs_program_and_ppu_renders() {
        let mut sys = System::new();
        // ARM program: set mode 3, write red to VRAM[0], loop forever.
        let program: [u32; 7] = [
            0xE3A0_0404, // MOV r0, #0x04000000
            0xE3A0_1003, // MOV r1, #3
            0xE1C0_10B0, // STRH r1, [r0]   (DISPCNT = mode 3)
            0xE3A0_0406, // MOV r0, #0x06000000
            0xE3A0_101F, // MOV r1, #0x1F   (red)
            0xE1C0_10B0, // STRH r1, [r0]   (VRAM[0] = red)
            0xEAFF_FFFE, // B .             (infinite loop)
        ];
        let mut rom = Vec::new();
        for w in &program {
            rom.extend_from_slice(&w.to_le_bytes());
        }
        sys.load_rom(&rom);
        sys.cpu.regs.set_pc(0x0800_0008);

        for _ in 0..6 {
            sys.step();
        }

        let mut fb = vec![0u8; 240 * 160 * 3];
        sys.render_frame(&mut fb);
        assert_eq!(&fb[0..3], &[255, 0, 0]); // red at (0,0)
    }

    #[test]
    fn fill_screen_demo_renders_red() {
        let mut sys = System::new();
        // Demo program: set mode 3, fill the 240x160 framebuffer red.
        let program: [u32; 10] = [
            0xE3A0_3404, // MOV r3, #0x04000000
            0xE3A0_4003, // MOV r4, #3
            0xE1C3_40B0, // STRH r4, [r3]   (mode 3)
            0xE3A0_0406, // MOV r0, #0x06000000
            0xE3A0_1C96, // MOV r1, #0x9600 (38400 px)
            0xE3A0_201F, // MOV r2, #0x1F   (red)
            0xE0C0_20B2, // loop: STRH r2, [r0], #2
            0xE251_1001, // SUBS r1, r1, #1
            0x1AFF_FFFC, // BNE loop
            0xEAFF_FFFE, // B .
        ];
        let mut rom = Vec::new();
        for w in &program {
            rom.extend_from_slice(&w.to_le_bytes());
        }
        sys.load_rom(&rom);
        sys.cpu.regs.set_pc(0x0800_0008);

        for _ in 0..200_000 {
            sys.step();
        }

        let mut fb = vec![0u8; 240 * 160 * 3];
        sys.render_frame(&mut fb);
        assert_eq!(&fb[0..3], &[255, 0, 0]); // first pixel
        let last = (159 * 240 + 239) * 3;
        assert_eq!(&fb[last..last + 3], &[255, 0, 0]); // last pixel
    }

    #[test]
    fn demo_scrolling_gradient_animates() {
        let mut sys = System::new();
        // Diagonal rainbow: color = x + y + t, scrolling each frame.
        let program: [u32; 18] = [
            0xE3A0_3404, // MOV r3, #0x04000000
            0xE3A0_4003, // MOV r4, #3
            0xE1C3_40B0, // STRH r4, [r3]   (mode 3)
            0xE3A0_5000, // MOV r5, #0      (t)
            0xE3A0_0406, // frame: MOV r0, #0x06000000
            0xE3A0_1000, // MOV r1, #0      (y)
            0xE1A0_6001, // row: MOV r6, r1
            0xE086_6005, // ADD r6, r6, r5  (color = y + t)
            0xE3A0_20F0, // MOV r2, #240    (x)
            0xE0C0_60B2, // px: STRH r6, [r0], #2
            0xE286_6001, // ADD r6, r6, #1  (color += 1 as x++)
            0xE252_2001, // SUBS r2, r2, #1
            0x1AFF_FFFB, // BNE px
            0xE281_1001, // ADD r1, r1, #1
            0xE351_00A0, // CMP r1, #160
            0x1AFF_FFF5, // BNE row
            0xE285_5001, // ADD r5, r5, #1  (scroll)
            0xEAFF_FFF1, // B frame
        ];
        let mut rom = Vec::new();
        for w in &program {
            rom.extend_from_slice(&w.to_le_bytes());
        }
        sys.load_rom(&rom);
        sys.cpu.regs.set_pc(0x0800_0008);

        // Setup (4) + one full frame (154564 instructions) → t = 0.
        for _ in 0..(4 + 154_564) {
            sys.step();
        }
        let mut fb = vec![0u8; 240 * 160 * 3];
        sys.render_frame(&mut fb);
        assert_eq!(&fb[0..3], &[0, 0, 0]); // (0,0) = x+y+t = 0

        // One more frame → t = 1.
        for _ in 0..154_564 {
            sys.step();
        }
        sys.render_frame(&mut fb);
        assert_eq!(&fb[0..3], &[8, 0, 0]); // (0,0) = 1 → R=1
        // Diagonal: color increases along x+y.
        assert_eq!(&fb[1 * 3..1 * 3 + 3], &[16, 0, 0]); // (0,1) → R=2
        assert_eq!(&fb[(1 * 240 + 1) * 3..(1 * 240 + 1) * 3 + 3], &[24, 0, 0]); // (1,1) → R=3
    }

    #[test]
    fn bounce_demo_moves_square() {
        let mut sys = System::new();
        // A bouncing 24×24 red square: clear screen, advance (dx,dy), bounce off
        // the edges with predicated RSB, then redraw via nested loops + MUL
        // addressing. Exercises arithmetic, conditionals, MUL, and LSL #1.
        let program: [u32; 48] = [
            0xE3A0_3404, // MOV r3, #0x04000000
            0xE3A0_4003, // MOV r4, #3
            0xE1C3_40B0, // STRH r4, [r3]   (mode 3)
            0xE3A0_5064, // MOV r5, #100    (x)
            0xE3A0_6046, // MOV r6, #70     (y)
            0xE3A0_7002, // MOV r7, #2      (dx)
            0xE3A0_8001, // MOV r8, #1      (dy)
            0xE3A0_9018, // MOV r9, #24     (size)
            0xE3A0_A01F, // MOV r10, #0x1F  (red)
            0xE3A0_C406, // MOV r12, #0x06000000
            0xE3A0_B0F0, // MOV r11, #0xF0  (240)
            0xE08B_B00B, // ADD r11, r11, r11 (480 = row stride)
            0xE1A0_000C, // frame: MOV r0, r12
            0xE3A0_1C96, // MOV r1, #0x9600 (38400 px)
            0xE3A0_2000, // MOV r2, #0      (black)
            0xE0C0_20B2, // clear: STRH r2, [r0], #2
            0xE251_1001, // SUBS r1, r1, #1
            0x1AFF_FFFC, // BNE clear
            0xE085_4007, // ADD r4, r5, r7  (x + dx)
            0xE354_0000, // CMP r4, #0
            0xB267_7000, // RSBLT r7, r7, #0
            0xB085_4007, // ADDLT r4, r5, r7
            0xE084_4009, // ADD r4, r4, r9  (+ size)
            0xE354_00F0, // CMP r4, #240
            0xC267_7000, // RSBGT r7, r7, #0
            0xE085_5007, // ADD r5, r5, r7  (x += dx)
            0xE086_4008, // ADD r4, r6, r8  (y + dy)
            0xE354_0000, // CMP r4, #0
            0xB268_8000, // RSBLT r8, r8, #0
            0xB086_4008, // ADDLT r4, r6, r8
            0xE084_4009, // ADD r4, r4, r9  (+ size)
            0xE354_00A0, // CMP r4, #160
            0xC268_8000, // RSBGT r8, r8, #0
            0xE086_6008, // ADD r6, r6, r8  (y += dy)
            0xE000_0B96, // MUL r0, r6, r11 (y * 480)
            0xE080_000C, // ADD r0, r0, r12 (+ base)
            0xE080_0085, // ADD r0, r0, r5, LSL #1 (+ x*2)
            0xE1A0_2009, // MOV r2, r9      (rows)
            0xE1A0_1000, // MOV r1, r0      (row addr)
            0xE1A0_3009, // row: MOV r3, r9 (cols)
            0xE1A0_4001, // MOV r4, r1      (pixel addr)
            0xE0C4_A0B2, // col: STRH r10, [r4], #2
            0xE253_3001, // SUBS r3, r3, #1
            0x1AFF_FFFC, // BNE col
            0xE081_100B, // ADD r1, r1, r11 (+ 480)
            0xE252_2001, // SUBS r2, r2, #1
            0x1AFF_FFF7, // BNE row
            0xEAFF_FFDB, // B frame
        ];
        let mut rom = Vec::new();
        for w in &program {
            rom.extend_from_slice(&w.to_le_bytes());
        }
        sys.load_rom(&rom);
        sys.cpu.regs.set_pc(0x0800_0008);

        const PER_FRAME: u32 = 117_073;

        // Setup (12) + one frame → square drawn at (100+dx, 70+dy).
        for _ in 0..(12 + PER_FRAME) {
            sys.step();
        }
        let mut fb = vec![0u8; 240 * 160 * 3];
        sys.render_frame(&mut fb);
        let i = (71 * 240 + 102) * 3; // (102, 71)
        assert_eq!(&fb[i..i + 3], &[255, 0, 0]);
        assert_eq!(&fb[0..3], &[0, 0, 0]); // background black

        // One more frame → square moves again by (dx, dy).
        for _ in 0..PER_FRAME {
            sys.step();
        }
        sys.render_frame(&mut fb);
        let j = (72 * 240 + 104) * 3; // (104, 72)
        assert_eq!(&fb[j..j + 3], &[255, 0, 0]);
        assert_eq!(&fb[i..i + 3], &[0, 0, 0]); // previous spot cleared
    }
}

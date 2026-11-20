//! The full GBA system — CPU + memory, with the fetch-decode-execute loop.

use crate::cpu::Arm7tdmi;
use crate::memory::Memory;

pub struct System {
    pub cpu: Arm7tdmi,
    pub mem: Memory,
}

impl System {
    pub fn new() -> Self {
        Self {
            cpu: Arm7tdmi::new(),
            mem: Memory::new(),
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.mem.load_rom(bytes);
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
            let instr = self.mem.read16(addr);
            let old = self.cpu.regs.pc();
            self.cpu.step_thumb(&mut self.mem, instr);
            if self.cpu.regs.pc() == old {
                self.cpu.regs.set_pc(old + 2);
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
}

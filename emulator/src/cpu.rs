//! ARM7TDMI CPU core — Phase 1.
//!
//! Implements the register file, the program status register (CPSR), and the
//! ARM condition-code evaluation. Instruction decoding/execution is added
//! incrementally in follow-on commits.

/// The 16 general-purpose registers (r0..r15). r15 is the program counter.
#[derive(Default, Clone)]
pub struct RegisterFile {
    pub r: [u32; 16],
}

impl RegisterFile {
    pub fn pc(&self) -> u32 {
        self.r[15]
    }

    pub fn set_pc(&mut self, value: u32) {
        self.r[15] = value;
    }
}

/// ARM condition codes (the top 4 bits of an ARM instruction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    Eq, // Z set
    Ne, // Z clear
    Cs, // C set (unsigned higher or same)
    Cc, // C clear (unsigned lower)
    Mi, // N set
    Pl, // N clear
    Vs, // V set
    Vc, // V clear
    Hi, // C set and Z clear (unsigned higher)
    Ls, // C clear or Z set (unsigned lower or same)
    Ge, // N == V (signed >=)
    Lt, // N != V (signed <)
    Gt, // !Z and N == V (signed >)
    Le, // Z or N != V (signed <=)
    Al, // always
}

impl Cond {
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0xF {
            0x0 => Cond::Eq,
            0x1 => Cond::Ne,
            0x2 => Cond::Cs,
            0x3 => Cond::Cc,
            0x4 => Cond::Mi,
            0x5 => Cond::Pl,
            0x6 => Cond::Vs,
            0x7 => Cond::Vc,
            0x8 => Cond::Hi,
            0x9 => Cond::Ls,
            0xA => Cond::Ge,
            0xB => Cond::Lt,
            0xC => Cond::Gt,
            0xD => Cond::Le,
            _ => Cond::Al,
        }
    }
}

/// Program Status Register — flags, mode, and interrupt/state bits.
#[derive(Debug, Clone, Copy, Default)]
pub struct Psr {
    /// N (31), Z (30), C (29), V (28).
    pub flags: u32,
    /// CPU mode bits 4..0 (User, FIQ, IRQ, SVC, Abort, Undefined, System).
    pub mode: u32,
    /// Thumb state (bit 5).
    pub thumb: bool,
    /// FIQ disable (bit 6).
    pub fiq_disable: bool,
    /// IRQ disable (bit 7).
    pub irq_disable: bool,
}

impl Psr {
    pub fn n(&self) -> bool {
        self.flags & (1 << 31) != 0
    }
    pub fn z(&self) -> bool {
        self.flags & (1 << 30) != 0
    }
    pub fn c(&self) -> bool {
        self.flags & (1 << 29) != 0
    }
    pub fn v(&self) -> bool {
        self.flags & (1 << 28) != 0
    }
}

/// Evaluates whether an ARM condition holds given the current flag bits.
pub fn condition_passes(cond: Cond, flags: u32) -> bool {
    let n = flags & (1 << 31) != 0;
    let z = flags & (1 << 30) != 0;
    let c = flags & (1 << 29) != 0;
    let v = flags & (1 << 28) != 0;
    match cond {
        Cond::Eq => z,
        Cond::Ne => !z,
        Cond::Cs => c,
        Cond::Cc => !c,
        Cond::Mi => n,
        Cond::Pl => !n,
        Cond::Vs => v,
        Cond::Vc => !v,
        Cond::Hi => c && !z,
        Cond::Ls => !c || z,
        Cond::Ge => n == v,
        Cond::Lt => n != v,
        Cond::Gt => !z && (n == v),
        Cond::Le => z || (n != v),
        Cond::Al => true,
    }
}

/// The CPU core.
#[derive(Default)]
pub struct Arm7tdmi {
    pub regs: RegisterFile,
    pub cpsr: Psr,
}

impl Arm7tdmi {
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a single ARM instruction, returning the cycles consumed.
    /// Phase 1: predicate evaluation is in place; decode/execute follows.
    pub fn step_arm(&mut self, instr: u32) -> u32 {
        let cond = Cond::from_bits(instr >> 28);
        if !condition_passes(cond, self.cpsr.flags) {
            // Predicated false — ARM still consumes one S-cycle.
            return 1;
        }
        let _ = instr;
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_codes_match_arm_semantics() {
        let n = 1 << 31;
        let z = 1 << 30;
        let c = 1 << 29;
        let v = 1 << 28;

        assert!(condition_passes(Cond::Eq, z));
        assert!(!condition_passes(Cond::Eq, 0));
        assert!(condition_passes(Cond::Ne, 0));
        assert!(condition_passes(Cond::Al, 0));
        assert!(condition_passes(Cond::Hi, c));
        assert!(!condition_passes(Cond::Hi, c | z));
        assert!(condition_passes(Cond::Ge, n | v));
        assert!(condition_passes(Cond::Lt, n));
        assert!(condition_passes(Cond::Gt, n | v));
        assert!(!condition_passes(Cond::Gt, z | n | v));
    }

    #[test]
    fn pc_is_r15() {
        let mut regs = RegisterFile::default();
        regs.set_pc(0x0800_0000);
        assert_eq!(regs.pc(), 0x0800_0000);
    }
}

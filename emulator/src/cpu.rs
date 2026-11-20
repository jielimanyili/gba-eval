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

/// Barrel-shifter shift types (bits 6..5 of the operand2 field).
const SH_LSL: u32 = 0b00;
const SH_LSR: u32 = 0b01;
const SH_ASR: u32 = 0b10;
const SH_ROR: u32 = 0b11;

/// Shift by an immediate 5-bit amount (with the `#0` special encodings).
fn shift_amount(operand: u32, shift_type: u32, imm5: u32, carry_in: bool) -> (u32, bool) {
    match shift_type {
        SH_LSL => {
            if imm5 == 0 {
                (operand, carry_in)
            } else {
                (operand << imm5, operand & (1 << (32 - imm5)) != 0)
            }
        }
        SH_LSR => {
            let amount = if imm5 == 0 { 32 } else { imm5 };
            if amount == 32 {
                (0, operand & (1 << 31) != 0)
            } else {
                (operand >> amount, operand & (1 << (amount - 1)) != 0)
            }
        }
        SH_ASR => {
            let sign = operand & (1 << 31) != 0;
            let amount = if imm5 == 0 { 32 } else { imm5 };
            if amount >= 32 {
                (if sign { u32::MAX } else { 0 }, sign)
            } else {
                (((operand as i32) >> amount) as u32, operand & (1 << (amount - 1)) != 0)
            }
        }
        SH_ROR => {
            if imm5 == 0 {
                // RRX — rotate right extended through carry.
                let out = (operand >> 1) | ((carry_in as u32) << 31);
                (out, operand & 1 != 0)
            } else {
                (operand.rotate_right(imm5), operand & (1 << (imm5 - 1)) != 0)
            }
        }
        _ => unreachable!(),
    }
}

/// Shift by a register (amount = `rs & 0xFF`).
fn shift_register(operand: u32, shift_type: u32, rs: u32, carry_in: bool) -> (u32, bool) {
    let amount = rs & 0xFF;
    match shift_type {
        SH_LSL => match amount {
            0 => (operand, carry_in),
            1..=31 => (operand << amount, operand & (1 << (32 - amount)) != 0),
            32 => (0, operand & 1 != 0),
            _ => (0, false),
        },
        SH_LSR => match amount {
            0 => (operand, carry_in),
            1..=31 => (operand >> amount, operand & (1 << (amount - 1)) != 0),
            32 => (0, operand & (1 << 31) != 0),
            _ => (0, false),
        },
        SH_ASR => {
            if amount == 0 {
                (operand, carry_in)
            } else {
                let sign = operand & (1 << 31) != 0;
                if amount >= 32 {
                    (if sign { u32::MAX } else { 0 }, sign)
                } else {
                    (((operand as i32) >> amount) as u32, operand & (1 << (amount - 1)) != 0)
                }
            }
        }
        SH_ROR => {
            if amount == 0 {
                (operand, carry_in)
            } else {
                let amt = amount & 0x1F;
                if amt == 0 {
                    (operand, operand & (1 << 31) != 0)
                } else {
                    (operand.rotate_right(amt), operand & (1 << (amt - 1)) != 0)
                }
            }
        }
        _ => unreachable!(),
    }
}

/// Data-processing opcodes (bits 24..21).
const DP_AND: u32 = 0x0;
const DP_EOR: u32 = 0x1;
const DP_SUB: u32 = 0x2;
const DP_RSB: u32 = 0x3;
const DP_ADD: u32 = 0x4;
const DP_ADC: u32 = 0x5;
const DP_SBC: u32 = 0x6;
const DP_RSC: u32 = 0x7;
const DP_TST: u32 = 0x8;
const DP_TEQ: u32 = 0x9;
const DP_CMP: u32 = 0xA;
const DP_CMN: u32 = 0xB;
const DP_ORR: u32 = 0xC;
const DP_MOV: u32 = 0xD;
const DP_BIC: u32 = 0xE;
const DP_MVN: u32 = 0xF;

/// Add with carry: returns (result, carry_out, signed_overflow).
fn add_with_carry(a: u32, b: u32, c: u32) -> (u32, bool, bool) {
    let sum = a as u64 + b as u64 + c as u64;
    let result = sum as u32;
    let carry = sum > 0xFFFF_FFFF;
    let overflow = (sum as i64) < (i32::MIN as i64) || (sum as i64) > (i32::MAX as i64);
    (result, carry, overflow)
}

/// Subtract with borrow-in: returns (result, carry_out (= NOT borrow), signed_overflow).
fn sub_with_borrow(a: u32, b: u32, borrow_in: u32) -> (u32, bool, bool) {
    let diff = a as i64 - b as i64 - borrow_in as i64;
    let result = diff as u32;
    let carry = diff >= 0;
    let overflow = diff < (i32::MIN as i64) || diff > (i32::MAX as i64);
    (result, carry, overflow)
}

/// Sign-extend an N-bit field to a 32-bit signed value.
fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
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
    pub fn step_arm(&mut self, instr: u32) -> u32 {
        let cond = Cond::from_bits(instr >> 28);
        if !condition_passes(cond, self.cpsr.flags) {
            // Predicated false — ARM still consumes one S-cycle.
            return 1;
        }
        // Data-processing class (bits 27..26 == 00) is implemented in Phase 1;
        // all other classes land in later phases.
        if (instr & 0x0C00_0000) == 0 {
            self.exec_data_processing(instr);
        }
        1
    }

    fn exec_data_processing(&mut self, instr: u32) {
        let i = (instr >> 25) & 1;
        let opcode = (instr >> 21) & 0xF;
        let s = (instr >> 20) & 1 == 1;
        let rn = ((instr >> 16) & 0xF) as usize;
        let rd = ((instr >> 12) & 0xF) as usize;
        let rn_val = self.regs.r[rn];

        // Decode operand2.
        let (operand, shifter_carry) = if i == 1 {
            // Immediate: ROR(imm8, 2 * rotate).
            let rotate = ((instr >> 8) & 0xF) * 2;
            let imm8 = instr & 0xFF;
            if rotate == 0 {
                (imm8, self.cpsr.c())
            } else {
                shift_register(imm8, SH_ROR, rotate, self.cpsr.c())
            }
        } else {
            let shift_type = (instr >> 5) & 0x3;
            let rm = (instr & 0xF) as usize;
            let rm_val = self.regs.r[rm];
            if (instr >> 4) & 1 == 0 {
                shift_amount(rm_val, shift_type, (instr >> 7) & 0x1F, self.cpsr.c())
            } else {
                let rs = ((instr >> 8) & 0xF) as usize;
                shift_register(rm_val, shift_type, self.regs.r[rs], self.cpsr.c())
            }
        };

        let c_flag = self.cpsr.c();
        let is_test = matches!(opcode, DP_TST | DP_TEQ | DP_CMP | DP_CMN);
        let sets_flags = s || is_test;

        let result: u32;
        let carry_out: bool;
        let overflow: bool;
        match opcode {
            DP_AND => { result = rn_val & operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_EOR => { result = rn_val ^ operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_ORR => { result = rn_val | operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_BIC => { result = rn_val & !operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_MOV => { result = operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_MVN => { result = !operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_TST => { result = rn_val & operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_TEQ => { result = rn_val ^ operand; carry_out = shifter_carry; overflow = self.cpsr.v(); }
            DP_ADD => { let (r, c, v) = add_with_carry(rn_val, operand, 0); result = r; carry_out = c; overflow = v; }
            DP_ADC => { let (r, c, v) = add_with_carry(rn_val, operand, c_flag as u32); result = r; carry_out = c; overflow = v; }
            DP_CMN => { let (r, c, v) = add_with_carry(rn_val, operand, 0); result = r; carry_out = c; overflow = v; }
            DP_SUB => { let (r, c, v) = sub_with_borrow(rn_val, operand, 0); result = r; carry_out = c; overflow = v; }
            DP_CMP => { let (r, c, v) = sub_with_borrow(rn_val, operand, 0); result = r; carry_out = c; overflow = v; }
            DP_RSB => { let (r, c, v) = sub_with_borrow(operand, rn_val, 0); result = r; carry_out = c; overflow = v; }
            DP_SBC => { let (r, c, v) = sub_with_borrow(rn_val, operand, (!c_flag) as u32); result = r; carry_out = c; overflow = v; }
            DP_RSC => { let (r, c, v) = sub_with_borrow(operand, rn_val, (!c_flag) as u32); result = r; carry_out = c; overflow = v; }
            _ => unreachable!(),
        }

        if sets_flags {
            let mut flags = 0u32;
            if result & (1 << 31) != 0 { flags |= 1 << 31; } // N
            if result == 0 { flags |= 1 << 30; }             // Z
            if carry_out { flags |= 1 << 29; }               // C
            if overflow { flags |= 1 << 28; }                // V
            self.cpsr.flags = flags;
        }

        if !is_test {
            if rd == 15 {
                self.regs.set_pc(result & 0xFFFF_FFFE);
            } else {
                self.regs.r[rd] = result;
            }
        }
    }

    fn set_nzcv(&mut self, result: u32, carry: bool, overflow: bool) {
        let mut flags = 0u32;
        if result & (1 << 31) != 0 {
            flags |= 1 << 31; // N
        }
        if result == 0 {
            flags |= 1 << 30; // Z
        }
        if carry {
            flags |= 1 << 29; // C
        }
        if overflow {
            flags |= 1 << 28; // V
        }
        self.cpsr.flags = flags;
    }

    /// Execute a single Thumb (16-bit) instruction, returning the cycles consumed.
    pub fn step_thumb(&mut self, instr: u16) -> u32 {
        let instr = instr as u32;
        match (instr >> 11) & 0x1F {
            0x00..=0x02 => self.thumb_move_shifted(instr),
            0x03 => self.thumb_add_sub(instr),
            0x04..=0x07 => self.thumb_mov_cmp_add_sub_imm(instr),
            0x08 => {
                if (instr >> 10) & 1 == 0 {
                    self.thumb_alu(instr);
                } else {
                    self.thumb_hi_reg(instr);
                }
            }
            0x14..=0x15 => self.thumb_load_address(instr),
            0x16 => {
                if (instr >> 10) & 1 == 0 {
                    self.thumb_add_sp(instr); // else PUSH/POP — Phase 2
                }
            }
            0x1A..=0x1B => self.thumb_cond_branch(instr),
            0x1C..=0x1D => self.thumb_branch(instr),
            // 0x09 (PC load), 0x0A..0x0F (load/store), 0x10..0x13 (halfword),
            // 0x18..0x19 (LDM/STM), 0x1E..0x1F (BL): deferred to Phase 2 (memory).
            _ => {}
        }
        1
    }

    /// Format 1: move shifted register (`000 op imm5 Rs Rd`).
    fn thumb_move_shifted(&mut self, instr: u32) {
        let op = (instr >> 11) & 0x3; // 00 LSL, 01 LSR, 10 ASR
        let imm5 = (instr >> 6) & 0x1F;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;
        let (result, carry) = shift_amount(self.regs.r[rs], op, imm5, self.cpsr.c());
        self.regs.r[rd] = result;
        self.set_nzcv(result, carry, self.cpsr.v());
    }

    /// Format 2: add/subtract (`00011 I op Rn/offset3 Rs Rd`).
    fn thumb_add_sub(&mut self, instr: u32) {
        let i = (instr >> 10) & 1;
        let op = (instr >> 9) & 1; // 0 ADD, 1 SUB
        let rn = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;
        let operand = if i == 0 {
            self.regs.r[((instr >> 6) & 0x7) as usize]
        } else {
            (instr >> 6) & 0x7
        };
        let (result, carry, overflow) = if op == 0 {
            add_with_carry(self.regs.r[rn], operand, 0)
        } else {
            sub_with_borrow(self.regs.r[rn], operand, 0)
        };
        self.regs.r[rd] = result;
        self.set_nzcv(result, carry, overflow);
    }

    /// Format 3: move/compare/add/subtract immediate (`001 op Rd offset8`).
    fn thumb_mov_cmp_add_sub_imm(&mut self, instr: u32) {
        let op = (instr >> 11) & 0x3; // 00 MOV, 01 CMP, 10 ADD, 11 SUB
        let rd = ((instr >> 8) & 0x7) as usize;
        let imm = instr & 0xFF;
        match op {
            0 => {
                self.regs.r[rd] = imm;
                self.set_nzcv(imm, self.cpsr.c(), self.cpsr.v());
            }
            1 => {
                let (result, carry, overflow) = sub_with_borrow(self.regs.r[rd], imm, 0);
                self.set_nzcv(result, carry, overflow);
            }
            2 => {
                let (result, carry, overflow) = add_with_carry(self.regs.r[rd], imm, 0);
                self.regs.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            3 => {
                let (result, carry, overflow) = sub_with_borrow(self.regs.r[rd], imm, 0);
                self.regs.r[rd] = result;
                self.set_nzcv(result, carry, overflow);
            }
            _ => unreachable!(),
        }
    }

    /// Format 4: ALU operations (`010000 op Rs Rd`).
    fn thumb_alu(&mut self, instr: u32) {
        let op = (instr >> 6) & 0xF;
        let rs = ((instr >> 3) & 0x7) as usize;
        let rd = (instr & 0x7) as usize;
        let a = self.regs.r[rd];
        let b = self.regs.r[rs];
        let c = self.cpsr.c();

        match op {
            0x0 => { let r = a & b; self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); }
            0x1 => { let r = a ^ b; self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); }
            0x2 => { let (r, cy) = shift_register(a, SH_LSL, b, c); self.regs.r[rd] = r; self.set_nzcv(r, cy, self.cpsr.v()); }
            0x3 => { let (r, cy) = shift_register(a, SH_LSR, b, c); self.regs.r[rd] = r; self.set_nzcv(r, cy, self.cpsr.v()); }
            0x4 => { let (r, cy) = shift_register(a, SH_ASR, b, c); self.regs.r[rd] = r; self.set_nzcv(r, cy, self.cpsr.v()); }
            0x5 => { let (r, cy, ov) = add_with_carry(a, b, c as u32); self.regs.r[rd] = r; self.set_nzcv(r, cy, ov); }
            0x6 => { let (r, cy, ov) = sub_with_borrow(a, b, (!c) as u32); self.regs.r[rd] = r; self.set_nzcv(r, cy, ov); }
            0x7 => { let (r, cy) = shift_register(a, SH_ROR, b, c); self.regs.r[rd] = r; self.set_nzcv(r, cy, self.cpsr.v()); }
            0x8 => { let r = a & b; self.set_nzcv(r, c, self.cpsr.v()); } // TST
            0x9 => { let (r, cy, ov) = sub_with_borrow(0, b, 0); self.regs.r[rd] = r; self.set_nzcv(r, cy, ov); } // NEG
            0xA => { let (r, cy, ov) = sub_with_borrow(a, b, 0); self.set_nzcv(r, cy, ov); } // CMP
            0xB => { let (r, cy, ov) = add_with_carry(a, b, 0); self.set_nzcv(r, cy, ov); } // CMN
            0xC => { let r = a | b; self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); }
            0xD => { let r = a.wrapping_mul(b); self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); } // MUL
            0xE => { let r = a & !b; self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); }
            0xF => { let r = !b; self.regs.r[rd] = r; self.set_nzcv(r, c, self.cpsr.v()); } // MVN
            _ => unreachable!(),
        }
    }

    /// Format 5: hi-register operations / branch exchange (`010001 op H1 H2 Rs Rd`).
    fn thumb_hi_reg(&mut self, instr: u32) {
        let op = (instr >> 8) & 0x3; // 00 ADD, 01 CMP, 10 MOV, 11 BX
        let rs = ((((instr >> 6) & 1) << 3) | ((instr >> 3) & 0x7)) as usize;
        let rd = ((((instr >> 7) & 1) << 3) | (instr & 0x7)) as usize;
        match op {
            0 => self.regs.r[rd] = self.regs.r[rd].wrapping_add(self.regs.r[rs]), // flags unchanged
            1 => {
                let (r, cy, ov) = sub_with_borrow(self.regs.r[rd], self.regs.r[rs], 0);
                self.set_nzcv(r, cy, ov);
            }
            2 => self.regs.r[rd] = self.regs.r[rs], // flags unchanged
            3 => {
                let target = self.regs.r[rs];
                self.cpsr.thumb = target & 1 != 0;
                self.regs.set_pc(target & 0xFFFF_FFFE);
            }
            _ => unreachable!(),
        }
    }

    /// Format 12: load address (`1010 SP Rd word8`) — Rd = (PC|SP) + (word8 << 2).
    fn thumb_load_address(&mut self, instr: u32) {
        let sp = (instr >> 11) & 1;
        let rd = ((instr >> 8) & 0x7) as usize;
        let word8 = instr & 0xFF;
        let base = if sp == 1 {
            self.regs.r[13]
        } else {
            self.regs.pc() & 0xFFFF_FFFC
        };
        self.regs.r[rd] = base.wrapping_add(word8 << 2);
    }

    /// Format 13: add/subtract to stack pointer (`1011 0000 S word7`).
    fn thumb_add_sp(&mut self, instr: u32) {
        let s = (instr >> 7) & 1;
        let delta = (instr & 0x7F) << 2;
        self.regs.r[13] = if s == 1 {
            self.regs.r[13].wrapping_sub(delta)
        } else {
            self.regs.r[13].wrapping_add(delta)
        };
    }

    /// Format 16: conditional branch (`1101 cond soffset8`).
    fn thumb_cond_branch(&mut self, instr: u32) {
        let cond_bits = (instr >> 8) & 0xF;
        if cond_bits == 0xF {
            return; // SWI — deferred (Phase 2+).
        }
        let cond = Cond::from_bits(cond_bits);
        if condition_passes(cond, self.cpsr.flags) {
            let offset = sign_extend(instr & 0xFF, 8);
            let target = (self.regs.pc() as i32).wrapping_add(offset << 1) as u32;
            self.regs.set_pc(target);
        }
    }

    /// Format 18: unconditional branch (`11100 offset11`).
    fn thumb_branch(&mut self, instr: u32) {
        let offset = sign_extend(instr & 0x7FF, 11);
        let target = (self.regs.pc() as i32).wrapping_add(offset << 1) as u32;
        self.regs.set_pc(target);
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

    /// Build a register-form data-processing instruction (Rm direct, LSL #0).
    fn dp(opcode: u32, s: bool, rn: u32, rd: u32, rm: u32) -> u32 {
        (0xE << 28) | (opcode << 21) | ((s as u32) << 20) | (rn << 16) | (rd << 12) | rm
    }

    #[test]
    fn add_sets_flags() {
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 5;
        cpu.regs.r[2] = 3;
        cpu.step_arm(dp(DP_ADD, true, 1, 0, 2));
        assert_eq!(cpu.regs.r[0], 8);
        assert!(!cpu.cpsr.n() && !cpu.cpsr.z() && !cpu.cpsr.c() && !cpu.cpsr.v());
    }

    #[test]
    fn sub_borrow_sets_carry_clear() {
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 3;
        cpu.regs.r[2] = 5;
        cpu.step_arm(dp(DP_SUB, true, 1, 0, 2));
        assert_eq!(cpu.regs.r[0], 0xFFFF_FFFE); // 3 - 5 = -2
        assert!(cpu.cpsr.n() && !cpu.cpsr.z() && !cpu.cpsr.c());
    }

    #[test]
    fn sub_no_borrow_sets_carry() {
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 5;
        cpu.regs.r[2] = 3;
        cpu.step_arm(dp(DP_SUB, true, 1, 0, 2));
        assert_eq!(cpu.regs.r[0], 2);
        assert!(cpu.cpsr.c());
    }

    #[test]
    fn cmp_does_not_write_rd() {
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 5;
        cpu.regs.r[2] = 3;
        cpu.regs.r[0] = 0xDEAD_BEEF;
        cpu.step_arm(dp(DP_CMP, false, 1, 0, 2)); // CMP r1, r2 (Rd untouched)
        assert_eq!(cpu.regs.r[0], 0xDEAD_BEEF);
        assert!(!cpu.cpsr.z());
    }

    #[test]
    fn mov_with_lsl_immediate() {
        // MOV r0, r1, LSL #4
        let instr = (0xE << 28) | (DP_MOV << 21) | (0 << 12) | (SH_LSL << 5) | (4 << 7) | 1;
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 0x0000_0001;
        cpu.step_arm(instr);
        assert_eq!(cpu.regs.r[0], 0x10);
    }

    #[test]
    fn rrx_through_carry() {
        // MOV r0, r1, ROR #0  ==  RRX
        let instr = (0xE << 28) | (DP_MOV << 21) | (0 << 12) | (SH_ROR << 5) | (0 << 7) | 1;
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 0b10;
        cpu.cpsr.flags = 1 << 29; // C = 1
        cpu.step_arm(instr);
        assert_eq!(cpu.regs.r[0], 0b1 | (1 << 31));
    }

    #[test]
    fn immediate_operand_rotated() {
        // MOV r0, #0xFF000000  (imm8 = 0xFF, rotate field = 4 → ROR by 8)
        let instr = (0xE << 28) | (DP_MOV << 21) | (1 << 25) | (4 << 8) | 0xFF;
        let mut cpu = Arm7tdmi::new();
        cpu.step_arm(instr);
        assert_eq!(cpu.regs.r[0], 0xFF00_0000);
    }

    #[test]
    fn thumb_add_register() {
        // ADD r0, r1, r2  (00011 0 0 010 001 000)
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 5;
        cpu.regs.r[2] = 3;
        cpu.step_thumb(0x1888);
        assert_eq!(cpu.regs.r[0], 8);
    }

    #[test]
    fn thumb_mov_immediate() {
        // MOV r0, #42  (001 00 000 0010_1010)
        let mut cpu = Arm7tdmi::new();
        cpu.step_thumb(0x202A);
        assert_eq!(cpu.regs.r[0], 42);
    }

    #[test]
    fn thumb_lsl_immediate() {
        // LSL r0, r1, #4  (000 00 00100 001 000)
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 1;
        cpu.step_thumb(0x0108);
        assert_eq!(cpu.regs.r[0], 0x10);
    }

    #[test]
    fn thumb_bx_switches_to_thumb_target() {
        // BX r1  (010001 11 0 0 001 000)
        let mut cpu = Arm7tdmi::new();
        cpu.regs.r[1] = 0x0800_0001; // bit0 = 1 → Thumb state
        cpu.cpsr.thumb = false;
        cpu.step_thumb(0x4708);
        assert_eq!(cpu.regs.pc(), 0x0800_0000);
        assert!(cpu.cpsr.thumb);
    }

    #[test]
    fn thumb_conditional_branch_taken_when_z() {
        // BEQ #4  (1101 0000 0000 0100)
        let mut cpu = Arm7tdmi::new();
        cpu.regs.set_pc(0x100);
        cpu.cpsr.flags = 1 << 30; // Z set
        cpu.step_thumb(0xD004);
        assert_eq!(cpu.regs.pc(), 0x108); // 0x100 + (4 << 1)
    }
}

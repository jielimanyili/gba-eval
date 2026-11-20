//! GBA memory map — Phase 2.
//!
//! A flat byte-addressable address space with the Game Boy Advance's regions:
//! BIOS, EWRAM, IWRAM, I/O registers, palette RAM, VRAM, OAM, Game Pak ROM,
//! and SRAM. Reads/writes are little-endian (the GBA is a little-endian ARM).

const BIOS_BASE: u32 = 0x0000_0000;
const BIOS_SIZE: usize = 0x4000; // 16 KB
const EWRAM_BASE: u32 = 0x0200_0000;
const EWRAM_SIZE: usize = 0x4_0000; // 256 KB
const IWRAM_BASE: u32 = 0x0300_0000;
const IWRAM_SIZE: usize = 0x8000; // 32 KB
const IO_BASE: u32 = 0x0400_0000;
const IO_SIZE: usize = 0x400; // 1 KB
const PALETTE_BASE: u32 = 0x0500_0000;
const PALETTE_SIZE: usize = 0x400; // 1 KB
const VRAM_BASE: u32 = 0x0600_0000;
const VRAM_SIZE: usize = 0x1_8000; // 96 KB
const OAM_BASE: u32 = 0x0700_0000;
const OAM_SIZE: usize = 0x400; // 1 KB
const ROM_BASE: u32 = 0x0800_0000;
const SRAM_BASE: u32 = 0x0E00_0000;
const SRAM_SIZE: usize = 0x1_0000; // 64 KB

pub struct Memory {
    bios: Vec<u8>,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    io: Vec<u8>,
    palette: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
    rom: Vec<u8>,
    sram: Vec<u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            bios: vec![0; BIOS_SIZE],
            ewram: vec![0; EWRAM_SIZE],
            iwram: vec![0; IWRAM_SIZE],
            io: vec![0; IO_SIZE],
            palette: vec![0; PALETTE_SIZE],
            vram: vec![0; VRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            rom: Vec::new(),
            sram: vec![0; SRAM_SIZE],
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the Game Pak ROM image (up to 32 MB).
    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.rom = bytes.to_vec();
    }

    /// Load the BIOS (16 KB); extra bytes are ignored.
    pub fn load_bios(&mut self, bytes: &[u8]) {
        let n = bytes.len().min(BIOS_SIZE);
        self.bios[..n].copy_from_slice(&bytes[..n]);
    }

    pub fn read8(&self, addr: u32) -> u8 {
        match addr {
            0x0000_0000..=0x01FF_FFFF => self.bios[(addr - BIOS_BASE) as usize % BIOS_SIZE],
            0x0200_0000..=0x02FF_FFFF => self.ewram[(addr - EWRAM_BASE) as usize % EWRAM_SIZE],
            0x0300_0000..=0x03FF_FFFF => self.iwram[(addr - IWRAM_BASE) as usize % IWRAM_SIZE],
            0x0400_0000..=0x04FF_FFFF => self.io[(addr - IO_BASE) as usize % IO_SIZE],
            0x0500_0000..=0x05FF_FFFF => self.palette[(addr - PALETTE_BASE) as usize % PALETTE_SIZE],
            0x0600_0000..=0x06FF_FFFF => self.vram[(addr - VRAM_BASE) as usize % VRAM_SIZE],
            0x0700_0000..=0x07FF_FFFF => self.oam[(addr - OAM_BASE) as usize % OAM_SIZE],
            0x0800_0000..=0x0DFF_FFFF => {
                let off = (addr - ROM_BASE) as usize;
                if off < self.rom.len() {
                    self.rom[off]
                } else {
                    0 // open bus beyond loaded ROM
                }
            }
            0x0E00_0000..=0x0EFF_FFFF => self.sram[(addr - SRAM_BASE) as usize % SRAM_SIZE],
            _ => 0, // open bus (simplified)
        }
    }

    pub fn read16(&self, addr: u32) -> u16 {
        self.read8(addr) as u16 | ((self.read8(addr + 1) as u16) << 8)
    }

    pub fn read32(&self, addr: u32) -> u32 {
        self.read8(addr) as u32
            | ((self.read8(addr + 1) as u32) << 8)
            | ((self.read8(addr + 2) as u32) << 16)
            | ((self.read8(addr + 3) as u32) << 24)
    }

    pub fn write8(&mut self, addr: u32, value: u8) {
        match addr {
            0x0000_0000..=0x01FF_FFFF => {} // BIOS is read-only
            0x0200_0000..=0x02FF_FFFF => {
                self.ewram[(addr - EWRAM_BASE) as usize % EWRAM_SIZE] = value
            }
            0x0300_0000..=0x03FF_FFFF => {
                self.iwram[(addr - IWRAM_BASE) as usize % IWRAM_SIZE] = value
            }
            0x0400_0000..=0x04FF_FFFF => self.io[(addr - IO_BASE) as usize % IO_SIZE] = value,
            0x0500_0000..=0x05FF_FFFF => self.palette[(addr - PALETTE_BASE) as usize % PALETTE_SIZE] = value,
            0x0600_0000..=0x06FF_FFFF => {
                self.vram[(addr - VRAM_BASE) as usize % VRAM_SIZE] = value
            }
            0x0700_0000..=0x07FF_FFFF => self.oam[(addr - OAM_BASE) as usize % OAM_SIZE] = value,
            0x0800_0000..=0x0DFF_FFFF => {} // ROM is read-only for the MVP
            0x0E00_0000..=0x0EFF_FFFF => self.sram[(addr - SRAM_BASE) as usize % SRAM_SIZE] = value,
            _ => {} // unmapped
        }
    }

    pub fn write16(&mut self, addr: u32, value: u16) {
        self.write8(addr, value as u8);
        self.write8(addr + 1, (value >> 8) as u8);
    }

    pub fn write32(&mut self, addr: u32, value: u32) {
        self.write8(addr, value as u8);
        self.write8(addr + 1, (value >> 8) as u8);
        self.write8(addr + 2, (value >> 16) as u8);
        self.write8(addr + 3, (value >> 24) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_iwram() {
        let mut mem = Memory::new();
        mem.write8(0x0300_0000, 0xAB);
        assert_eq!(mem.read8(0x0300_0000), 0xAB);
    }

    #[test]
    fn read16_is_little_endian() {
        let mut mem = Memory::new();
        mem.write16(0x0200_0000, 0x1234);
        assert_eq!(mem.read16(0x0200_0000), 0x1234);
        assert_eq!(mem.read8(0x0200_0000), 0x34);
        assert_eq!(mem.read8(0x0200_0001), 0x12);
    }

    #[test]
    fn read32_is_little_endian() {
        let mut mem = Memory::new();
        mem.write32(0x0300_0000, 0xDEAD_BEEF);
        assert_eq!(mem.read32(0x0300_0000), 0xDEAD_BEEF);
        assert_eq!(mem.read8(0x0300_0000), 0xEF);
        assert_eq!(mem.read8(0x0300_0003), 0xDE);
    }

    #[test]
    fn rom_is_loaded_and_readable() {
        let mut mem = Memory::new();
        mem.load_rom(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(mem.read32(0x0800_0000), 0x0403_0201);
        assert_eq!(mem.read8(0x0800_0004), 0); // beyond loaded ROM
    }

    #[test]
    fn bios_is_read_only() {
        let mut mem = Memory::new();
        mem.load_bios(&[0x11, 0x22]);
        assert_eq!(mem.read8(0x0000_0000), 0x11);
        mem.write8(0x0000_0000, 0xFF);
        assert_eq!(mem.read8(0x0000_0000), 0x11); // unchanged
    }

    #[test]
    fn iwram_mirrors_within_region() {
        let mut mem = Memory::new();
        mem.write8(0x0300_0000, 0x77);
        // 0x0300_8000 is a mirror of 0x0300_0000 (modulo IWRAM_SIZE).
        assert_eq!(mem.read8(0x0300_8000), 0x77);
    }

    #[test]
    fn unmapped_read_returns_zero() {
        let mem = Memory::new();
        assert_eq!(mem.read8(0x0F00_0000), 0);
    }
}

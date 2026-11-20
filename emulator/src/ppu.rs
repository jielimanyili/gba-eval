//! Picture Processing Unit (PPU) — Phase 3.
//!
//! Renders the current frame from VRAM + palette + I/O registers into an RGB
//! framebuffer. Bitmap modes 3 and 4 come first; tile modes and sprites follow.

use crate::memory::Memory;

/// Display dimensions: 240×160.
pub const WIDTH: usize = 240;
pub const HEIGHT: usize = 160;

pub struct Ppu;

impl Ppu {
    pub fn new() -> Self {
        Ppu
    }

    /// Render the current frame into `fb` (WIDTH*HEIGHT*3 RGB bytes).
    pub fn render(&self, mem: &Memory, fb: &mut [u8]) {
        debug_assert!(fb.len() >= WIDTH * HEIGHT * 3);
        let dispcnt = mem.read16(0x0400_0000);
        match dispcnt & 0x7 {
            3 => self.render_mode3(mem, fb),
            4 => self.render_mode4(mem, fb, dispcnt),
            // Tile modes 0-2, mode 5, and sprites: not yet implemented.
            _ => {
                for px in fb.iter_mut() {
                    *px = 0;
                }
            }
        }
    }

    fn render_mode3(&self, mem: &Memory, fb: &mut [u8]) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = mem.read16(0x0600_0000 + ((y * WIDTH + x) * 2) as u32);
                let (r, g, b) = bgr555(color);
                let i = (y * WIDTH + x) * 3;
                fb[i] = r;
                fb[i + 1] = g;
                fb[i + 2] = b;
            }
        }
    }

    fn render_mode4(&self, mem: &Memory, fb: &mut [u8], dispcnt: u16) {
        let frame = ((dispcnt >> 4) & 1) as u32; // display frame select
        let base = 0x0600_0000 + frame * 0xA000;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = mem.read8(base + (y * WIDTH + x) as u32) as u32;
                let color = mem.read16(0x0500_0000 + idx * 2);
                let (r, g, b) = bgr555(color);
                let i = (y * WIDTH + x) * 3;
                fb[i] = r;
                fb[i + 1] = g;
                fb[i + 2] = b;
            }
        }
    }
}

/// Convert a BGR555 color to 8-bit RGB channels.
fn bgr555(color: u16) -> (u8, u8, u8) {
    let r5 = (color & 0x1F) as u8;
    let g5 = ((color >> 5) & 0x1F) as u8;
    let b5 = ((color >> 10) & 0x1F) as u8;
    (
        (r5 << 3) | (r5 >> 2),
        (g5 << 3) | (g5 >> 2),
        (b5 << 3) | (b5 >> 2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode3_renders_direct_color() {
        let mut mem = Memory::new();
        mem.write16(0x0400_0000, 3); // mode 3
        mem.write16(0x0600_0000, 0x001F); // red (BGR555 R=31)
        let mut fb = vec![0u8; WIDTH * HEIGHT * 3];
        Ppu.render(&mem, &mut fb);
        assert_eq!(&fb[0..3], &[255, 0, 0]);
    }

    #[test]
    fn mode4_renders_palette() {
        let mut mem = Memory::new();
        mem.write16(0x0400_0000, 4); // mode 4
        mem.write16(0x0500_0002, 0x03E0); // palette entry 1 = green
        mem.write8(0x0600_0000, 1); // pixel (0,0) uses palette index 1
        let mut fb = vec![0u8; WIDTH * HEIGHT * 3];
        Ppu.render(&mem, &mut fb);
        assert_eq!(&fb[0..3], &[0, 255, 0]);
    }

    #[test]
    fn unimplemented_mode_renders_black() {
        let mut mem = Memory::new();
        mem.write16(0x0400_0000, 0); // mode 0 (tile) — not yet implemented
        let mut fb = vec![7u8; WIDTH * HEIGHT * 3];
        Ppu.render(&mem, &mut fb);
        assert_eq!(&fb[0..3], &[0, 0, 0]);
    }
}

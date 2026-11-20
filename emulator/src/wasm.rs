//! WebAssembly bindings — expose the emulator to the browser (Phase 5).
//!
//! A minimal, dependency-free `extern "C"` interface. JS allocates a buffer
//! via `emu_alloc`, loads a ROM, steps the CPU, and renders the framebuffer.

use crate::ppu::{HEIGHT, WIDTH};
use crate::system::System;

const FB_SIZE: usize = WIDTH * HEIGHT * 3;

/// Create an emulator, returning an opaque pointer.
#[no_mangle]
pub extern "C" fn emu_create() -> *mut System {
    Box::into_raw(Box::new(System::new()))
}

/// Free an emulator.
#[no_mangle]
pub unsafe extern "C" fn emu_destroy(emu: *mut System) {
    if !emu.is_null() {
        drop(Box::from_raw(emu));
    }
}

/// Allocate `len` bytes in wasm linear memory (for a ROM image / framebuffer).
#[no_mangle]
pub extern "C" fn emu_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Load a ROM image from `ptr` (`len` bytes) into the emulator.
#[no_mangle]
pub unsafe extern "C" fn emu_load_rom(emu: *mut System, ptr: *const u8, len: usize) {
    let rom = std::slice::from_raw_parts(ptr, len);
    (*emu).load_rom(rom);
}

/// Set the CPU program counter and reset to ARM state.
#[no_mangle]
pub unsafe extern "C" fn emu_set_pc(emu: *mut System, pc: u32) {
    (*emu).cpu.regs.set_pc(pc);
    (*emu).cpu.cpsr.thumb = false;
}

/// Step the CPU `n` times.
#[no_mangle]
pub unsafe extern "C" fn emu_step(emu: *mut System, n: u32) {
    for _ in 0..n {
        (*emu).step();
    }
}

/// Render the current frame into `fb` (WIDTH*HEIGHT*3 RGB bytes).
#[no_mangle]
pub unsafe extern "C" fn emu_render(emu: *mut System, fb: *mut u8) {
    let fb = std::slice::from_raw_parts_mut(fb, FB_SIZE);
    (*emu).render_frame(fb);
}

/// The framebuffer size in bytes (240 * 160 * 3).
#[no_mangle]
pub extern "C" fn emu_fb_size() -> usize {
    FB_SIZE
}

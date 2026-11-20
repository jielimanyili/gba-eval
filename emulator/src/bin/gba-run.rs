//! Native CLI: run a ROM for N steps and dump the framebuffer as a PPM image.
//!
//! This is the "candidate emulator" side of the grading pipeline (ADR-002):
//! the grader runs this binary to produce a candidate frame, then compares it
//! against the oracle's reference frame with SSIM (ADR-003).

use std::env;
use std::fs;

use gba_emulator::ppu::{HEIGHT, WIDTH};
use gba_emulator::system::System;

fn parse_addr(s: &str) -> u32 {
    let t = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(t, 16).expect("expected a hex address")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: gba-run <rom> <steps> <out.ppm> [--pc ADDR]");
        eprintln!("  runs the ROM for <steps> instructions and writes a PPM framebuffer.");
        eprintln!("  defaults to --pc 0x080000C8 (program at 0x080000C0 after the header).");
        std::process::exit(2);
    }

    let rom = fs::read(&args[1]).expect("failed to read ROM");
    let steps: u32 = args[2].parse().expect("steps must be an integer");
    let out = &args[3];

    // Dev ROMs ship a 192-byte GBA header; the program lives at 0x080000C0.
    // r15 = addr + 8, so the default start is 0x080000C8. Raw (headerless)
    // ROMs can override with --pc 0x08000008.
    let mut pc = 0x0800_00C8u32;
    let mut i = 4;
    while i < args.len() {
        if args[i] == "--pc" {
            pc = parse_addr(&args[i + 1]);
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut sys = System::new();
    sys.load_rom(&rom);
    sys.cpu.regs.set_pc(pc);

    for _ in 0..steps {
        sys.step();
    }

    let mut fb = vec![0u8; WIDTH * HEIGHT * 3];
    sys.render_frame(&mut fb);

    // PPM P6 (binary RGB) — trivial for the grader to parse.
    let mut ppm = format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).into_bytes();
    ppm.extend_from_slice(&fb);
    fs::write(out, &ppm).expect("failed to write PPM");
}

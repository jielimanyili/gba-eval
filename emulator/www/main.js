const WIDTH = 240, HEIGHT = 160, FB_SIZE = WIDTH * HEIGHT * 3;

// Demo ROM: a small ARM program that sets mode 3 and fills the screen red.
//   MOV r3, #0x04000000 ; MOV r4, #3 ; STRH r4, [r3]
//   MOV r0, #0x06000000 ; MOV r1, #0x9600 ; MOV r2, #0x1F
//   loop: STRH r2, [r0], #2 ; SUBS r1, r1, #1 ; BNE loop ; B .
const DEMO_ROM = new Uint8Array([
  0x04, 0x34, 0xA0, 0xE3, // MOV r3, #0x04000000
  0x03, 0x40, 0xA0, 0xE3, // MOV r4, #3
  0xB0, 0x40, 0xC3, 0xE1, // STRH r4, [r3]   (mode 3)
  0x06, 0x04, 0xA0, 0xE3, // MOV r0, #0x06000000
  0x96, 0x1C, 0xA0, 0xE3, // MOV r1, #0x9600 (38400 px)
  0x1F, 0x20, 0xA0, 0xE3, // MOV r2, #0x1F   (red)
  0xB2, 0x20, 0xC0, 0xE0, // loop: STRH r2, [r0], #2
  0x01, 0x10, 0x51, 0xE2, // SUBS r1, r1, #1
  0xFC, 0xFF, 0xFF, 0x1A, // BNE loop
  0xFE, 0xFF, 0xFF, 0xEA, // B .
]);

async function main() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch('gba_emulator.wasm'), {});
  const w = instance.exports;

  const emu = w.emu_create();

  // Load the ROM into wasm memory.
  const romPtr = w.emu_alloc(DEMO_ROM.length);
  new Uint8Array(w.memory.buffer, romPtr, DEMO_ROM.length).set(DEMO_ROM);
  w.emu_load_rom(emu, romPtr, DEMO_ROM.length);

  // Reset PC to the ROM entry point (ARM state).
  w.emu_set_pc(emu, 0x08000008);

  // Run enough instructions to fill the screen (6 setup + 38400 * 3 loop).
  w.emu_step(emu, 200000);

  // Render to a framebuffer.
  const fbPtr = w.emu_alloc(FB_SIZE);
  w.emu_render(emu, fbPtr);
  const fb = new Uint8Array(w.memory.buffer, fbPtr, FB_SIZE);

  // Draw to the canvas.
  const canvas = document.getElementById('screen');
  canvas.width = WIDTH;
  canvas.height = HEIGHT;
  const ctx = canvas.getContext('2d');
  const img = ctx.createImageData(WIDTH, HEIGHT);
  for (let i = 0; i < WIDTH * HEIGHT; i++) {
    img.data[i * 4 + 0] = fb[i * 3 + 0];
    img.data[i * 4 + 1] = fb[i * 3 + 1];
    img.data[i * 4 + 2] = fb[i * 3 + 2];
    img.data[i * 4 + 3] = 255;
  }
  ctx.putImageData(img, 0, 0);

  w.emu_destroy(emu);
}

main();

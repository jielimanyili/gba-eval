const WIDTH = 240, HEIGHT = 160, FB_SIZE = WIDTH * HEIGHT * 3;

// Animated vertical gradient: color = row + offset, scrolling every frame.
//   MOV r3, #0x04000000 ; MOV r4, #3 ; STRH r4, [r3]   (mode 3)
//   MOV r5, #0                                        (offset)
//   frame: MOV r0, #0x06000000 ; MOV r1, #0           (y)
//   row:   MOV r6, r1 ; ADD r6, r6, r5                (color = y + offset)
//          MOV r2, #240                               (x)
//   px:    STRH r6, [r0], #2 ; SUBS r2, r2, #1 ; BNE px
//          ADD r1, r1, #1 ; CMP r1, #160 ; BNE row
//          ADD r5, r5, #1                             (scroll)
//          B frame
const DEMO_ROM = new Uint8Array([
  0x04, 0x34, 0xA0, 0xE3, // MOV r3, #0x04000000
  0x03, 0x40, 0xA0, 0xE3, // MOV r4, #3
  0xB0, 0x40, 0xC3, 0xE1, // STRH r4, [r3]   (mode 3)
  0x00, 0x50, 0xA0, 0xE3, // MOV r5, #0      (offset)
  0x06, 0x04, 0xA0, 0xE3, // frame: MOV r0, #0x06000000
  0x00, 0x10, 0xA0, 0xE3, // MOV r1, #0      (y)
  0x01, 0x60, 0xA0, 0xE1, // row: MOV r6, r1
  0x05, 0x60, 0x86, 0xE0, // ADD r6, r6, r5  (color = y + offset)
  0xF0, 0x20, 0xA0, 0xE3, // MOV r2, #240    (x)
  0xB2, 0x60, 0xC0, 0xE0, // px: STRH r6, [r0], #2
  0x01, 0x20, 0x52, 0xE2, // SUBS r2, r2, #1
  0xFC, 0xFF, 0xFF, 0x1A, // BNE px
  0x01, 0x10, 0x81, 0xE2, // ADD r1, r1, #1
  0xA0, 0x00, 0x51, 0xE3, // CMP r1, #160
  0xF6, 0xFF, 0xFF, 0x1A, // BNE row
  0x01, 0x50, 0x85, 0xE2, // ADD r5, r5, #1  (scroll)
  0xF2, 0xFF, 0xFF, 0xEA, // B frame
]);

// One full gradient fill (160 rows) + frame advance = 116,164 instructions.
const PER_FRAME = 116164;

async function main() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch('gba_emulator.wasm'), {});
  const w = instance.exports;

  const emu = w.emu_create();

  // Load the ROM into wasm memory.
  const romPtr = w.emu_alloc(DEMO_ROM.length);
  new Uint8Array(w.memory.buffer, romPtr, DEMO_ROM.length).set(DEMO_ROM);
  w.emu_load_rom(emu, romPtr, DEMO_ROM.length);

  // Reset PC to the ROM entry point (ARM state) and run the 4 setup instructions.
  w.emu_set_pc(emu, 0x08000008);
  w.emu_step(emu, 4);

  // Framebuffer (allocated once; wasm memory won't grow after this point).
  const fbPtr = w.emu_alloc(FB_SIZE);
  const fb = new Uint8Array(w.memory.buffer, fbPtr, FB_SIZE);

  const canvas = document.getElementById('screen');
  canvas.width = WIDTH;
  canvas.height = HEIGHT;
  const ctx = canvas.getContext('2d');
  const img = ctx.createImageData(WIDTH, HEIGHT);

  const fpsEl = document.getElementById('fps');
  let frames = 0;
  let lastTime = performance.now();

  function render() {
    // Run exactly one gradient fill (CPU executes a real program each frame).
    w.emu_step(emu, PER_FRAME);
    w.emu_render(emu, fbPtr);

    for (let i = 0; i < WIDTH * HEIGHT; i++) {
      img.data[i * 4 + 0] = fb[i * 3 + 0];
      img.data[i * 4 + 1] = fb[i * 3 + 1];
      img.data[i * 4 + 2] = fb[i * 3 + 2];
      img.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);

    frames++;
    const now = performance.now();
    if (now - lastTime >= 1000) {
      fpsEl.textContent = frames;
      frames = 0;
      lastTime = now;
    }
    requestAnimationFrame(render);
  }

  requestAnimationFrame(render);
}

main();

const WIDTH = 240, HEIGHT = 160, FB_SIZE = WIDTH * HEIGHT * 3;

// Diagonal rainbow: color = x + y + t, scrolling every frame.
// (BGR555 — bit 15 is ignored, so the palette cycles red → green → blue.)
const DEMO_ROM = new Uint8Array([
  0x04, 0x34, 0xA0, 0xE3, // MOV r3, #0x04000000
  0x03, 0x40, 0xA0, 0xE3, // MOV r4, #3
  0xB0, 0x40, 0xC3, 0xE1, // STRH r4, [r3]   (mode 3)
  0x00, 0x50, 0xA0, 0xE3, // MOV r5, #0      (t)
  0x06, 0x04, 0xA0, 0xE3, // frame: MOV r0, #0x06000000
  0x00, 0x10, 0xA0, 0xE3, // MOV r1, #0      (y)
  0x01, 0x60, 0xA0, 0xE1, // row: MOV r6, r1
  0x05, 0x60, 0x86, 0xE0, // ADD r6, r6, r5  (color = y + t)
  0xF0, 0x20, 0xA0, 0xE3, // MOV r2, #240    (x)
  0xB2, 0x60, 0xC0, 0xE0, // px: STRH r6, [r0], #2
  0x01, 0x60, 0x86, 0xE2, // ADD r6, r6, #1  (color += 1 as x++)
  0x01, 0x20, 0x52, 0xE2, // SUBS r2, r2, #1
  0xFB, 0xFF, 0xFF, 0x1A, // BNE px
  0x01, 0x10, 0x81, 0xE2, // ADD r1, r1, #1
  0xA0, 0x00, 0x51, 0xE3, // CMP r1, #160
  0xF5, 0xFF, 0xFF, 0x1A, // BNE row
  0x01, 0x50, 0x85, 0xE2, // ADD r5, r5, #1  (scroll)
  0xF1, 0xFF, 0xFF, 0xEA, // B frame
]);

// One diagonal fill (160 rows) + frame advance = 154,564 instructions.
const PER_FRAME = 154564;

// The program, disassembled, so the register readout can be mapped to code.
const PROGRAM = [
  [0x08000000, 'MOV r3, #0x04000000', 'DISPCNT register'],
  [0x08000004, 'MOV r4, #3',           'video mode 3 (bitmap)'],
  [0x08000008, 'STRH r4, [r3]',        'set mode 3'],
  [0x0800000C, 'MOV r5, #0',           't = 0 (scroll offset)'],
  [0x08000010, 'MOV r0, #0x06000000',  'frame: r0 = VRAM base'],
  [0x08000014, 'MOV r1, #0',           'y = 0'],
  [0x08000018, 'MOV r6, r1',           'row: color = y'],
  [0x0800001C, 'ADD r6, r6, r5',       'color += t'],
  [0x08000020, 'MOV r2, #240',         'x = 240'],
  [0x08000024, 'STRH r6, [r0], #2',    'px: write pixel, r0 += 2'],
  [0x08000028, 'ADD r6, r6, #1',       'color += 1 (next column)'],
  [0x0800002C, 'SUBS r2, r2, #1',      'x--'],
  [0x08000030, 'BNE 0x08000024',       'loop px'],
  [0x08000034, 'ADD r1, r1, #1',       'y++'],
  [0x08000038, 'CMP r1, #160',         'y == 160?'],
  [0x0800003C, 'BNE 0x08000018',       'loop row'],
  [0x08000040, 'ADD r5, r5, #1',       't++ (scroll)'],
  [0x08000044, 'B 0x08000010',         'loop frame'],
];

const REG_NAMES = ['r0','r1','r2','r3','r4','r5','r6','r7','r8','r9','r10','r11','r12','sp','lr','pc'];

async function main() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch('gba_emulator.wasm'), {});
  const w = instance.exports;

  const emu = w.emu_create();

  // Load ROM, reset PC (r15 = address + 8), run the 4 setup instructions.
  const romPtr = w.emu_alloc(DEMO_ROM.length);
  new Uint8Array(w.memory.buffer, romPtr, DEMO_ROM.length).set(DEMO_ROM);
  w.emu_load_rom(emu, romPtr, DEMO_ROM.length);
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
  const instrEl = document.getElementById('instr');
  const regsEl = document.getElementById('regs');

  // Static program listing.
  document.getElementById('program').textContent = PROGRAM.map(
    ([a, asm, note]) => `0x${a.toString(16).padStart(8, '0')}  ${asm.padEnd(20)} ${note}`
  ).join('\n');

  let frames = 0;
  let totalInstr = 0;
  let lastTime = performance.now();

  function render() {
    // Execute exactly one frame's worth of instructions.
    w.emu_step(emu, PER_FRAME);
    w.emu_render(emu, fbPtr);
    totalInstr += PER_FRAME;

    for (let i = 0; i < WIDTH * HEIGHT; i++) {
      img.data[i * 4 + 0] = fb[i * 3 + 0];
      img.data[i * 4 + 1] = fb[i * 3 + 1];
      img.data[i * 4 + 2] = fb[i * 3 + 2];
      img.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);

    // Live register readout (two columns).
    const cols = [[], []];
    for (let r = 0; r < 16; r++) {
      const name = REG_NAMES[r];
      const val = '0x' + (w.emu_get_reg(emu, r) >>> 0).toString(16).padStart(8, '0');
      cols[r < 8 ? 0 : 1].push(`${name.padEnd(3)} ${val}`);
    }
    const cpsr = w.emu_get_cpsr(emu) >>> 0;
    const flag = (b) => (cpsr & (1 << b)) ? 1 : 0;
    const nzcv = `${flag(31) ? 'N' : '-'}${flag(30) ? 'Z' : '-'}${flag(29) ? 'C' : '-'}${flag(28) ? 'V' : '-'}`;
    const ift = `${flag(7) ? 'I' : '-'}${flag(6) ? 'F' : '-'}${flag(5) ? 'T' : '-'}`;
    let lines = [];
    for (let i = 0; i < 8; i++) lines.push(`${cols[0][i]}   ${cols[1][i]}`);
    regsEl.textContent = lines.join('\n') + `\nCPSR ${nzcv}  ${ift}`;

    frames++;
    const now = performance.now();
    if (now - lastTime >= 1000) {
      fpsEl.textContent = frames;
      instrEl.textContent = (totalInstr / 1e6).toFixed(1) + 'M';
      frames = 0;
      lastTime = now;
    }
    requestAnimationFrame(render);
  }

  requestAnimationFrame(render);
}

main();


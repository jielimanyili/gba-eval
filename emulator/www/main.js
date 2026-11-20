const WIDTH = 240, HEIGHT = 160, FB_SIZE = WIDTH * HEIGHT * 3;

// A bouncing 24×24 red square: clear the screen, advance position (dx, dy),
// bounce off the edges with predicated RSB, then redraw via nested loops and
// MUL-based addressing. A real program with state, collision, and drawing.
const DEMO_ROM = new Uint8Array([
  0x04, 0x34, 0xA0, 0xE3, // MOV r3, #0x04000000
  0x03, 0x40, 0xA0, 0xE3, // MOV r4, #3
  0xB0, 0x40, 0xC3, 0xE1, // STRH r4, [r3]   (mode 3)
  0x64, 0x50, 0xA0, 0xE3, // MOV r5, #100    (x)
  0x46, 0x60, 0xA0, 0xE3, // MOV r6, #70     (y)
  0x02, 0x70, 0xA0, 0xE3, // MOV r7, #2      (dx)
  0x01, 0x80, 0xA0, 0xE3, // MOV r8, #1      (dy)
  0x18, 0x90, 0xA0, 0xE3, // MOV r9, #24     (size)
  0x1F, 0xA0, 0xA0, 0xE3, // MOV r10, #0x1F  (red)
  0x06, 0xC4, 0xA0, 0xE3, // MOV r12, #0x06000000
  0xF0, 0xB0, 0xA0, 0xE3, // MOV r11, #0xF0  (240)
  0x0B, 0xB0, 0x8B, 0xE0, // ADD r11, r11, r11 (480 = row stride)
  0x0C, 0x00, 0xA0, 0xE1, // frame: MOV r0, r12
  0x96, 0x1C, 0xA0, 0xE3, // MOV r1, #0x9600 (38400 px)
  0x00, 0x20, 0xA0, 0xE3, // MOV r2, #0      (black)
  0xB2, 0x20, 0xC0, 0xE0, // clear: STRH r2, [r0], #2
  0x01, 0x10, 0x51, 0xE2, // SUBS r1, r1, #1
  0xFC, 0xFF, 0xFF, 0x1A, // BNE clear
  0x07, 0x40, 0x85, 0xE0, // ADD r4, r5, r7  (x + dx)
  0x00, 0x00, 0x54, 0xE3, // CMP r4, #0
  0x00, 0x70, 0x67, 0xB2, // RSBLT r7, r7, #0
  0x07, 0x40, 0x85, 0xB0, // ADDLT r4, r5, r7
  0x09, 0x40, 0x84, 0xE0, // ADD r4, r4, r9  (+ size)
  0xF0, 0x00, 0x54, 0xE3, // CMP r4, #240
  0x00, 0x70, 0x67, 0xC2, // RSBGT r7, r7, #0
  0x07, 0x50, 0x85, 0xE0, // ADD r5, r5, r7  (x += dx)
  0x08, 0x40, 0x86, 0xE0, // ADD r4, r6, r8  (y + dy)
  0x00, 0x00, 0x54, 0xE3, // CMP r4, #0
  0x00, 0x80, 0x68, 0xB2, // RSBLT r8, r8, #0
  0x08, 0x40, 0x86, 0xB0, // ADDLT r4, r6, r8
  0x09, 0x40, 0x84, 0xE0, // ADD r4, r4, r9  (+ size)
  0xA0, 0x00, 0x54, 0xE3, // CMP r4, #160
  0x00, 0x80, 0x68, 0xC2, // RSBGT r8, r8, #0
  0x08, 0x60, 0x86, 0xE0, // ADD r6, r6, r8  (y += dy)
  0x96, 0x0B, 0x00, 0xE0, // MUL r0, r6, r11 (y * 480)
  0x0C, 0x00, 0x80, 0xE0, // ADD r0, r0, r12 (+ base)
  0x85, 0x00, 0x80, 0xE0, // ADD r0, r0, r5, LSL #1 (+ x*2)
  0x09, 0x20, 0xA0, 0xE1, // MOV r2, r9      (rows)
  0x00, 0x10, 0xA0, 0xE1, // MOV r1, r0      (row addr)
  0x09, 0x30, 0xA0, 0xE1, // row: MOV r3, r9 (cols)
  0x01, 0x40, 0xA0, 0xE1, // MOV r4, r1      (pixel addr)
  0xB2, 0xA0, 0xC4, 0xE0, // col: STRH r10, [r4], #2
  0x01, 0x30, 0x53, 0xE2, // SUBS r3, r3, #1
  0xFC, 0xFF, 0xFF, 0x1A, // BNE col
  0x0B, 0x10, 0x81, 0xE0, // ADD r1, r1, r11 (+ 480)
  0x01, 0x20, 0x52, 0xE2, // SUBS r2, r2, #1
  0xF7, 0xFF, 0xFF, 0x1A, // BNE row
  0xDB, 0xFF, 0xFF, 0xEA, // B frame
]);

// One animation frame = clear + bounce + draw + loop = 117,073 instructions.
const PER_FRAME = 117073;

// The program, disassembled, so the register readout can be mapped to code.
const PROGRAM = [
  [0x08000000, 'MOV r3, #0x04000000', 'DISPCNT register'],
  [0x08000004, 'MOV r4, #3',           'video mode 3 (bitmap)'],
  [0x08000008, 'STRH r4, [r3]',        'set mode 3'],
  [0x0800000C, 'MOV r5, #100',         'x = 100'],
  [0x08000010, 'MOV r6, #70',          'y = 70'],
  [0x08000014, 'MOV r7, #2',           'dx = 2'],
  [0x08000018, 'MOV r8, #1',           'dy = 1'],
  [0x0800001C, 'MOV r9, #24',          'size = 24'],
  [0x08000020, 'MOV r10, #0x1F',       'color = red'],
  [0x08000024, 'MOV r12, #0x06000000', 'VRAM base'],
  [0x08000028, 'MOV r11, #0xF0',       '240'],
  [0x0800002C, 'ADD r11, r11, r11',    '480 = row stride'],
  [0x08000030, 'MOV r0, r12',          'frame: r0 = VRAM'],
  [0x08000034, 'MOV r1, #0x9600',      '38400 px'],
  [0x08000038, 'MOV r2, #0',           'black'],
  [0x0800003C, 'STRH r2, [r0], #2',    'clear: write pixel'],
  [0x08000040, 'SUBS r1, r1, #1',      'pixels--'],
  [0x08000044, 'BNE 0x0800003C',       'loop clear'],
  [0x08000048, 'ADD r4, r5, r7',       'x + dx'],
  [0x0800004C, 'CMP r4, #0',           'hit left edge?'],
  [0x08000050, 'RSBLT r7, r7, #0',     'if so: dx = -dx'],
  [0x08000054, 'ADDLT r4, r5, r7',     'recompute x + dx'],
  [0x08000058, 'ADD r4, r4, r9',       'x + dx + size'],
  [0x0800005C, 'CMP r4, #240',         'hit right edge?'],
  [0x08000060, 'RSBGT r7, r7, #0',     'if so: dx = -dx'],
  [0x08000064, 'ADD r5, r5, r7',       'x += dx'],
  [0x08000068, 'ADD r4, r6, r8',       'y + dy'],
  [0x0800006C, 'CMP r4, #0',           'hit top edge?'],
  [0x08000070, 'RSBLT r8, r8, #0',     'if so: dy = -dy'],
  [0x08000074, 'ADDLT r4, r6, r8',     'recompute y + dy'],
  [0x08000078, 'ADD r4, r4, r9',       'y + dy + size'],
  [0x0800007C, 'CMP r4, #160',         'hit bottom edge?'],
  [0x08000080, 'RSBGT r8, r8, #0',     'if so: dy = -dy'],
  [0x08000084, 'ADD r6, r6, r8',       'y += dy'],
  [0x08000088, 'MUL r0, r6, r11',      'y * 480'],
  [0x0800008C, 'ADD r0, r0, r12',      '+ VRAM base'],
  [0x08000090, 'ADD r0, r0, r5, LSL #1', '+ x*2 (top-left)'],
  [0x08000094, 'MOV r2, r9',           'rows = size'],
  [0x08000098, 'MOV r1, r0',           'row addr'],
  [0x0800009C, 'MOV r3, r9',           'row: cols = size'],
  [0x080000A0, 'MOV r4, r1',           'pixel addr'],
  [0x080000A4, 'STRH r10, [r4], #2',   'col: write red'],
  [0x080000A8, 'SUBS r3, r3, #1',      'cols--'],
  [0x080000AC, 'BNE 0x080000A4',       'loop col'],
  [0x080000B0, 'ADD r1, r1, r11',      'row += 480'],
  [0x080000B4, 'SUBS r2, r2, #1',      'rows--'],
  [0x080000B8, 'BNE 0x0800009C',       'loop row'],
  [0x080000BC, 'B 0x08000030',         'loop frame'],
];

const REG_NAMES = ['r0','r1','r2','r3','r4','r5','r6','r7','r8','r9','r10','r11','r12','sp','lr','pc'];

// --- Grading (SSIM replay, ADR-003) -------------------------------------

// Global SSIM between two equal-length byte arrays (port of grader/replay.py).
function ssim(a, b) {
  const n = a.length;
  if (n === 0 || n !== b.length) throw new Error('length mismatch');
  const C1 = (0.01 * 255) ** 2;
  const C2 = (0.03 * 255) ** 2;
  let muA = 0, muB = 0;
  for (let i = 0; i < n; i++) { muA += a[i]; muB += b[i]; }
  muA /= n; muB /= n;
  let va = 0, vb = 0, cov = 0;
  for (let i = 0; i < n; i++) {
    const da = a[i] - muA, db = b[i] - muB;
    va += da * da; vb += db * db; cov += da * db;
  }
  va /= (n - 1); vb /= (n - 1); cov /= (n - 1);
  return (2 * muA * muB + C1) * (2 * cov + C2) /
         ((muA * muA + muB * muB + C1) * (va + vb + C2));
}

// Known-correct first frame: black, with a red 24×24 square at (102, 71) —
// the bounce ROM's state after setup (12 instructions) + one frame.
function bounceReferenceFrame() {
  const ref = new Uint8Array(FB_SIZE);
  const X = 102, Y = 71, S = 24;
  for (let y = Y; y < Y + S; y++) {
    for (let x = X; x < X + S; x++) {
      ref[(y * WIDTH + x) * 3] = 255;
    }
  }
  return ref;
}

// Run a fresh emulator instance to the deterministic checkpoint and SSIM its
// frame against the golden reference — the in-browser replay grader.
function gradeBounceDemo(w) {
  const emu = w.emu_create();
  const romPtr = w.emu_alloc(DEMO_ROM.length);
  new Uint8Array(w.memory.buffer, romPtr, DEMO_ROM.length).set(DEMO_ROM);
  w.emu_load_rom(emu, romPtr, DEMO_ROM.length);
  w.emu_set_pc(emu, 0x08000008);
  w.emu_step(emu, 12 + PER_FRAME); // setup + one frame

  const fbPtr = w.emu_alloc(FB_SIZE);
  w.emu_render(emu, fbPtr);
  const candidate = new Uint8Array(w.memory.buffer, fbPtr, FB_SIZE);
  const score = ssim(candidate, bounceReferenceFrame());
  w.emu_destroy(emu);
  return score;
}

async function main() {
  const { instance } = await WebAssembly.instantiateStreaming(fetch('gba_emulator.wasm'), {});
  const w = instance.exports;

  const emu = w.emu_create();

  // Load ROM, reset PC (r15 = address + 8), run the 12 setup instructions.
  const romPtr = w.emu_alloc(DEMO_ROM.length);
  new Uint8Array(w.memory.buffer, romPtr, DEMO_ROM.length).set(DEMO_ROM);
  w.emu_load_rom(emu, romPtr, DEMO_ROM.length);
  w.emu_set_pc(emu, 0x08000008);
  w.emu_step(emu, 12);

  // Grade the candidate against the known-correct first frame (SSIM replay).
  // Do this before allocating the main framebuffer view: emu_alloc may grow
  // wasm memory, which detaches existing Uint8Array views of it.
  const ssimScore = gradeBounceDemo(w);

  // Framebuffer (allocated after all emu_alloc growth, so this view is stable).
  const fbPtr = w.emu_alloc(FB_SIZE);
  const fb = new Uint8Array(w.memory.buffer, fbPtr, FB_SIZE);

  const canvas = document.getElementById('screen');
  canvas.width = WIDTH;
  canvas.height = HEIGHT;
  const ctx = canvas.getContext('2d');
  const img = ctx.createImageData(WIDTH, HEIGHT);

  const fpsEl = document.getElementById('fps');
  const instrEl = document.getElementById('instr');
  const pcEl = document.getElementById('pc');
  const regsEl = document.getElementById('regs');

  // Show the grading report.
  const ssimEl = document.getElementById('ssim');
  const verdictEl = document.getElementById('verdict');
  ssimEl.textContent = ssimScore.toFixed(6);
  const pass = ssimScore >= 0.99;
  verdictEl.textContent = pass ? 'PASS ✓' : 'FAIL ✗';
  verdictEl.className = pass ? 'pass' : 'fail';

  // Build the disassembly listing as highlightable rows.
  const programEl = document.getElementById('program');
  const rows = new Map();
  for (const [addr, asm, note] of PROGRAM) {
    const row = document.createElement('div');
    row.className = 'instr';
    const a = document.createElement('span'); a.className = 'addr';
    a.textContent = '0x' + addr.toString(16).padStart(8, '0');
    const m = document.createElement('span'); m.className = 'mnem'; m.textContent = asm;
    const n = document.createElement('span'); n.className = 'note'; n.textContent = note;
    row.append(a, m, n);
    programEl.appendChild(row);
    rows.set(addr, row);
  }

  const toggleBtn = document.getElementById('toggle');
  const stepBtn = document.getElementById('step');
  let running = true;
  let totalInstr = 0;
  let frames = 0;
  let lastTime = performance.now();

  function highlight(addr) {
    for (const row of rows.values()) row.classList.remove('current');
    const row = rows.get(addr);
    if (row) row.classList.add('current');
  }

  function draw() {
    w.emu_render(emu, fbPtr);
    for (let i = 0; i < WIDTH * HEIGHT; i++) {
      img.data[i * 4 + 0] = fb[i * 3 + 0];
      img.data[i * 4 + 1] = fb[i * 3 + 1];
      img.data[i * 4 + 2] = fb[i * 3 + 2];
      img.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);
  }

  function update() {
    draw();

    // Live register readout (two columns).
    const cols = [[], []];
    for (let r = 0; r < 16; r++) {
      const val = '0x' + (w.emu_get_reg(emu, r) >>> 0).toString(16).padStart(8, '0');
      cols[r < 8 ? 0 : 1].push(`${REG_NAMES[r].padEnd(3)} ${val}`);
    }
    const cpsr = w.emu_get_cpsr(emu) >>> 0;
    const f = (b) => (cpsr & (1 << b)) ? 1 : 0;
    const nzcv = `${f(31) ? 'N' : '-'}${f(30) ? 'Z' : '-'}${f(29) ? 'C' : '-'}${f(28) ? 'V' : '-'}`;
    const ift = `${f(7) ? 'I' : '-'}${f(6) ? 'F' : '-'}${f(5) ? 'T' : '-'}`;
    let lines = [];
    for (let i = 0; i < 8; i++) lines.push(`${cols[0][i]}   ${cols[1][i]}`);
    regsEl.textContent = lines.join('\n') + `\nCPSR ${nzcv}  ${ift}`;

    // Current instruction (ARM: next fetch address = r15 - 8).
    const addr = (w.emu_get_reg(emu, 15) >>> 0) - 8;
    pcEl.textContent = '0x' + addr.toString(16).padStart(8, '0');
    highlight(addr);
  }

  function stepN(n) {
    w.emu_step(emu, n);
    totalInstr += n;
    update();
    instrEl.textContent = (totalInstr / 1e6).toFixed(1) + 'M';
  }

  function setRunning(v) {
    running = v;
    toggleBtn.textContent = running ? 'Pause' : 'Run';
    stepBtn.disabled = running;
    if (!running) fpsEl.textContent = '—';
  }

  toggleBtn.addEventListener('click', () => setRunning(!running));
  stepBtn.addEventListener('click', () => { setRunning(false); stepN(1); });
  window.addEventListener('keydown', (e) => {
    if (e.code === 'Space') { e.preventDefault(); setRunning(!running); }
    else if (e.code === 'ArrowRight' && !running) stepN(1);
  });

  function tick() {
    if (running) {
      stepN(PER_FRAME);
      frames++;
    }
    const now = performance.now();
    if (now - lastTime >= 1000) {
      fpsEl.textContent = frames;
      frames = 0;
      lastTime = now;
    }
    requestAnimationFrame(tick);
  }

  update(); // initial state: mode 3 set, black screen, PC at frame
  requestAnimationFrame(tick);
}

main();


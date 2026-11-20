-- Capture the first rendered frame of a ROM to a raw RGB file.
--
-- Mesen2 Lua script — the black-box oracle from ADR-002. Mesen2 is GPL and is
-- invoked as a binary only; this script is our own code and only talks to
-- Mesen2's public scripting API.
--
-- Usage: Mesen2 --runscript mesen2_capture.lua <rom> <out.raw> <steps>
--
-- NOTE: the API used here (emu.loadRom / emu.reset / emu.runFrame /
-- emu.getScreenBuffer) is Mesen2's public Lua interface. It has not been
-- exercised against a real Mesen2 binary in this repo's CI (no Mesen2 in the
-- test environment); verify the exact function names/return format against
-- the installed Mesen2 version before relying on the `mesen2` backend.

local rom = ...
local out = ...
local steps = tonumber(...)

-- GBA screen dimensions.
local WIDTH = 240
local HEIGHT = 160

emu.loadRom(rom)
emu.reset()

-- Run far enough that the ROM has rendered a stable frame.
for i = 1, steps do
    emu.runFrame()
end

-- getScreenBuffer returns the current framebuffer. Mesen2 exposes it as a
-- 1-indexed table of 32-bit ARGB pixels; emit 8-bit RGB for the grader.
local buf = emu.getScreenBuffer()
local f = io.open(out, "wb")
for i = 1, #buf do
    local px = buf[i]
    local b = px & 0xFF
    local g = (px >> 8) & 0xFF
    local r = (px >> 16) & 0xFF
    f:write(string.char(r, g, b))
end
f:close()

print(string.format("captured %d pixels to %s", #buf, out))

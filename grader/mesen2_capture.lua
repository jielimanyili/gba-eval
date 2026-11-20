-- Capture a deterministic frame to a raw RGB file.
--
-- Mesen/MesenCE headless "test runner" mode (ADR-002):
--   Mesen --testrunner <rom> this_script.lua
-- Mesen loads the ROM; this script counts frames, captures the framebuffer at
-- the target frame, writes RGB bytes, and calls emu.stop().
--
-- Output path and frame target are passed via environment (set by oracle.py),
-- avoiding any dependency on how Mesen forwards CLI args to Lua scripts.
--
-- NOTE: the API used here (emu.addEventCallback / emu.eventType.endFrame /
-- emu.getScreenBuffer / emu.stop) is Mesen's public Lua interface. It has not
-- been exercised against a real Mesen binary in this repo's CI; verify the
-- exact names/return format against the installed version.

local out = os.getenv("MESEN_CAPTURE_OUT") or "mesen_capture.raw"
local frameTarget = tonumber(os.getenv("MESEN_CAPTURE_FRAMES") or "5")

local frame = 0

emu.addEventCallback(emu.eventType.endFrame, function()
    frame = frame + 1
    if frame == frameTarget then
        -- getScreenBuffer returns a 1-indexed table of 32-bit ARGB pixels.
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
        emu.stop()
    end
end)

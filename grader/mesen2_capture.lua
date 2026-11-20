-- Capture a deterministic frame to a raw RGB file.
--
-- Mesen/MesenCE headless "test runner" mode (ADR-002):
--   Mesen --testRunner <rom> this_script.lua
-- Mesen loads the ROM; this script counts frames, captures the framebuffer at
-- the target frame, writes RGB bytes, and calls emu.stop().
--
-- Requirements discovered while wiring this up (MesenCE 2.2.1):
--   1. The Lua `io`/`os` libraries are gated behind a config setting. Seed
--      settings.json with:
--        {"Debug": {"ScriptWindow": {"AllowIoOsAccess": true}}}
--      (the Dockerfile does this).
--   2. addEventCallback signature is (callbackFunction, eventType) — the
--      function comes FIRST.
--   3. getScreenBuffer() returns a 1-indexed table of 38400 32-bit pixels
--      (ARGB — verify the channel order against a known non-black frame).
--   4. emu.stop(code) returns `code` as the process exit code.

local out = os.getenv("MESEN_CAPTURE_OUT") or "mesen_capture.raw"
local frameTarget = tonumber(os.getenv("MESEN_CAPTURE_FRAMES") or "5")

local frame = 0

emu.addEventCallback(function()
    frame = frame + 1
    if frame == frameTarget then
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
        emu.stop(0)
    end
end, emu.eventType.endFrame)

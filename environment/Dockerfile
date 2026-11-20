# Candidate sandbox for the GBA Eval clone (ADR-004).
#
# Provides the toolchain, the ABI spec (TASK.md), dev ROMs, and a black-box
# oracle CLI. The candidate emulator is written and built *inside* this
# sandbox, with no internet at runtime (enforced by `docker run --network none`).

FROM rust:1-bookworm

# --- Runtime deps for the oracle CLI (Python) + Mesen download (unzip/curl) +
# --- SDL2 (required by Mesen/MesenCE's native Linux build).
RUN apt-get update && apt-get install -y --no-install-recommends \
        python3 curl unzip libsdl2-2.0-0 \
    && rm -rf /var/lib/apt/lists/*

# --- MesenCE oracle (black-box binary, GPL — invoked, never vendored/modified).
# MesenCE is the maintained successor to the archived Mesen2. The release asset
# name/URL must be verified against the pinned release before building.
ARG MESENCE_VERSION=2.2.1
RUN set -eux; \
    case "$(uname -m)" in \
      x86_64) ARCH=x64 ;; \
      aarch64|arm64) ARCH=ARM64 ;; \
      *) echo "unsupported arch: $(uname -m)"; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/mesen.zip \
      "https://github.com/nesdev-org/MesenCE/releases/download/${MESENCE_VERSION}/Mesen_${MESENCE_VERSION}_Linux_${ARCH}.zip"; \
    unzip -q /tmp/mesen.zip -d /opt/mesen; \
    rm /tmp/mesen.zip

# --- Toolchain: Rust + wasm32 (the candidate compiles to WebAssembly).
RUN rustup target add wasm32-unknown-unknown

# --- Sandbox content: ABI spec, dev ROMs, oracle CLI.
WORKDIR /work
COPY environment/TASK.md /work/TASK.md
COPY grader/oracle.py grader/mesen2_capture.lua grader/replay.py /work/grader/
COPY grader/roms/ /work/roms/
# Build the candidate emulator (gba-run) inside the sandbox, for Linux — the
# model's deliverable would be built here the same way.
COPY emulator/ /work/emulator/
RUN cd /work/emulator && cargo build --release --bin gba-run
COPY environment/oracle /usr/local/bin/oracle
RUN chmod +x /usr/local/bin/oracle

ENV MESEN=/opt/mesen/Mesen
ENV PATH="/opt/mesen:${PATH}"
# Pre-seed Mesen's config so it skips the first-run GUI wizard (which needs
# X11) AND grants Lua scripts io/os access (required to write the captured
# framebuffer). The io/os gate is Debug.ScriptWindow.AllowIoOsAccess.
RUN mkdir -p /root/.config/Mesen2 \
    && echo '{"Debug":{"ScriptWindow":{"AllowIoOsAccess":true}}}' > /root/.config/Mesen2/settings.json
# Headless (no display/audio) — Mesen's SDL2 must not try to open a window.
ENV SDL_VIDEODRIVER=dummy
ENV SDL_AUDIODRIVER=dummy

CMD ["/bin/bash"]

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
ARG MESENCE_VERSION=2.2.0
ARG MESENCE_URL=https://github.com/nesdev-org/MesenCE/releases/download/${MESENCE_VERSION}/Mesen.${MESENCE_VERSION}-linux-x64.zip
RUN curl -fsSL -o /tmp/mesen.zip "$MESENCE_URL" \
    && unzip -q /tmp/mesen.zip -d /opt/mesen \
    && rm /tmp/mesen.zip

# --- Toolchain: Rust + wasm32 (the candidate compiles to WebAssembly).
RUN rustup target add wasm32-unknown-unknown

# --- Sandbox content: ABI spec, dev ROMs, oracle CLI.
WORKDIR /work
COPY environment/TASK.md /work/TASK.md
COPY grader/oracle.py grader/mesen2_capture.lua grader/replay.py /work/grader/
COPY grader/roms/ /work/roms/
COPY environment/oracle /usr/local/bin/oracle
RUN chmod +x /usr/local/bin/oracle

ENV MESEN=/opt/mesen/Mesen
ENV PATH="/opt/mesen:${PATH}"

CMD ["/bin/bash"]

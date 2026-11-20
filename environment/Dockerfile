# Candidate sandbox for the GBA Eval clone (ADR-004).
#
# Provides the toolchain, the ABI spec (TASK.md), dev ROMs, and a black-box
# oracle CLI. The candidate emulator is written and built *inside* this
# sandbox, with no internet at runtime (enforced by `docker run --network none`).

FROM rust:1-bookworm

# --- Runtime deps for the oracle CLI (Python) + Mesen2 download (unzip/curl).
RUN apt-get update && apt-get install -y --no-install-recommends \
        python3 curl unzip \
    && rm -rf /var/lib/apt/lists/*

# --- Mesen2 oracle (black-box binary, GPL — invoked, never vendored/modified).
# Pin a release; adjust the asset name to the exact Mesen2 release if needed.
ARG MESEN2_VERSION=2.0.0.1
ARG MESEN2_URL=https://github.com/SourMesen/Mesen2/releases/download/${MESEN2_VERSION}/Mesen2.${MESEN2_VERSION}-linux-x64.zip
RUN curl -fsSL -o /tmp/mesen2.zip "$MESEN2_URL" \
    && unzip -q /tmp/mesen2.zip -d /opt/mesen2 \
    && rm /tmp/mesen2.zip

# --- Toolchain: Rust + wasm32 (the candidate compiles to WebAssembly).
RUN rustup target add wasm32-unknown-unknown

# --- Sandbox content: ABI spec, dev ROMs, oracle CLI.
WORKDIR /work
COPY environment/TASK.md /work/TASK.md
COPY grader/oracle.py grader/mesen2_capture.lua grader/replay.py /work/grader/
COPY grader/roms/ /work/roms/
COPY environment/oracle /usr/local/bin/oracle
RUN chmod +x /usr/local/bin/oracle

ENV MESEN2=/opt/mesen2/Mesen2
ENV PATH="/opt/mesen2:${PATH}"

CMD ["/bin/bash"]

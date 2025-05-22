FROM rust:1.84.1

# Install needed packages and clean up
RUN apt update && \
    apt install -y --no-install-recommends lsb-release wget software-properties-common gnupg gcc-multilib git && \
    rm -rf /var/lib/apt/lists/*

# Install clang and clean up
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 17 && \
    ln -s $(which clang-17) /usr/bin/clang && \
    ln -s $(which llvm-ar-17) /usr/bin/llvm-ar && \
    rm llvm.sh

# Install rust target
RUN rustup target install wasm32-wasip1

# Install cargo-msfs
RUN cargo install --git https://github.com/navigraph/cargo-msfs

# Cache bust arg to re-install both SDKs
ARG CACHEBUST

# Install MSFS2020 and MSFS2024 SDK
RUN cargo-msfs install msfs2020 && \
    cargo-msfs install msfs2024

# Needed when running in CI/CD to avoid dubious ownership errors
RUN git config --global --add safe.directory /workspace
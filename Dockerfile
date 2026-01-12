FROM rust:1.90

# Install base needed packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends lsb-release wget gnupg gcc-multilib git

# Install llvm-toolchain-17
RUN apt-get install -y --no-install-recommends \
    clang-17 \
    clangd-17 \
    clang-format-17 \
    clang-tidy-17 \
    clang-tools-17 \
    llvm-17-dev \
    llvm-17-tools \
    lld-17 \
    lldb-17 \
    liblldb-17-dev \
    libomp-17-dev \
    libc++-17-dev \
    libc++abi-17-dev \
    libunwind-17-dev \
    libclang-common-17-dev \
    libclang-17-dev \
    libclang-cpp17-dev && \
    ln -s $(which clang-17) /usr/bin/clang && \
    ln -s $(which llvm-ar-17) /usr/bin/llvm-ar

# Remove apt index
RUN rm -rf /var/lib/apt/lists/*

# Install rust target
RUN rustup target install wasm32-wasip1

# Install cargo-msfs
RUN cargo install --git https://github.com/navigraph/cargo-msfs --tag v1.1.0 --locked

# Cache bust arg to re-install both SDKs
ARG CACHEBUST

# Install MSFS2020 SDK
RUN cargo-msfs install msfs2020

# Needed when running in CI/CD to avoid dubious ownership errors
RUN git config --global --add safe.directory /workspace
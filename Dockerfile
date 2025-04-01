FROM ghcr.io/navigraph/cargo-msfs-bin:latest AS base

RUN apt-get update
RUN apt install git -y 

RUN cargo-msfs install msfs2020
RUN cargo-msfs install msfs2024

FROM base AS builder

WORKDIR /external

COPY rust-toolchain.toml ./
RUN rustup show

COPY . .

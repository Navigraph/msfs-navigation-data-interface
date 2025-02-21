FROM ghcr.io/navigraph/cargo-msfs-bin:latest

WORKDIR /external

RUN apt install git npm -y 

RUN cargo-msfs install msfs2020
RUN cargo-msfs install msfs2024

COPY rust-toolchain.toml ./
RUN rustup show

COPY package.json ./
COPY package-lock.json ./
RUN npm i

COPY . .

FROM ghcr.io/navigraph/cargo-msfs-bin:latest

WORKDIR /external

RUN apt install git -y 

RUN cargo-msfs install msfs2020
RUN cargo-msfs install msfs2024

RUN apt install npm -y

COPY package.json ./
COPY package-lock.json ./

RUN npm i

COPY rust-toolchain.toml ./

RUN rustup show

COPY . .



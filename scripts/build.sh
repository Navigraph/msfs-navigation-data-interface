#!/bin/bash

# Required for compilation
cargo install --git https://github.com/navigraph/cargo-msfs

cargo-msfs install msfs2020

# Flags needed to get sqlite3 to work in the sim
export LIBSQLITE3_FLAGS="-DSQLITE_OMIT_SHARED_CACHE -D_LARGEFILE64_SOURCE"

cargo-msfs build msfs2020 -i .. -o ../out/msfs_navigation_data_interface.wasm

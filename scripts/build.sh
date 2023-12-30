#!/bin/bash

# Flags needed to get sqlite3 to work in the sim
export LIBSQLITE3_FLAGS="-DSQLITE_OMIT_SHARED_CACHE -D_LARGEFILE64_SOURCE"

cargo build --target wasm32-wasi --release && wasm-opt -O1 --signext-lowering --enable-bulk-memory -o /out/msfs_navdata_interface.wasm /external/target/wasm32-wasi/release/msfs_navdata_interface.wasm

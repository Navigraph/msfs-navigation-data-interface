#!/bin/bash

# Flags needed to get sqlite3 to work in the sim
export LIBSQLITE3_FLAGS="-DSQLITE_OMIT_SHARED_CACHE -D_LARGEFILE64_SOURCE"

cargo build --target wasm32-wasi

cp /external/target/wasm32-wasi/debug/msfs_navigation_data_interface.wasm /out/msfs_navigation_data_interface.wasm 

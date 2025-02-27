#!/bin/bash

if ! [ -a out20 ]; then
mkdir out20
fi

if ! [ -a out24 ]; then
mkdir out24
fi

if [ $1 = "2020" ] || [ $1 = "both" ]; then
    echo "Building MSFS 2020"
    cargo-msfs build msfs2020 -i . -o out20/msfs_navigation_data_interface.wasm
fi

if [ $1 = "2024" ] || [ $1 = "both" ]; then
    echo "Building MSFS 2024"
    cargo-msfs build msfs2024 -i . -o out24/msfs_navigation_data_interface.wasm
fi

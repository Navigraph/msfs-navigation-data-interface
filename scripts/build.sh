mkdir ./out20
mkdir ./out24

echo Installing MSFS SDK

cargo-msfs install msfs2020
cargo-msfs install msfs2024

echo Building packages

cargo-msfs build msfs2020 -i . -o ./out20/msfs_navigation_data_interface.wasm
cargo-msfs build msfs2024 -i . -o ./out24/msfs_navigation_data_interface.wasm


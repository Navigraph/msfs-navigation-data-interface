mkdir ../out

cargo-msfs info

cargo-msfs build msfs2020 -i .. -o ../out/msfs_navigation_data_interface.wasm

zip ../interface-2020.zip ../out/msfs_navigation_data_interface.wasm

rm -rf ../out

cargo-msfs build msfs2024 -i .. -o ../out/msfs_navigation_data_interface.wasm

zip ../interface-2024.zip ../out/msfs_navigation_data_interface.wasm

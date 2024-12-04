@echo off

cd %~dp0

call cargo-msfs build msfs%1
call wasm-opt -O1 --signext-lowering --enable-bulk-memory -o ../out/msfs_navigation_data_interface.wasm ../target/wasm32-wasip1/release/msfs_navigation_data_interface.wasm

cd %~dp0

copy ..\out\msfs_navigation_data_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navigation_Data_Interface_Aircraft\panel
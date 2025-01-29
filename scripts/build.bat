@echo off

cd %~dp0

mkdir ..\out

cargo-msfs build msfs2020 -i .. -o ..\out\msfs_navigation_data_interface.wasm

copy ..\out\msfs_navigation_data_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navigation_Data_Interface_Aircraft\panel
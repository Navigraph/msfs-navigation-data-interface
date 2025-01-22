@echo off

cd %~dp0

cargo-msfs build msfs2020 -i .. -o ..\out\msfs_navigation_data_interface.wasm

cd %~dp0

copy ..\out\msfs_navigation_data_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navigation_Data_Interface_Aircraft\panel
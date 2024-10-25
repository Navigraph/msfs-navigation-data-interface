@echo off

cd %~dp0

call .\run_docker_cmd.bat ./scripts/build_dev.sh

cd %~dp0

:: copy ..\out\msfs_navigation_data_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navigation_Data_Interface_Aircraft\panel
@echo off

cd %~dp0

CALL .\run_docker_cmd.bat ./scripts/build.sh %*

copy ..\out20\msfs_navigation_data_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navigation_Data_Interface_Aircraft\panel
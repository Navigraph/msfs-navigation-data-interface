@echo off

cd %~dp0

call .\run_docker_cmd.bat ./scripts/build.sh

cd %~dp0

copy ..\out\msfs_navdata_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navdata_Interface_Aircraft\panel
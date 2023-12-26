@echo off

cd %~dp0

call .\run_docker_cmd.bat ./scripts/build.sh

@REM For some reason, the call command messes up the working directory, so we need to change it back
cd %~dp0

copy ..\out\msfs_navdata_interface.wasm ..\examples\aircraft\PackageSources\SimObjects\Airplanes\Navigraph_Navdata_Interface_Aircraft\panel
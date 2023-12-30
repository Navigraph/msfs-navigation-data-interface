@echo off

cd %~dp0

if "%1"=="" (
    echo No Cargo command specified
) else (
    .\run_docker_cmd.bat cargo %*
)
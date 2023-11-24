@echo off

cd %~dp0

if "%1"=="" (
        echo No Cargo command specified
) else (
    .\scripts\run_docker_cmd.bat ./scripts/cargo_cmd.sh %*
)
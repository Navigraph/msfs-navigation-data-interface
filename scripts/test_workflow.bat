@echo off

cd %~dp0

rmdir /s /q ..\test_work
mkdir ..\test_work

:: Docker is a must for now, WASI preopens don't exist on windows.
call .\run_docker_cmd_workflow.bat npm run jest

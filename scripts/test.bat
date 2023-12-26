@echo off

cd %~dp0

rmdir /s /q ..\test_out
mkdir ..\test_out

call .\run_docker_cmd.bat npm run jest

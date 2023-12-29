@echo off

cd %~dp0

rmdir /s /q ..\test_work
mkdir ..\test_work

call .\run_docker_cmd.bat npm run jest

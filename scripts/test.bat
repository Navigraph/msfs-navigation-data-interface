@echo off

cd %~dp0

rmdir /s /q ..\test_work
mkdir ..\test_work
mkdir ..\test_work\navigraph-test

call .\run_docker_cmd.bat npm run jest

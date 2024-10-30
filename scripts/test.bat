@echo off

cd %~dp0

rmdir /s /q ..\test_work\work
mkdir ..\test_work\work
mkdir ..\test_work\work\navigraph-test

cd ..

call bun test --bail

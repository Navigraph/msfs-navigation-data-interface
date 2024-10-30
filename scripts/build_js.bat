@echo off

cd .\src\js

call bun run build

cd %~dp0

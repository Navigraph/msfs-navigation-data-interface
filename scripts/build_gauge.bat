@echo off

cd .\examples\gauge

call bun run build

cd %~dp0
@echo off

cd .\src\js

call npm run build

cd %~dp0

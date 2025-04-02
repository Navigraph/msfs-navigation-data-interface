@echo off

set image="local"

cd %~dp0

docker run --rm -it -v "%cd%\..\:/external" -w /external %image% %*
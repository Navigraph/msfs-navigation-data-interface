@echo off

set image="ghcr.io/flybywiresim/dev-env@sha256:528f8e1ca9063b9346c7d4f684d7aadbcb58ca1fba2b1a3c2cdd9c820c4236f4"

cd %~dp0

docker run --rm -it -v "%cd%\..\:/external" -v "%cd%\..\out:/out" %image% %*
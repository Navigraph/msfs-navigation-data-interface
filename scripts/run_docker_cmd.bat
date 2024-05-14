@echo off

set image="ghcr.io/flybywiresim/dev-env@sha256:cb436009c99e1c86ff075f137f870201f6e04de08e0b6364d38b83a2f81dc58e"

cd %~dp0

docker run --rm -it -v "%cd%\..\:/external" -v "%cd%\..\out:/out" %image% %*
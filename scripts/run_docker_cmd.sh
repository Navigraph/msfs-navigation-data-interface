#!/bin/bash

IMAGE="ghcr.io/flybywiresim/dev-env@sha256:cb436009c99e1c86ff075f137f870201f6e04de08e0b6364d38b83a2f81dc58e"

cd "$(dirname "$0")"

echo "Running $@ in docker"

docker run --rm -v "$(pwd)/../:/external" -v "$(pwd)/../out:/out" $IMAGE "$@"

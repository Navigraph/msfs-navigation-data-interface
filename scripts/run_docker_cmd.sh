#!/bin/bash

IMAGE="local"

cd "$(dirname "$0")"

echo "Running $@ in docker"

docker run --rm -v "$(pwd)/../:/external" -w /external $IMAGE "$@"
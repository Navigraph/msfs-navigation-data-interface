#!/bin/bash

IMAGE="ghcr.io/flybywiresim/dev-env@sha256:13b03e3cdbc4a22f1a710e22771db0cb3f7bd54af42f9eb5cb749f816eb68844"

cd "$(dirname "$0")"

echo "Running $@ in docker"

docker run --rm -v "$(pwd)/../:/external" -v "$(pwd)/../out:/out" $IMAGE "$@"

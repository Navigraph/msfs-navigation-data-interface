#!/bin/bash

IMAGE="ghcr.io/flybywiresim/dev-env@sha256:528f8e1ca9063b9346c7d4f684d7aadbcb58ca1fba2b1a3c2cdd9c820c4236f4"

cd "$(dirname "$0")"

docker run --rm -v "$(pwd)/../:/external" -v "$(pwd)/../out:/out" $IMAGE "$@"

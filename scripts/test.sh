#!/bin/bash

rm -rf test_work
mkdir test_work

cd "$(dirname "$0")"

./run_docker_cmd.sh npm ci
./run_docker_cmd.sh npm run jest
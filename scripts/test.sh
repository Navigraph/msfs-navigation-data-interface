#!/bin/bash

rm -rf test_work
mkdir test_work
mkdir test_work/navigraph-test

source "${BASH_SOURCE%/*}/run_docker_cmd.sh" npm ci
source "${BASH_SOURCE%/*}/run_docker_cmd.sh" npm run jest
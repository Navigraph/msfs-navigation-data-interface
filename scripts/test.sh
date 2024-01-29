#!/bin/bash

rm -rf test_work
mkdir test_work

source "${BASH_SOURCE%/*}/run_docker_cmd.sh" npm run jest
#!/bin/bash

rm -rf test_work
mkdir test_work
mkdir test_work/navigraph-test

bash ./scripts/run_docker_cmd.sh npm ci
bash ./scripts/run_docker_cmd.sh npm run jest
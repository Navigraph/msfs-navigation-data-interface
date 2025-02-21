#!/bin/bash

docker build -t local .

./scripts/run_docker_cmd.sh ./scripts/build.sh both

echo "Building done, now zipping"

zip -j 2020.zip out20/*
zip -j 2024.zip out24/*
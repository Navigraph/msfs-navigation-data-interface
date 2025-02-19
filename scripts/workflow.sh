docker build -t local .

./scripts/run_docker_cmd.sh ./scripts/build.sh both

zip -j interface2020.zip out20/*
zip -j interface2024.zip out24/*
#!/usr/bin/env bash

set -e
set -x

docker rm -f wasmcloud-postgres ||:
docker \
       run \
       --name wasmcloud-postgres \
       -e POSTGRES_PASSWORD=hunter2 \
       -e POSTGRES_DB=wasmcloud \
       -p 5432:5432 \
       -d \
       postgres:alpine
sleep 2
diesel migration run

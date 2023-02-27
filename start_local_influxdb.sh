#!/usr/bin/env bash
CONTAINER_NAME=solaris-influxdb
docker run --rm -p 8086:8086 -d --name $CONTAINER_NAME influxdb:2.6.1
echo "waiting 10 seconds for influxdb to start"
sleep 10
docker exec $CONTAINER_NAME influx setup \
      --username solaris \
      --password solaris!test123 \
      --org solaris \
      --bucket solaris \
      --force

TOKEN=$(docker exec $CONTAINER_NAME influx auth list \
      --user solaris \
      --json | jq -r '.[].token')

sed -i "s/token:.*/token: $TOKEN/" ./config.yaml

ORG_ID=$(docker exec solaris-influxdb influx org list \
      --name solaris \
      --json | jq -r '.[].id')

sed -i "s/org:.*/org: $ORG_ID/" ./config.yaml

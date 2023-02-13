#!/usr/bin/env bash
set -x
set -eo pipefail

# if a redis container is running, print instructions to kill it and exit 
RUNNING_CONTAINER=$(docker ps --filter 'name=smtp' --format '{{.ID}}')
if [[ -n $RUNNING_CONTAINER ]]; then
  echo >&2 "there is an smtp container already running, kill it with."
  echo >&2 "    docker kill ${RUNNING_CONTAINER}"
  exit 1
fi

# Launch Redis using Docker
docker run \
    -p "1025:1025" \
    -p "8025:8025" \
    -d \
    --name "smtp_$(date '+%s')" \
    mailhog/mailhog

>&2 echo "mailhog Smtp server is ready to go"

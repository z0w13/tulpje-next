#!/usr/bin/env bash
# Utility to inject service IP addresses from docker, rather than using the hostnames
# which doesn't work when running on the host

get_container_ip() {
  docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$1"
}

export RABBITMQ_ADDRESS=amqp://$(get_container_ip "tulpje-next-rabbitmq-1"):5672
export DISCORD_PROXY=$(get_container_ip "tulpje-next-discord_proxy-1"):80
export DISCORD_GATEWAY_QUEUE=http://$(get_container_ip "tulpje-next-gateway_queue-1"):80
export REDIS_URL=redis://$(get_container_ip "tulpje-next-valkey-1"):6379

exec "$@"

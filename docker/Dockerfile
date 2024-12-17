FROM debian:bookworm

RUN apt-get update -y \
 && apt-get install -y ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY target/debug/tulpje-gateway /bin/tulpje-gateway
COPY target/debug/tulpje-handler /bin/tulpje-handler

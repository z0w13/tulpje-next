FROM docker.io/alpine:3.21.0

# Add curl for health check
RUN apk --no-cache add curl

COPY --from=upstream_image /twilight-http-proxy /twilight-http-proxy
ENTRYPOINT [ "/twilight-http-proxy" ]

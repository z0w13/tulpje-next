#!/usr/bin/env fish

if test -z "$DOCKER_REPO"
    echo "DOCKER_REPO env var is empty please specify remote repository"
    exit 1
end

set -x IMAGE_SUFFIX :(git describe --abbrev=0 | string sub -s2)
echo "* image tag:" (string sub -s2 "$IMAGE_SUFFIX")

# Build binaries
echo "* building binaries..."
cross build --target=x86_64-unknown-linux-musl --release

# Build images
echo "* building images..."
docker compose --profile=full build

echo "* tagging images correctly..."
docker tag discord-proxy$IMAGE_SUFFIX  $DOCKER_REPO/tulpje/discord-proxy$IMAGE_SUFFIX
docker tag tulpje-handler$IMAGE_SUFFIX $DOCKER_REPO/tulpje/handler$IMAGE_SUFFIX
docker tag tulpje-gateway$IMAGE_SUFFIX $DOCKER_REPO/tulpje/gateway$IMAGE_SUFFIX
docker tag gateway-queue$IMAGE_SUFFIX  $DOCKER_REPO/tulpje/gateway-queue$IMAGE_SUFFIX

echo "* pushing images..."
docker push $DOCKER_REPO/tulpje/discord-proxy$IMAGE_SUFFIX
docker push $DOCKER_REPO/tulpje/handler$IMAGE_SUFFIX
docker push $DOCKER_REPO/tulpje/gateway$IMAGE_SUFFIX
docker push $DOCKER_REPO/tulpje/gateway-queue$IMAGE_SUFFIX

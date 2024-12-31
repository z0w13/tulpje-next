#!/usr/bin/env bash
set -euo pipefail

##############
#
# Run through a bunch of checks, tests, etc to see if we can actually deploy
#
##############

export RUSTFLAGS="-Dwarnings"

echo "* auditing dependencies..."
cargo audit

echo "* running clippy..."
cargo clippy

echo "* running tests (x86-unknown-linux-musl)..."
cross test --target=x86_64-unknown-linux-musl --release

echo "* building binaries (x86-unknown-linux-musl)..."
cross build --target=x86_64-unknown-linux-musl --release

echo "* building containers..."
docker compose --profile=full build

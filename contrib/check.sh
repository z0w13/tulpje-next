#!/usr/bin/env bash
set -euo pipefail

##############
#
# Run through a bunch of checks, tests, etc to see if we can actually deploy
#
##############

export RUSTFLAGS="-Dwarnings"

HOST_TARGET="$(rustc --version --verbose | pcregrep -o1 'host: (.*)')"

FEAT_PERMUTATIONS=(
  "amqp-amqprs"
  "amqp-lapin"
)

TARGETS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl")

echo "* auditing dependencies..."
cargo audit

for permutation in ${FEAT_PERMUTATIONS[@]}; do
  echo "* running clippy ($permutation)..."
  cargo clippy --no-default-features -F "$permutation" --quiet
done

for target in ${TARGETS[@]}; do
  # clean up the target/release folder otherwise some weird issues happen with GLIBC and serde
  rm -rf target/release

  if [[ "$target" = "$HOST_TARGET" ]]; then
    rustBin=cargo
  else
    rustBin=cross
  fi

  for permutation in ${FEAT_PERMUTATIONS[@]}; do
    echo "* building binaries ($target, $permutation)..."
    $rustBin build --target=$target --no-default-features -F "$permutation" --release --quiet

    echo "* running tests ($target, $permutation)..."
    $rustBin test --target=$target --no-default-features -F "$permutation" --release --quiet
  done
done

echo "* building containers..."
docker compose --profile=full build

#!/usr/bin/env fish

# Set unique image suffix (current UNIX timestamp)
set -x IMAGE_SUFFIX :(date +%s)

# Set shard count from command line argument or ask discord
if test (count $argv) -gt 0
  set -x SHARD_COUNT $argv[1]
else
  set -x SHARD_COUNT (cargo run -p tulpje-manager)
end

echo "* shard count: $SHARD_COUNT"
echo "* writing secrets from .env to file..."

for L in (cat .env | grep -vE '^(#|$)');
  set -l varName (string split -f1 "=" "$L")
  # substitute bash style expansion with fish style
  set -l varVal (string split -f2 -m1 "=" "$L" | sed 's/${/{$/g')

  # eval the variable so we can get the actual expanded value
  eval set -x $varName $varVal

  # Store each var in .env in a separate file
  echo $$varName > _secrets/(string split -f1 "=" "$L" | string lower);
end

# Build binaries
echo "* building binaries..."
cross build --target=x86_64-unknown-linux-musl --release

# Build images
echo "* building images..."
docker compose --profile=full build

# Deploy images
echo "* deploying..."
docker stack deploy --detach=false -c compose.swarm.yml tulpje-next-staging

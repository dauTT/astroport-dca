#!/usr/bin/env bash

set -e
set -o pipefail

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

docker run --rm -v "$projectPath":/code \
  --mount type=volume,source="$(basename "$projectPath")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6

echo "copy ../artifacts/astroport_dca_module.wasm to astroport_artifacts/"
cp ../artifacts/astroport_dca_module.wasm  astroport_artifacts/
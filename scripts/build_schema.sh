#!/usr/bin/env bash

set -e
set -o pipefail

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

for c in "$projectPath"/contracts/*; do
  if [[ "$c" != *"tokenomics" ]] && [[ "$c" != *"periphery" ]]; then
    
    echo "******************"$c"********************************"
      (cd $c && cargo schema)
  
  fi
done

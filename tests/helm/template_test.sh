#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CHART_DIR="$REPO_ROOT/helm/model2vec-serve"

render() {
  helm template model2vec-serve "$CHART_DIR" "$@"
}

echo "Running helm template tests..."

# Default values render a single model and the standard resources.
OUTPUT=$(render)
echo "$OUTPUT" | grep -q "kind: Deployment"
echo "$OUTPUT" | grep -q "kind: Service"
echo "$OUTPUT" | grep -q "kind: ConfigMap"
[ "$(echo "$OUTPUT" | grep -cE -- '- --model$')" -eq 1 ]

# Backward-compatible single-model override still works.
OUTPUT=$(render \
  --set model=minishlab/potion-base-2M \
  --set apiKey=secret)
echo "$OUTPUT" | grep -q "kind: Secret"
[ "$(echo "$OUTPUT" | grep -cE -- '- --model$')" -eq 1 ]
echo "$OUTPUT" | grep -q 'minishlab/potion-base-2M'

# Multi-model values render one --model flag per entry and a default model.
OUTPUT=$(render \
  --set "models[0]=minishlab/potion-base-2M" \
  --set "models[1]=minishlab/potion-multilingual-128M" \
  --set defaultModel=minishlab/potion-base-2M \
  --set apiKey=secret)
echo "$OUTPUT" | grep -q "kind: Secret"
[ "$(echo "$OUTPUT" | grep -cE -- '- --model$')" -eq 2 ]
echo "$OUTPUT" | grep -q 'minishlab/potion-base-2M'
echo "$OUTPUT" | grep -q 'minishlab/potion-multilingual-128M'
echo "$OUTPUT" | grep -q -- '- --default-model$'

echo "Helm chart validation passed."

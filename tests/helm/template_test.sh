#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CHART_DIR="$REPO_ROOT/helm/model2vec-serve"

echo "Running helm template..."
OUTPUT=$(helm template model2vec-serve "$CHART_DIR" \
  --set model=minishlab/potion-base-2M \
  --set apiKey=secret)

echo "Checking rendered resources..."
echo "$OUTPUT" | grep -q "kind: Deployment"
echo "$OUTPUT" | grep -q "kind: Service"
echo "$OUTPUT" | grep -q "kind: Secret"
echo "$OUTPUT" | grep -q "kind: ConfigMap"

echo "Helm chart validation passed."

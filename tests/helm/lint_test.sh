#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
CHART_DIR="$REPO_ROOT/helm/model2vec-serve"

echo "Running helm lint..."
helm lint "$CHART_DIR"

echo "Helm lint passed."

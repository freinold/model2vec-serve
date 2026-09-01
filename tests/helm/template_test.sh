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
  --set modelOwner=acme \
  --set apiKey=secret)
echo "$OUTPUT" | grep -q "kind: Secret"
[ "$(echo "$OUTPUT" | grep -cE -- '- --model$')" -eq 2 ]
echo "$OUTPUT" | grep -q 'minishlab/potion-base-2M'
echo "$OUTPUT" | grep -q 'minishlab/potion-multilingual-128M'
echo "$OUTPUT" | grep -q -- '- --default-model$'
echo "$OUTPUT" | grep -q -- '- --model-owner$'
echo "$OUTPUT" | grep -q 'acme'

# Chart metadata and default image reference are publish-ready. The expected
# values are derived from Chart.yaml so the test stays valid across automated
# chart bumps (chart version mirrors the app version).
CHART_VERSION="$(awk '/^version:/ {print $2}' "$CHART_DIR/Chart.yaml")"
# The helm.sh/chart label sanitizes the version ("+" becomes "_").
CHART_VERSION="${CHART_VERSION//+/_}"
APP_VERSION="$(awk -F'"' '/^appVersion:/ {print $2}' "$CHART_DIR/Chart.yaml")"
OUTPUT=$(render)
echo "$OUTPUT" | grep -q "helm.sh/chart: model2vec-serve-${CHART_VERSION}"
echo "$OUTPUT" | grep -q "app.kubernetes.io/version: \"${APP_VERSION}\""
echo "$OUTPUT" | grep -q "image: \"ghcr.io/freinold/model2vec-serve:${APP_VERSION}\""

# Model aliases render the MODEL_ALIAS env var as KEY=ALIAS pairs.
OUTPUT=$(render \
  --set modelAliases[0].key=minishlab/potion-base-2M \
  --set modelAliases[0].alias=base \
  --set modelAliases[1].key=minishlab/potion-multilingual-128M \
  --set modelAliases[1].alias=multi)
echo "$OUTPUT" | grep -q 'value: "minishlab/potion-base-2M=base,minishlab/potion-multilingual-128M=multi"'

# Persistence is disabled by default: no PVC, no models volume, no HOME override.
OUTPUT=$(render)
! echo "$OUTPUT" | grep -q "kind: PersistentVolumeClaim"
! echo "$OUTPUT" | grep -q "name: models"
! echo "$OUTPUT" | grep -q "name: HOME"

# Enabling persistence renders the claim and wires volume, mount, and HOME.
OUTPUT=$(render --set persistence.enabled=true)
echo "$OUTPUT" | grep -q "kind: PersistentVolumeClaim"
echo "$OUTPUT" | grep -q "name: model2vec-serve-models"
echo "$OUTPUT" | grep -qE 'storage: "?5Gi"?'
echo "$OUTPUT" | grep -q "ReadWriteOnce"
echo "$OUTPUT" | grep -q "claimName: model2vec-serve-models"
echo "$OUTPUT" | grep -q "mountPath: /models"
echo "$OUTPUT" | grep -A1 "name: HOME" | grep -qE 'value: "?/models"?'

# An existing claim is referenced without creating a PVC.
OUTPUT=$(render --set persistence.enabled=true --set persistence.existingClaim=my-models)
! echo "$OUTPUT" | grep -q "kind: PersistentVolumeClaim"
echo "$OUTPUT" | grep -q "claimName: my-models"

# Ingress is disabled by default.
OUTPUT=$(render)
! echo "$OUTPUT" | grep -q "kind: Ingress"

# Enabling the ingress renders rules, merged extra labels, and the service backend.
OUTPUT=$(render \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.extraLabels.environment=staging)
echo "$OUTPUT" | grep -q "kind: Ingress"
INGRESS_DOC=$(echo "$OUTPUT" | sed -n '/^kind: Ingress$/,/^---$/p')
echo "$INGRESS_DOC" | grep -q "environment: staging"
echo "$INGRESS_DOC" | grep -q "app.kubernetes.io/name: model2vec-serve"
echo "$INGRESS_DOC" | grep -q "host: embeddings.example.com"
echo "$INGRESS_DOC" | grep -q "pathType: Prefix"
echo "$INGRESS_DOC" | grep -q "name: http"

# TLS configuration renders a tls block.
OUTPUT=$(render \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=embeddings.example.com \
  --set ingress.tls[0].secretName=tls-secret \
  --set "ingress.tls[0].hosts[0]=embeddings.example.com")
INGRESS_DOC=$(echo "$OUTPUT" | sed -n '/^kind: Ingress$/,/^---$/p')
echo "$INGRESS_DOC" | grep -q "secretName: tls-secret"

echo "Helm chart validation passed."

#!/usr/bin/env bash
# Bump the Helm chart to mirror an app release and push the change to main.
#
# Called by the helm-chart-bump job in .github/workflows/release.yml after
# release-plz has created an app release (git tag + GitHub release). The push
# of the resulting commit triggers helm-release.yml, which publishes the chart
# to GHCR via chart-releaser.
#
# Required environment:
#   APP_VERSION - the released app version without the "v" prefix (e.g. 0.5.2)
set -euo pipefail

APP_VERSION="${APP_VERSION:?APP_VERSION must be set without the v prefix (e.g. 0.5.2)}"
CHART_NAME="model2vec-serve"
CHART_DIR="helm/${CHART_NAME}"
DOC_FILES=("docs/deployment/helm.md" "README.md" "helm/${CHART_NAME}/README.md")

# The chart version mirrors the app version. If a chart with that version was
# already published (e.g. a chart-only hotfix between app releases), bump the
# patch until a free version is found: chart-releaser tags every published
# chart as "<chart-name>-<version>".
chart_version="${APP_VERSION}"
while git rev-parse -q --verify "refs/tags/${CHART_NAME}-${chart_version}" >/dev/null 2>&1; do
  echo "Chart ${CHART_NAME}-${chart_version} already published, bumping patch"
  base="${chart_version%.*}"
  patch="${chart_version##*.}"
  chart_version="${base}.$((patch + 1))"
done
echo "Releasing chart ${CHART_NAME}-${chart_version} (appVersion ${APP_VERSION})"

# Chart.yaml: version mirrors the app version; appVersion is the image tag
# (bare semver, no v prefix - the deployment falls back to it via
# image.tag | default .Chart.AppVersion).
sed -i "s/^version: .*/version: ${chart_version}/" "${CHART_DIR}/Chart.yaml"
sed -i "s/^appVersion: .*/appVersion: \"${APP_VERSION}\"/" "${CHART_DIR}/Chart.yaml"

# Keep the documented install commands in sync with the published chart.
git add "${CHART_DIR}/Chart.yaml"
for doc in "${DOC_FILES[@]}"; do
  if [ -f "${doc}" ]; then
    sed -i "s/--version [0-9][0-9.]*/--version ${chart_version}/g" "${doc}"
    git add "${doc}"
  else
    echo "warning: expected doc file ${doc} not found, skipping" >&2
  fi
done

if git diff --cached --quiet; then
  echo "Nothing to commit; chart is already at ${chart_version}."
  exit 0
fi

helm lint "${CHART_DIR}"

git commit -m "chore(helm): release chart ${chart_version} with appVersion ${APP_VERSION}"

# Push to main; on a race with another commit, rebase once and retry.
if ! git push origin HEAD:main; then
  echo "Push rejected, rebasing onto origin/main and retrying"
  git fetch origin main
  git rebase origin/main
  git push origin HEAD:main
fi

echo "Pushed chart bump; helm-release.yml will publish chart ${CHART_NAME}-${chart_version}."

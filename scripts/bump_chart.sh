#!/usr/bin/env bash
# Bump the Helm chart to mirror an app release and push the change to main.
#
# Called by the helm-chart-bump job in .github/workflows/release.yml after
# release-plz has created an app release (git tag + GitHub release). The push
# of the resulting commit triggers helm-release.yml, which publishes the chart
# to GHCR via chart-releaser.
#
# Required environment:
#   APP_VERSION       - the released app version without the "v" prefix (e.g. 0.5.2)
#   RELEASE_PLZ_TOKEN - optional PAT; when set, the push is authenticated with
#                       it scoped to this script only (checkout does not persist
#                       credentials). Without it, plain git credentials are used.
set -euo pipefail

APP_VERSION="${APP_VERSION:?APP_VERSION must be set without the v prefix (e.g. 0.5.2)}"
CHART_NAME="model2vec-serve"
CHART_DIR="helm/${CHART_NAME}"
DOC_FILES=("docs/deployment/helm.md" "README.md" "helm/${CHART_NAME}/README.md" "specs/004-helm-chart-enhancements/contracts/publishing.md")

# Collision policy for the mirrored chart version:
# - No-op when the published chart at this version belongs to the same app
#   release (its Chart.yaml appVersion equals APP_VERSION): the job can be
#   re-run safely after a partial failure without publishing a duplicate.
# - Otherwise (the version was taken by a chart-only hotfix for a different
#   app version), bump the patch until a free version is found.
chart_version="${APP_VERSION}"
while git rev-parse -q --verify "refs/tags/${CHART_NAME}-${chart_version}" >/dev/null 2>&1; do
  existing_app_version="$(git show "refs/tags/${CHART_NAME}-${chart_version}:${CHART_DIR}/Chart.yaml" 2>/dev/null \
    | awk -F'"' '/^appVersion:/ {print $2}' || true)"
  if [ "${existing_app_version}" = "${APP_VERSION}" ]; then
    echo "Chart ${CHART_NAME}-${chart_version} is already published for app ${APP_VERSION}; nothing to do."
    exit 0
  fi
  echo "Chart version ${chart_version} was taken by a chart-only hotfix (appVersion ${existing_app_version:-unknown}), bumping patch"
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

# Keep every live version example in sync with the release:
# - helm install --version <chart-version> in the docs and both READMEs
# - docker pull/run ghcr.io/...:v<app-version> in the top-level README
git add "${CHART_DIR}/Chart.yaml"
for doc in "${DOC_FILES[@]}"; do
  if [ -f "${doc}" ]; then
    sed -i "s/--version [0-9][0-9.]*/--version ${chart_version}/g" "${doc}"
    git add "${doc}"
  else
    echo "warning: expected doc file ${doc} not found, skipping" >&2
  fi
done
sed -i "s|ghcr.io/freinold/model2vec-serve:v[0-9][0-9.]*|ghcr.io/freinold/model2vec-serve:v${APP_VERSION}|g" README.md
git add README.md

if git diff --cached --quiet; then
  echo "Nothing to commit; chart is already at ${chart_version}."
  exit 0
fi

helm lint "${CHART_DIR}"

git commit -m "chore(helm): release chart ${chart_version} with appVersion ${APP_VERSION}"

# Push to main with credentials scoped to this command (the token must not be
# persisted by checkout); on a race with another commit, rebase once and retry.
push_main() {
  if [ -n "${RELEASE_PLZ_TOKEN:-}" ]; then
    local header
    header="AUTHORIZATION: basic $(printf 'x-access-token:%s' "${RELEASE_PLZ_TOKEN}" | base64 | tr -d '\n')"
    git -c "http.https://github.com/.extraheader=${header}" push origin HEAD:main
  else
    git push origin HEAD:main
  fi
}
if ! push_main; then
  echo "Push rejected, rebasing onto origin/main and retrying"
  git fetch origin main
  git rebase origin/main
  push_main
fi

echo "Pushed chart bump; helm-release.yml will publish chart ${CHART_NAME}-${chart_version}."

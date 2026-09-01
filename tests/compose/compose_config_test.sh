#!/usr/bin/env bash
# Offline validation for docker-compose.yml, mirroring tests/helm/*.sh.
#
# Renders the compose configuration with `docker compose config` (no image
# pull, no network) and asserts the deployment contract from
# specs/006-docker-compose-support/contracts/compose.md.
#
# Per-story assertion blocks are added by the corresponding tasks; the script
# fails fast so a newly added block fails until its implementation lands.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
COMPOSE_FILE="$REPO_ROOT/docker-compose.yml"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}
pass() {
    echo "PASS: $*"
}

# Renders the config into $1 with the given env vars (name=value pairs after
# the target file). Optional service variables are explicitly unset first, and
# --env-file /dev/null disables automatic .env loading, so assertions are
# deterministic regardless of the caller's shell environment or a repo .env.
render() {
    local out="$1"
    shift
    env -u API_KEY -u MODEL_OWNER -u MODEL_ALIAS -u MAX_BATCH_SIZE \
        -u MAX_INPUT_LENGTH -u LOG_LEVEL -u REQUEST_TIMEOUT_SECONDS \
        -u MODEL -u DEFAULT_MODEL -u MODEL2VEC_IMAGE -u MODEL2VEC_PORT \
        -u MODEL2VEC_CACHE_DIR \
        docker compose --env-file /dev/null -f "$COMPOSE_FILE" config \
        > "$out" 2>/dev/null \
        || fail "docker compose config failed to render"
}

[ -f "$COMPOSE_FILE" ] || fail "docker-compose.yml not found at repository root"

TMP="$(mktemp)"
trap 'rm -f "$TMP" "$TMP_OVERRIDE"' EXIT
TMP_OVERRIDE="$(mktemp)"

render "$TMP"

# --- Global invariants (Phase 2) -------------------------------------------

if grep -qE '^version:' "$TMP"; then
    fail "obsolete top-level 'version:' key present"
fi
pass "no obsolete top-level version key"

grep -q 'restart: unless-stopped' "$TMP" \
    || fail "restart: unless-stopped missing"
pass "restart policy is unless-stopped"

grep -q 'stop_grace_period: 30s' "$TMP" \
    || fail "stop grace period is 30s"
pass "stop grace period is 30s"

# --- US1: one-command two-model launch (Phase 3) ----------------------------

grep -q '^  model2vec-serve:' "$TMP" \
    || fail "service 'model2vec-serve' missing"
pass "service model2vec-serve is defined"

grep -q 'image: ghcr.io/freinold/model2vec-serve:latest' "$TMP" \
    || fail "default image is not the published GHCR image"
pass "default image is ghcr.io/freinold/model2vec-serve:latest"

grep -q \
    'MODEL: minishlab/potion-multilingual-128M,minishlab/potion-code-16M-v2' \
    "$TMP" \
    || fail "MODEL does not contain exactly the two default models"
pass "MODEL serves multilingual (first/default) + code-v2"

grep -q 'DEFAULT_MODEL: null' "$TMP" \
    || fail "DEFAULT_MODEL must not be pinned (default = first MODEL entry)"
pass "DEFAULT_MODEL falls back to the first MODEL entry"

grep -q 'target: 8080' "$TMP" || fail "container target port is not 8080"
grep -q 'host_ip: 127.0.0.1' "$TMP" \
    || fail "host port is not bound to the loopback interface"
grep -q 'published: "8080"' "$TMP" || fail "host port does not default to 8080"
pass "port mapping defaults to loopback 127.0.0.1:8080"

MODEL2VEC_PORT=9090 docker compose --env-file /dev/null -f "$COMPOSE_FILE" \
    config > "$TMP_OVERRIDE" 2>/dev/null \
    || fail "config render with MODEL2VEC_PORT failed"
grep -q 'published: "9090"' "$TMP_OVERRIDE" \
    || fail "MODEL2VEC_PORT override is not honored"
grep -q 'host_ip: 127.0.0.1' "$TMP_OVERRIDE" \
    || fail "MODEL2VEC_PORT override must keep the loopback binding"
pass "MODEL2VEC_PORT override honored"

# --- US2: model cache persistence (Phase 4) ---------------------------------

grep -q "source: $REPO_ROOT/models$" "$TMP" \
    || fail "volume source does not default to ./models"
grep -q 'target: /models$' "$TMP" \
    || fail "volume is not mounted at /models"
pass "cache volume defaults to ./models:/models"

grep -q 'HOME: /models' "$TMP" \
    || fail "container HOME is not redirected to /models"
pass "HOME is redirected to /models"

if grep -q 'HF_HOME' "$TMP"; then
    fail "HF_HOME must not be used (ineffective with hf-hub 0.4.3 sync API)"
fi
pass "no ineffective HF_HOME variable"

MODEL2VEC_CACHE_DIR=/tmp/m2v-cache-override-test docker compose \
    --env-file /dev/null -f "$COMPOSE_FILE" config > "$TMP_OVERRIDE" \
    2>/dev/null || fail "config render with MODEL2VEC_CACHE_DIR failed"
grep -q 'source: /tmp/m2v-cache-override-test$' "$TMP_OVERRIDE" \
    || fail "MODEL2VEC_CACHE_DIR override is not honored"
pass "MODEL2VEC_CACHE_DIR override honored"

# --- US3: customization without editing the file (Phase 5) ------------------

# Optional service variables must be ABSENT (not empty) when unset. The
# short syntax renders as `VAR: null` in config output, which compose
# resolves to "not set in the container" (verified empirically); a non-null
# value for an unset variable would be the empty-string bug.
for var in DEFAULT_MODEL API_KEY MODEL_OWNER MODEL_ALIAS MAX_BATCH_SIZE \
    MAX_INPUT_LENGTH LOG_LEVEL REQUEST_TIMEOUT_SECONDS; do
    if grep "$var:" "$TMP" | grep -qv ': null$'; then
        fail "$var has a value but is unset (must be absent, never empty)"
    fi
done
pass "optional service variables absent when unset"

# Setting one passes through verbatim.
API_KEY=probe-key DEFAULT_MODEL=probe-model docker compose --env-file /dev/null \
    -f "$COMPOSE_FILE" config > "$TMP_OVERRIDE" 2>/dev/null \
    || fail "config render with variable probes failed"
grep -q 'API_KEY: probe-key' "$TMP_OVERRIDE" \
    || fail "API_KEY is not passed through when set"
grep -q 'DEFAULT_MODEL: probe-model' "$TMP_OVERRIDE" \
    || fail "DEFAULT_MODEL is not passed through when set"
pass "optional service variables pass through verbatim when set"

# .env.example documents the full variable surface.
ENV_EXAMPLE="$REPO_ROOT/.env.example"
[ -f "$ENV_EXAMPLE" ] || fail ".env.example missing"
for var in MODEL2VEC_IMAGE MODEL2VEC_PORT MODEL2VEC_CACHE_DIR MODEL \
    DEFAULT_MODEL API_KEY MODEL_OWNER MODEL_ALIAS MAX_BATCH_SIZE \
    MAX_INPUT_LENGTH LOG_LEVEL REQUEST_TIMEOUT_SECONDS; do
    grep -q "$var" "$ENV_EXAMPLE" || fail ".env.example does not document $var"
done
pass ".env.example documents every variable"

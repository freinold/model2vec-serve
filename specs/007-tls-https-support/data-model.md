# Data Model: TLS/HTTPS Serving with End-to-End Encryption

No persistent data is introduced. All entities are startup-time configuration
and deployment wiring. Field tables follow the multi-model data-model
conventions (`specs/003-multi-model-serving/data-model.md`).

## Entities

### 1. TLS Configuration (operator-facing)

Controls whether the process speaks HTTPS. Binary per process: enabled iff
both paths are configured.

| Field | Source | Type | Default | Validation | Notes |
|-------|--------|------|---------|------------|-------|
| `tls_cert` | `--tls-cert` / `TLS_CERT` | path | none (unset) | see TLS Certificate Package | Certificate file (PEM, full chain allowed) |
| `tls_key` | `--tls-key` / `TLS_KEY` | path | none (unset) | see TLS Certificate Package | Private key file (PEM, unencrypted) |
| enabled (derived) | — | bool | `false` | all-or-nothing rule | `true` iff both paths present |

State transitions (process lifecycle):

| From | Event | To |
|------|-------|----|
| HTTP (default) | both paths set + valid | HTTPS (single port, same state object) |
| HTTP (default) | only one path set | startup failure (FR-003) |
| HTTPS | any validation failure | startup failure (FR-004) |
| HTTPS | expired-but-valid cert | HTTPS with prominent `warn!` (spec default) |
| HTTPS | mounted files rotated while running | unchanged until restart (documented) |

### 2. TLS Certificate Package (validated artifact)

The parsed, validated result of reading the two configured files. Exists only
in memory at startup.

| Field | Source | Type | Validation | Notes |
|-------|--------|------|------------|-------|
| certificate chain | `tls_cert` file | PEM → DER certs | parseable X.509; ≥ 1 cert | leaf + optional intermediates passed through to rustls |
| private key | `tls_key` file | PEM → DER key | parseable; unencrypted; matches leaf | encrypted-key PEM headers rejected with dedicated error |
| pair match | derived | bool | `with_single_cert` acceptance | mismatch → dedicated startup error |
| subject | derived from leaf | X.509 subject | — | logged at startup (FR-007) |
| issuer | derived from leaf | X.509 issuer | — | logged at startup (FR-007) |
| not_after | derived from leaf | timestamp | expiry check | expired → `warn!`, not failure |

Relationships: exactly one TLS Certificate Package per enabled TLS
Configuration; absent entirely when TLS is disabled.

### 3. TLS Listener (runtime)

| Field | Source | Type | Notes |
|-------|--------|------|-------|
| bind address | existing `host`/`port` | socket addr | unchanged — single listener serves HTTP or HTTPS |
| protocol | derived from TLS Configuration | HTTP/1.1+TLS | TLS 1.2/1.3 only (rustls); HTTP contracts byte-identical |
| shutdown semantics | existing signal handling | graceful | TLS path mirrors current `with_graceful_shutdown` behavior |

### 4. Deployment TLS Binding (chart)

Chart-level wiring mapping an operator secret to the container's configured
file paths. Disabled by default; no rendered output when disabled.

| Field | Source | Type | Default | Validation | Notes |
|-------|--------|------|---------|------------|-------|
| `tls.enabled` | values.yaml | bool | `false` | — | master switch |
| `tls.existingSecret` | values.yaml | string | `""` | required (template fail) when enabled | operator-managed secret |
| `tls.certKey` | values.yaml | string | `tls.crt` | non-empty | secret key with cert chain |
| `tls.keyKey` | values.yaml | string | `tls.key` | non-empty | secret key with private key |
| `tls.mountPath` | values.yaml | path | `/etc/model2vec-serve/tls` | — | container mount point |
| volume + mount | derived | — | — | secret `optional: false` | missing secret fails pod scheduling |
| probe scheme | derived | `HTTP`/`HTTPS` | HTTP | HTTPS when `tls.enabled` | kubelet skips cert verification |

Relationships: binds exactly one secret → one volume → two file paths → the
service's `--tls-cert`/`--tls-key` args.

## Invariants

- INV-1: At most one transport mode per process: plain HTTP (no TLS config)
  or HTTPS (both paths configured). No dual-port dual-protocol exposure.
- INV-2: No endpoint is reachable over plain HTTP when TLS is enabled.
- INV-3: TLS enablement never changes request/response bodies, status codes,
  auth behavior, or error JSON — only the transport.
- INV-4: Private key material never appears in configuration values (chart
  values, env vars), logs, or error messages.
- INV-5: Default chart render (all values default) is byte-identical to the
  pre-feature chart.

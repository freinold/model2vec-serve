# Contract: Service TLS Configuration

This contract defines the operator-facing TLS configuration of the
model2vec-serve binary. **HTTP API contracts are unchanged**: every endpoint
(`POST /v1/embeddings`, `GET /v1/models`, `POST /embed`, `GET /info`,
`GET /health`, `GET /ready`, `GET /metrics`, `/docs`) keeps its exact
request/response shape, status codes, authentication behavior, and error JSON
in both transports. Only the transport differs.

## Configuration options

| Option | Flag | Environment | Type | Default | Description |
|--------|------|-------------|------|---------|-------------|
| TLS certificate | `--tls-cert <path>` | `TLS_CERT` | file path | unset | PEM file with the X.509 certificate (leaf + optional intermediate chain) |
| TLS private key | `--tls-key <path>` | `TLS_KEY` | file path | unset | PEM file with the unencrypted private key |

Rules:

- **R-1 (opt-in)**: With neither option set, the service serves plain HTTP on
  `host:port` exactly as current releases (all other options unaffected).
- **R-2 (all-or-nothing)**: TLS is enabled iff both options are set. Setting
  exactly one is a startup error (see error matrix E1).
- **R-3 (paths only)**: Certificate/key are accepted as file paths only;
  inline PEM via env or CLI is not supported.
- **R-4 (single listener)**: When TLS is enabled, `host:port` serves HTTPS
  only; no plain-HTTP listener exists in that mode.
- **R-5 (protocol floor)**: Only TLS 1.2 and TLS 1.3 are negotiated; older
  versions are refused at handshake.

## Startup error matrix

All TLS errors fail startup before the port is bound, within seconds, with
messages that name the offending item and the remedy (SC-004). Messages are
stable identifiers for tests and must never include key material or file
contents beyond the configured path.

| ID | Condition | Required error message shape |
|----|-----------|------------------------------|
| E1 | Exactly one of cert/key set | `TLS requires both --tls-cert and --tls-key; only <one> was provided` |
| E2 | File missing/unreadable | `failed to read TLS <certificate|private key> file '<path>': <os reason>` |
| E3 | Unparseable content | `TLS <certificate|private key> file '<path>' is not a valid PEM <X.509 certificate|private key>` |
| E4 | Encrypted private key | `TLS private key file '<path>' is encrypted; encrypted keys are not supported` |
| E5 | Key/cert mismatch | `TLS certificate in '<cert path>' does not match the private key in '<key path>'` |
| E6 | Expired certificate | non-fatal: `warn!` log `TLS certificate '<path>' expired on <date>; proceeding` (service starts) |

Startup success logging (TLS enabled): one `info!` line stating TLS is
enabled plus the certificate's subject, issuer, and expiry date. No private
key material is ever logged (E6-style metadata only).

## Transport behavior

| Aspect | Contract |
|--------|----------|
| Endpoint coverage | All endpoints over TLS when enabled; none fall back to plain HTTP |
| Response identity | Same bytes over HTTP and HTTPS for identical requests (SC-002) |
| Auth | `Authorization: Bearer` behavior identical in both transports |
| Correlation/metrics | `x-request-id` echo and Prometheus metrics unchanged |
| Graceful shutdown | SIGINT/SIGTERM drain in-flight HTTPS connections, mirroring current behavior |
| Client compatibility | Standard HTTPS clients work by changing the scheme to `https://` (SC-002) |

## Environment variable compatibility

`TLS_CERT` / `TLS_KEY` compose with all existing env vars (`HOST`, `PORT`,
`MODEL`, `API_KEY`, …) with no interaction beyond the rules above. Docker
Compose / plain-binary operators enable TLS by mounting cert/key files and
setting both env vars (documented in `docs/`).

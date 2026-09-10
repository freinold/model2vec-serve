# Quickstart: TLS/HTTPS Serving with End-to-End Encryption

End-to-end validation for spec
[spec.md](./spec.md). Error identifiers (E1–E6) and option semantics are
defined in [contracts/configuration.md](./contracts/configuration.md); chart
behavior in [contracts/chart-values.md](./contracts/chart-values.md).

## Prerequisites

- Rust toolchain (MSRV 1.85) or the project's container image
- `openssl` (or any tool that produces a cert/key PEM pair)
- A Kubernetes cluster + `kubectl` + `helm` for the deployment scenario
  (section 4); `kind` works for local validation

## 1. Generate a test certificate pair

```bash
mkdir -p /tmp/tls
openssl req -x509 -newkey rsa:2048 -nodes -days 30 \
  -keyout /tmp/tls/key.pem -out /tmp/tls/cert.pem \
  -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

## 2. Serve HTTPS and verify (User Story 1 / SC-001, SC-002)

```bash
cargo run --release -- \
  --model minishlab/potion-base-2M \
  --tls-cert /tmp/tls/cert.pem --tls-key /tmp/tls/key.pem --port 8443
```

Expected: startup log states TLS is enabled with subject, issuer, and expiry
(`E6`-style metadata), no key material anywhere in the log.

```bash
# HTTPS works for every endpoint class
# --cacert verifies the server certificate instead of accepting any
# certificate; the SANs from step 1 cover localhost and 127.0.0.1.
curl --cacert /tmp/tls/cert.pem https://localhost:8443/health
curl --cacert /tmp/tls/cert.pem https://localhost:8443/v1/models
curl --cacert /tmp/tls/cert.pem https://localhost:8443/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{"input":"hello","model":"minishlab/potion-base-2M"}'

# Plain HTTP to the TLS port fails at transport level (service stays healthy)
curl -s http://localhost:8443/health || echo "rejected as expected"
```

**Pass criteria**:

- All HTTPS calls return the documented response shapes
  (`specs/001-model2vec-embedding-api/contracts/`).
- The embeddings JSON over HTTPS equals the same request over a plain-HTTP
  run (`cargo run --release -- --model minishlab/potion-base-2M --port 8080`,
  compare with `curl http://localhost:8080/...`) — byte-identical responses
  (SC-002).

## 3. Backward compatibility + misconfiguration matrix (FR-002, US3, SC-004)

Run each case; every failure must appear **within seconds**, name the item,
and exit before the port binds (the E6 row's `curl -k` is deliberate: an
expired certificate cannot pass verification, which is the behavior under
test):

| Case | Command | Expected |
|------|---------|----------|
| HTTP unchanged | `cargo run --release -- --port 8080` | plain HTTP exactly as before, no TLS logs |
| Cert only (E1) | `--tls-cert /tmp/tls/cert.pem` | `TLS requires both --tls-cert and --tls-key; ...` |
| Key only (E1) | `--tls-key /tmp/tls/key.pem` | same shape, names the missing cert |
| Missing file (E2) | `--tls-cert /nope.pem --tls-key /tmp/tls/key.pem` | `failed to read TLS certificate file '/nope.pem': ...` |
| Garbage file (E3) | `echo hello > /tmp/tls/bad.pem` + paths | `... is not a valid PEM ...` |
| Mismatched pair (E5) | generate a second key, pair cert1+key2 | `... does not match the private key ...` |
| Encrypted key (E4) | `openssl req -newkey rsa:2048 -aes256 -passout pass:x ...` | `... encrypted keys are not supported` |
| Expired cert (E6) | `openssl req -x509 -newkey rsa:2048 -nodes -keyout expired.key -out expired.pem -subj "/CN=old" -not_before 20250101000000Z -not_after 20250201000000Z` (requires OpenSSL ≥ 3.1; on older versions generate a past-dated pair with your platform's tooling — the rcgen fixture in `tests/common/mod.rs` shows one way) | `warn!` names expiry, service still starts |

No TLS mode may print private key contents anywhere (FR-007).

## 4. End-to-end encrypted Kubernetes deployment (User Story 2, SC-003)

```bash
kubectl create secret tls m2v-tls \
  --cert=/tmp/tls/cert.pem --key=/tmp/tls/key.pem

helm install m2v ./helm/model2vec-serve \
  --set models=minishlab/potion-base-2M \
  --set tls.enabled=true \
  --set tls.existingSecret=m2v-tls
```

**Pass criteria**:

- `helm template` with default values is byte-identical to the pre-feature
  chart (run the chart's template test suite).
- With the values above, the rendered Deployment mounts the secret, passes
  `--tls-cert/--tls-key`, and probes use `scheme: HTTPS`.
- `kubectl port-forward` + `curl --cacert /tmp/tls/cert.pem https://...` reaches the pod over TLS (verified against the certificate, not blindly accepted).
- Passthrough ingress (nginx example in `helm/model2vec-serve/README.md`):
  from outside the cluster, `openssl s_client -connect <edge>:443 -servername
  <host>` shows the **application's** certificate chain end to end, proving
  the edge did not terminate TLS (SC-003).
- Enabling `tls.enabled` without `existingSecret` fails `helm lint`/install
  with a template error.

## 5. Performance sanity (SC-005)

Run the existing embedding benchmark over plain HTTP and over the TLS
listener (same build, loopback) and compare steady-state p99/throughput;
the delta must stay within 10%. Exact commands are recorded with the bench
during implementation (constitution: reproducible benchmark invocations).

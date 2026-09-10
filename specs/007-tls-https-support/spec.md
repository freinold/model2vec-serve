# Feature Specification: TLS/HTTPS Serving with End-to-End Encryption

**Feature Branch**: `007-tls-https-support`

**Created**: 2026-09-10

**Status**: Draft

**Input**: User description: "model2vec-serve should optionally support a ssl cert+key package to provide the service via HTTPs and support e2e encryption up to the application inside the container."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Serve the API over HTTPS using an operator-provided certificate and key (Priority: P1)

A platform operator who must protect embedding traffic on an untrusted network
provides a TLS certificate and its matching private key through configuration
when starting the service. The service then accepts HTTPS connections on its
existing port, and any standard HTTPS client can call every existing endpoint
(`/v1/embeddings`, `/v1/models`, `/embed`, `/info`, health, readiness,
metrics, docs) exactly as before — only the URL scheme changes. When the
operator provides no TLS configuration, the service behaves exactly as it does
today over plain HTTP, so existing deployments are unaffected.

**Why this priority**: This is the core of the feature. Without the ability to
serve HTTPS from the application itself, end-to-end encryption to the process
inside the container is impossible and nothing else in this feature matters.

**Independent Test**: Can be fully tested by starting the service with a valid
certificate/key pair and confirming an HTTPS client completes requests to all
endpoints, then restarting the service without TLS configuration and
confirming plain HTTP works unchanged; it delivers transport-level encryption
under operator control.

**Acceptance Scenarios**:

1. **Given** the operator starts the service with valid certificate and key
   paths configured, **When** a client requests the model list or embeddings
   over HTTPS, **Then** the TLS connection succeeds and the response is
   identical to the plain-HTTP response for the same request.
2. **Given** the service is running with TLS enabled, **When** any existing
   endpoint is called over HTTPS (embedding endpoints, health, readiness,
   metrics, interactive docs), **Then** each endpoint responds as it does over
   HTTP today, including authentication and error behavior.
3. **Given** the operator starts the service with no TLS configuration,
   **When** clients connect over plain HTTP, **Then** the service behaves
   exactly as in current releases (no new required settings, no changed
   defaults).
4. **Given** TLS is enabled, **When** a client negotiates TLS, **Then** the
   connection uses a modern TLS version (1.2 or newer) and older protocol
   versions are refused.

---

### User Story 2 - End-to-end encrypted containerized deployment (Priority: P2)

A platform team deploys model2vec-serve in Kubernetes with a requirement that
traffic is encrypted on every network hop — from the client, through the
cluster edge, all the way to the application process inside the container.
The deployment wiring lets the operator supply the certificate and key as
files (for example from a Kubernetes secret) so the application terminates TLS
itself, with the cluster edge routing encrypted traffic through without
termination. Teams that prefer edge termination keep today's option of putting
TLS termination in front of a plain-HTTP deployment; TLS remains optional per
deployment.

**Why this priority**: The stated goal is end-to-end encryption "up to the
application inside the container", which only exists in practice when the
deployment can deliver cert/key files to the container and route encrypted
traffic to it. This is the primary production value of the feature after the
core capability exists.

**Independent Test**: Can be fully tested by deploying with the TLS values
into a Kubernetes cluster whose edge is configured to pass encrypted traffic
through, then completing an HTTPS request from outside the cluster and
verifying no plaintext hop exists between edge and pod; it delivers encryption
that reaches the application process.

**Acceptance Scenarios**:

1. **Given** deployment values that reference a secret containing the
   certificate and key, **When** the chart is deployed, **Then** the files are
   available to the application at configured paths inside the container and
   the application serves HTTPS.
2. **Given** a deployment configured for end-to-end encryption, **When** an
   external client makes an HTTPS request that traverses the cluster edge,
   **Then** the edge forwards the encrypted traffic without terminating TLS
   and the application inside the container performs the decryption.
3. **Given** an operator who deploys without any TLS values, **When** the
   chart is deployed or upgraded, **Then** the result is functionally
   identical to current releases (plain HTTP, no extra mounts, no new required
   values).
4. **Given** an operator who already terminates TLS at the edge (or a load
   balancer), **When** they upgrade, **Then** they can continue doing so
   without change, because application-side TLS is optional.

---

### User Story 3 - Fail-fast, actionable TLS configuration diagnostics (Priority: P3)

An operator who makes a TLS mistake — a missing file, a key that does not
match the certificate, a corrupt file, or a passphrase-protected key — gets an
immediate, specific startup failure that names the offending item and the
remedy, instead of a service that starts and then fails connections silently.
When TLS is configured correctly, the startup log confirms TLS is enabled and
shows the certificate's subject, issuer, and expiry date so operators can plan
renewals. Private key material and certificate contents are never logged.

**Why this priority**: Diagnostics do not add capability but determine how
painful misconfiguration is in production; they are valuable only once TLS
serving exists.

**Independent Test**: Can be fully tested by starting the service once per
misconfiguration class (missing file, invalid content, key/cert mismatch,
encrypted key, cert-only, key-only) and verifying each startup fails within
seconds with a distinct, actionable message, then starting with a valid pair
and checking the logged certificate metadata; it delivers fast,
self-explanatory failure recovery.

**Acceptance Scenarios**:

1. **Given** a TLS configuration referencing a file that does not exist or is
   unreadable, **When** the service starts, **Then** startup fails within
   seconds with an error identifying the file and the missing condition.
2. **Given** a certificate and a private key that do not form a matching pair,
   **When** the service starts, **Then** startup fails with an explicit
   mismatch error.
3. **Given** a file that cannot be parsed as the expected certificate or key
   format, **When** the service starts, **Then** startup fails with an error
   stating which file is invalid and which format is expected.
4. **Given** only a certificate or only a key is configured, **When** the
   service starts, **Then** startup fails with an error naming the missing
   piece (all-or-nothing configuration).
5. **Given** a valid TLS configuration, **When** the service starts, **Then**
   the startup log states TLS is enabled and reports the certificate's
   subject, issuer, and expiry, and no private key material appears in any
   log.

---

### Edge Cases

- What happens when the configured certificate has expired but is otherwise
  valid? The service starts with a prominent warning naming the expiry, so a
  temporary certificate-monitoring gap does not cause an outage; invalid,
  unparseable, or mismatched files still fail fast.
- What happens when a client connects with plain HTTP to the TLS-enabled
  port? The connection is rejected at the transport layer, the service logs
  the failed connection, and the service remains healthy and serving other
  clients.
- What happens when the certificate lacks intermediate chain certificates?
  The service starts, but browser-style clients may fail verification; the
  documentation states that the full chain must be supplied.
- What happens when the private key is encrypted with a passphrase?
  Startup fails with a clear message that encrypted keys are not supported.
- What happens when the mounted certificate/key files are rotated (for
  example by a certificate manager) while the service is running? The service
  keeps serving with the certificate it loaded at startup; renewal is applied
  by a rolling restart, and the documentation says so explicitly.
- What happens when operational endpoints are scraped or probed while TLS is
  enabled? Health, readiness, and metrics are served on the same TLS listener;
  the documentation explains how to configure TLS-aware probes and monitoring.
- What happens when extremely large or binary garbage is supplied as
  certificate/key files? The files fail parsing at startup with the same
  clear diagnostics as any other invalid content.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow operators to enable HTTPS by providing paths
  to a TLS certificate and its matching private key through configuration,
  available both as command-line options and as equivalent environment
  variables, consistent with all other service settings.
- **FR-002**: When TLS is not configured, System MUST serve plain HTTP with
  behavior and defaults identical to current releases (opt-in feature, zero
  impact on existing deployments).
- **FR-003**: System MUST treat TLS configuration as all-or-nothing: if
  exactly one of certificate or key is provided, startup MUST fail with an
  error that names the missing piece.
- **FR-004**: System MUST validate the TLS files at startup — existence,
  readability, parseability, and that certificate and key form a matching
  pair — and MUST fail fast with actionable, operator-facing error messages on
  any failure.
- **FR-005**: When TLS is enabled, System MUST serve every existing endpoint
  over TLS on the same configured port, with unchanged request/response
  contracts, authentication behavior, and error bodies; no endpoint may be
  reachable only over plain HTTP.
- **FR-006**: System MUST accept the certificate and key as file paths (not
  inline configuration values) so they can be provisioned as mounted files —
  for example from platform secrets — and MUST refuse to read key material
  from command-line or environment values directly.
- **FR-007**: System MUST NOT write private key material or full certificate
  contents to logs; startup logging of TLS state is limited to enablement
  status plus certificate subject, issuer, and expiry date.
- **FR-008**: System MUST negotiate only TLS 1.2 or newer; connection attempts
  using older protocol versions MUST be rejected.
- **FR-009**: The deployment chart MUST provide values that supply the
  certificate and key files to the container (from a Kubernetes secret or
  another mounted source) so the application itself terminates TLS and
  traffic can be encrypted end to end to the process inside the container.
- **FR-010**: The deployment chart MUST support an edge-routing mode that
  forwards encrypted traffic to the pod without terminating TLS, and MUST
  document this mode as the way to achieve end-to-end encryption.
- **FR-011**: The deployment chart MUST continue to produce a fully
  functional plain-HTTP deployment when TLS values are absent, with no new
  required values and no changed defaults for existing users.
- **FR-012**: System MUST remain compatible with deployments that terminate
  TLS at the edge (load balancer or ingress) in front of a plain-HTTP
  service; TLS support MUST be optional per deployment.
- **FR-013**: Project documentation MUST describe every new configuration
  option, the accepted file formats, the supported protocol versions, and a
  worked end-to-end encrypted deployment example; chart documentation MUST be
  updated in the same change.
- **FR-014**: TLS behavior MUST be identical for containerized and
  non-containerized runs of the same version, so operators can validate
  configurations outside the cluster.

### Key Entities *(include if feature involves data)*

- **TLS certificate package**: The operator-supplied pair of files — an
  X.509 certificate (full chain encouraged, intermediate chain optional) and
  its matching private key — provided as two file paths; carries observable
  metadata (subject, issuer, validity window) surfaced in startup logs.
- **TLS configuration**: The operator-facing setting that controls whether
  the service speaks HTTPS; consists of certificate path, key path, and a
  defaulted minimum protocol version; binary enabled/disabled per process
  (enabled iff both files are configured).
- **Deployment TLS binding**: The chart-level wiring that maps an
  operator-provided secret or volume to configured file paths inside the
  container and selects edge routing (passthrough for end-to-end encryption,
  edge termination otherwise).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can enable HTTPS by changing only configuration (no
  code or image change) and receive a successful HTTPS response within 5
  minutes of obtaining the certificate and key files.
- **SC-002**: 100% of existing API clients work over HTTPS by changing only
  the URL scheme from HTTP to HTTPS, with response bodies identical to the
  HTTP responses for the same requests.
- **SC-003**: In a deployment configured for end-to-end encryption, 100% of
  client-to-application traffic is encrypted on every network hop (verified
  by observing no plaintext HTTP traffic between the cluster edge and the
  application container).
- **SC-004**: 100% of TLS misconfiguration classes (missing file, unreadable
  file, invalid content, key/cert mismatch, single-sided configuration) cause
  startup to fail within 5 seconds with an error naming the failing item and
  the remedy.
- **SC-005**: Steady-state HTTPS throughput and p99 latency are within 10% of
  plain HTTP under identical load, so encryption does not become a
  performance bottleneck.
- **SC-006**: An existing deployment upgrading to a release containing this
  feature without TLS values experiences zero behavior change: no new
  required values, no new default mounts, and identical plain-HTTP serving.

## Assumptions

- Single-listener model: the configured port serves either plain HTTP
  (default) or HTTPS (when TLS is configured); simultaneously exposing both
  protocols on separate ports is out of scope for this feature.
- Server-side TLS only: mutual TLS (client certificate verification) is out
  of scope for the initial version; it can be a follow-up feature.
- Accepted formats: PEM-encoded X.509 certificate (full chain encouraged)
  and an unencrypted private key; passphrase-protected keys are unsupported
  and rejected with a clear error.
- No hot certificate reload: rotation is performed by updating the mounted
  files and rolling the deployment; continuous reload-on-change is a possible
  follow-up, not part of this feature.
- Minimum protocol version is TLS 1.2 (TLS 1.3 supported where available);
  no cipher-suite customization surface in this version beyond defaults.
- An expired but otherwise valid certificate starts with a prominent warning
  rather than a refusal, prioritizing availability; this default is
  documented so operators can plan monitoring.
- Health, readiness, and metrics run on the same TLS listener when TLS is
  enabled; TLS-aware probe/monitoring configuration is a documented operator
  responsibility, not a code concern.
- Edge TLS termination in front of a plain-HTTP service remains a fully
  supported alternative; passthrough is recommended only when end-to-end
  encryption to the application is required.
- The container image needs no changes beyond accepting the configured file
  paths; the chart change is additive wiring plus documentation.

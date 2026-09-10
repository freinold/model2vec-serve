---
description: "Task list for TLS/HTTPS Serving with End-to-End Encryption"
---

# Tasks: TLS/HTTPS Serving with End-to-End Encryption

**Input**: Design documents from `/specs/007-tls-https-support/`

**Prerequisites**: plan.md (required), spec.md (required for user stories),
research.md, data-model.md, contracts/configuration.md,
contracts/chart-values.md, quickstart.md

**Tests**: Included. The constitution (Principle II) requires automated tests
for every behavior. Each story phase writes its tests FIRST and confirms they
FAIL (RED) before the implementation that satisfies them. For Rust compile-
time features, RED means the test does not compile (missing field/function) —
that is a valid failing state; do not write the implementation before the
test exists.

**Organization**: Tasks are grouped by user story to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single Rust project at repository root (per plan.md Project Structure):
`src/` (new `src/tls.rs`; modified `src/config.rs`, `src/main.rs`),
`helm/model2vec-serve/`, `tests/` (new `tests/tls_integration.rs`,
`tests/tls_validation.rs`; extended `tests/common/mod.rs`,
`tests/config_unit.rs`, `tests/helm/template_test.sh`), `docs/`.

Error identifiers (E1–E6) below refer to
[contracts/configuration.md](./contracts/configuration.md) → Startup error
matrix. Use those exact message shapes.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Dependency groundwork everything else builds on

- [X] T001 Add TLS dependencies to `Cargo.toml` and verify the baseline
  - Runtime deps: `axum-server = { version = "0.7", features = ["tls-rustls"] }`
    and `x509-parser = "0.18"` (features: default PEM parsing) — justified in
    research.md and Complexity Tracking; no other runtime deps.
  - Dev deps: `rcgen` (certificate fixture generation) and `reqwest` with
    default-features off + `rustls-tls` + `json` (HTTPS test client that
    accepts invalid certs for self-signed fixtures).
  - Run `cargo build && cargo clippy --all-targets --all-features -- -D
    warnings` — baseline must be clean before any feature code; if
    aws-lc-rs builds cause friction in the container toolchain, fall back to
    `tls-rustls-no-provider` + explicit `ring` provider install (research.md
    stack decision, fallback clause) and record the deviation in
    `research.md`.

**Checkpoint**: Toolchain compiles with the TLS stack available; no feature
code yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared test infrastructure used by US1 and US3 tests

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Add TLS certificate fixture helpers to `tests/common/mod.rs`
  - `rcgen`-based helpers writing PEM files into a `tempfile::TempDir` and
    returning their paths (pattern: existing `tests/common/mod.rs` helpers):
    `tls_pair()` (self-signed valid pair with `localhost` DNS SAN + `127.0.0.1`
    IP SAN, plus the leaf DER for pinned-root clients),
    `mismatched_pair()` (cert from one key + a second, different key),
    `expired_pair()`, `garbage_file()` (valid filename, non-PEM bytes), and
    `encrypted_key_pair()` (an encrypted PKCS#8 key produced via `openssl`
    when available, falling back to a PEM-label-correct stub otherwise —
    both satisfy the label-driven E4 detection contract, so the E4 test is
    always runnable without OpenSSL).
  - Helpers must not be `pub` outside `tests/` scope conventions already
    used; keep clippy pedantic-clean (no `unwrap` — propagate results).
  - Smoke-check: a tiny test in `tests/common/mod.rs`'s test module or the
    first US1 test run compiles and creates the fixtures.

**Checkpoint**: Fixture helpers compile; US1 tests can request cert pairs.

---

## Phase 3: User Story 1 — Serve the API over HTTPS using an operator-provided certificate and key (Priority: P1) 🎯 MVP

**Goal**: With `--tls-cert` + `--tls-key`, the existing single port serves
HTTPS for every endpoint with byte-identical responses; without them, plain
HTTP is unchanged.

**Independent Test**: Start the service with a valid pair (fixture or
quickstart.md §1 certs) and call `/health`, `/v1/models`, `/embed` over
HTTPS; restart without TLS options and confirm plain HTTP is unchanged
(quickstart.md §2). No Helm or docs needed.

### Tests for User Story 1 (write first, confirm RED) ⚠️

- [X] T003 [P] [US1] Add TLS configuration tests to `tests/config_unit.rs`
  - Follow existing `config_unit.rs` patterns (CLI args + env parsing):
    `TLS_CERT`/`TLS_KEY` env aliases populate `tls_cert`/`tls_key` fields;
    both default to `None`; only one set → the all-or-nothing check reports
    E1; neither set → TLS disabled.
  - RED: fields do not exist yet — compile failure is the expected failing
    state (see Tests note above).
- [X] T004 [P] [US1] Create HTTPS end-to-end tests in `tests/tls_integration.rs`
  - Pattern: existing `*_integration.rs` tests (spawn the real binary or
    in-process app with `tokio`), but on a real TCP listener at
    `127.0.0.1:0` with TLS enabled using `tests/common` fixtures:
    (a) HTTPS request to `/health`, `/v1/models`, `/embed` succeeds via
    `reqwest` (client pinned to the fixture root, never accepting invalid
    certs) with the documented response shapes
    (`specs/001-model2vec-embedding-api/contracts/`);
    (b) response bytes for `/embed` equal a plain-HTTP run of the same build
    (contract identity, SC-002);
    (c) sending plain-HTTP bytes to the TLS port fails at transport level
    while the service keeps serving subsequent HTTPS requests (edge case);
    (d) startup log line states TLS is listening (no key material).
  - GREEN state additionally covers (FR-005 full contract): status/body
    parity for `/health`, `/ready`, `/info`, `/v1/models`, `/docs`,
    `/metrics`, `/embed`, `/v1/embeddings` (success AND unknown-model error
    body), and the TEI per-model routes; API-key auth parity (401/401/200
    for missing/wrong/correct key) with `/health`, `/ready`, `/metrics`
    staying public over both transports.
  - RED: `--tls-cert` does not exist yet.

### Implementation for User Story 1

- [X] T005 [US1] Add TLS options to `src/config.rs`
  - `tls_cert: Option<PathBuf>` via `#[arg(long = "tls-cert", env =
    "TLS_CERT")]`, `tls_key: Option<PathBuf>` via `--tls-key` / `TLS_KEY`
    (contracts/configuration.md option table); doc comments in the existing
    style.
  - Add `Config::tls_paths()`-style helper returning `Result<ResolvedTls,
    String>` enforcing all-or-nothing E1 with the contract message shapes.
  - GREEN: T003 passes.
- [X] T006 [US1] Create TLS setup module `src/tls.rs`
  - Load both files (existence/readability → E2 naming the path), parse PEM
    (block-type errors → E3), build `rustls::ServerConfig` via
    `builder.with_single_cert(certs, key)` and map its failure to E5
    (mismatch); no-op path: absent config returns a disabled marker so
    `main.rs` can branch.
  - Only messages from contracts/configuration.md may surface; never include
    key bytes; module doc comment states the FR mapping (FR-003/004/006/008).
  - Unit-testable pure functions; integration coverage comes via T004.
- [X] T007 [US1] Branch the serve path in `src/main.rs`
  - TLS disabled: keep `tokio TcpListener` + `axum::serve(...).with_graceful_
    shutdown(shutdown_signal())` byte-for-byte behavior (FR-002).
  - TLS enabled: validate via `src/tls.rs` before binding, then
    `axum_server::bind_rustls(addr, config).handle(handle).serve(app(state).
    into_make_service())`; drive `handle.graceful_shutdown(...)` from the
    existing `shutdown_signal()` future so SIGINT/SIGTERM semantics are
    preserved (research.md stack decision).
  - `info!` log mirrors the existing "listening on {addr}" line plus TLS
    enabled status.
  - GREEN: T004 passes; quickstart.md §2 §3 first row verified.
- [X] T008 [US1] Document TLS configuration in `docs/configuration.md`
  - New TLS section: option table (flag/env/default), enablement rule
    (all-or-nothing), link to error matrix semantics, worked example
    (quickstart §1–§2 commands), the single-listener rule, and the
    "identical API over both schemes" statement (FR-013; docs are built from
    spec/config — keep wording aligned with contracts/configuration.md).

**Checkpoint**: US1 fully functional and independently testable — spec
acceptance scenarios 1–4 verifiable via quickstart.md §2–§3. MVP complete.

---

## Phase 4: User Story 2 — End-to-end encrypted containerized deployment (Priority: P2)

**Goal**: The chart mounts an operator secret as cert/key files, passes
`--tls-cert/--tls-key`, switches probes to HTTPS, and documents edge
passthrough for encryption on every hop.

**Independent Test**: `helm template` renders the tls wiring with the values
from contracts/chart-values.md; default render unchanged; a cluster install
(quickstart.md §4) serves HTTPS from the pod. Does not require US1 code
beyond the arg names already fixed by contracts.

### Tests for User Story 2 (write first, confirm RED) ⚠️

- [X] T009 [P] [US2] Add TLS assertions to `tests/helm/template_test.sh`
  - Follow the script's existing render-and-grep pattern:
    (a) default render contains no `tls` volume, no `--tls-cert`/`--tls-key`
    args, probes have no `scheme: HTTPS` (default-render immutability,
    contracts/chart-values.md → enabled: false);
    (b) `--set tls.enabled=true --set tls.existingSecret=m2v-tls` render
    contains: secret volume with `optional: false`, volumeMount at
    `/etc/model2vec-serve/tls` readOnly, args `--tls-cert
    /etc/model2vec-serve/tls/tls.crt` and `--tls-key
    /etc/model2vec-serve/tls/tls.key`, probes `scheme: HTTPS`;
    (c) `tls.enabled=true` without `existingSecret` → `helm template`
    exits non-zero with the template failure;
    (d) `certKey`/`keyKey` overrides change the rendered arg paths.
  - RED: values do not exist yet — renders lack the tls block.

### Implementation for User Story 2

- [X] T010 [US2] Add `tls` block to `helm/model2vec-serve/values.yaml` and wire
  `templates/deployment.yaml`
  - values.yaml: the exact block from contracts/chart-values.md (enabled,
    existingSecret, certKey `tls.crt`, keyKey `tls.key`, mountPath
    `/etc/model2vec-serve/tls`) with comments matching chart style.
  - deployment.yaml: secret-sourced volume (`optional: false`) + readOnly
    volumeMount rendered only when enabled; args appended after existing
    args; probes switch to `scheme: HTTPS` when enabled (kubelet skips cert
    verification — research.md chart decision); template `fail` when enabled
    without `existingSecret`.
  - Service/ports untouched (port name stays `http` — research.md).
  - GREEN: T009 passes; `helm lint` clean (tests/helm/lint_test.sh).
- [X] T011 [US2] Document the chart TLS values in `helm/model2vec-serve/README.md`
  and `docs/deployment/helm.md`
  - Secret provisioning (`kubectl create secret tls`), the values table,
    the passthrough-vs-edge-termination comparison with the nginx
    `ssl-passthrough` + `backend-protocol: HTTPS` annotation example, and
    the restart-based rotation note (FR-009/010/013; wording follows
    contracts/chart-values.md).

**Checkpoint**: US2 works independently — quickstart.md §4 verifiable;
US1 + US2 both functional.

---

## Phase 5: User Story 3 — Fail-fast, actionable TLS configuration diagnostics (Priority: P3)

**Goal**: Every TLS misconfiguration fails startup within seconds with the
contract message naming the item and remedy; success logs cert metadata; no
key material is ever logged.

**Independent Test**: Run the validation matrix (quickstart.md §3) against
the binary; each case exits with its E-identifier message. Builds on US1's
`src/tls.rs` skeleton.

### Tests for User Story 3 (write first, confirm RED) ⚠️

- [X] T012 [P] [US3] Create the validation matrix tests in `tests/tls_validation.rs`
  - Direct unit tests of the `src/tls.rs` validation functions using
    `tests/common` fixtures — no listener: E1 single-sided, E2 missing file,
    E3 garbage PEM, E4 encrypted key, E5 mismatched pair, E6 expired cert
    (must be non-fatal: parse succeeds + expiry flag set), plus a
    happy-path parse returning metadata fields (subject/issuer/not_after)
    for the US3 logging task.
  - Assert error strings match contracts/configuration.md shapes exactly.
  - RED: E3 refinement, E4 detection, and E6 expiry do not exist yet (E1/E2/
    E5 from US1 already pass — note which cases are already GREEN and only
    the new ones must fail).

### Implementation for User Story 3

- [X] T013 [US3] Add encrypted-key detection and refined parse errors in `src/tls.rs`
  - PEM header inspection (`ENCRYPTED PRIVATE KEY`, legacy
    `Proc-Type: 4,ENCRYPTED`/`DEK-Info`) → E4 message; block-type validation
    distinguishing cert vs key parse failures for E3.
  - GREEN: T012 E3/E4 cases.
- [X] T014 [US3] Add expiry warning and certificate metadata logging in `src/tls.rs`
  - `x509-parser` extracts leaf subject, issuer, not_after; expired-but-valid
    → `warn!` naming the expiry date and continue (E6, spec assumption);
    success → `info!` "TLS enabled" line with subject/issuer/expiry
    (FR-007); grep the module for any path that could log key bytes — none
    may exist.
  - GREEN: T012 E6 + metadata cases; manual check of quickstart.md §2 log
    line.

**Checkpoint**: All user stories independently functional; validation matrix
fully contract-conformant.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Non-story deliverables and final gates

- [X] T015 [P] Cover compose/plain-binary TLS usage in docs
  - `README.md`: one-line TLS example under the run commands
    (`--tls-cert/--tls-key`). `docs/deployment/compose.md`: short section —
    mount cert/key files, set `TLS_CERT`/`TLS_KEY` env vars (default compose
    setup stays plain HTTP per FR-002). `.env.example`: commented
    `TLS_CERT`/`TLS_KEY` entries consistent with existing style.
- [X] T016 Measure HTTP vs HTTPS performance (constitution V, SC-005)
  - Extend the existing criterion bench setup (`benches/`, async_tokio
    feature) to run the same steady-state embedding load against a plain-
    HTTP and a TLS listener; report p99/throughput delta; record the exact
    invocation commands and results in
    `specs/007-tls-https-support/plan.md` (append a "Benchmark results"
    section) per the constitution's reproducible-benchmark rule.
  - Deterministic gate: setting `TLS_BENCH_MAX_DELTA_PCT` (e.g. `10`) fails
    the bench when the TLS median-latency or throughput delta exceeds the
    threshold (noise-stable signals; p99 on sub-millisecond loopback samples
    swings ±20% between shared runners and is reported for review instead).
    CI runs it as a reporting-only, non-blocking step-level job; the
    recorded deltas are reviewed against the 10% budget before release.
- [X] T017 Run the full validation suite and quickstart walk
  - `cargo fmt -- --check`; `cargo clippy --all-targets --all-features -- -D
    warnings`; `cargo test`; `bash tests/helm/lint_test.sh && bash
    tests/helm/template_test.sh`; `bash tests/compose/compose_config_test.sh`;
    then execute quickstart.md §1–§5 end to end and fix any drift between
    docs and behavior (SC-001–SC-006 evidence).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 first — everything compiles against it.
- **Foundational (Phase 2)**: T002 depends on T001 (rcgen availability).
- **User Story 1 (Phase 3)**: T003/T004 (RED) → T005 → T006 → T007 → T008.
  Depends on Phase 2 (fixtures).
- **User Story 2 (Phase 4)**: T009 → T010 → T011. **Independent of US1 code**
  (chart work touches only `helm/` + `tests/helm/`) — can run in parallel
  with US1 after Phase 2.
- **User Story 3 (Phase 5)**: T012 (RED) → T013 → T014. Depends on US1
  (`src/tls.rs` skeleton + config fields from T005/T006).
- **Polish (Phase 6)**: T015/T016 after their subject matter exists (T015
  after US1; T016 after US1); T017 last, after all stories.

### User Story Dependencies

- **US1 (P1)**: Foundational only — no cross-story dependencies.
- **US2 (P2)**: No US1 code dependency; relies on the arg names fixed by
  contracts/configuration.md (documented contract, not code).
- **US3 (P3)**: Extends US1's validation module; must follow US1.

### Within Each User Story

- Tests written and confirmed failing before implementation (RED → GREEN).
- Config/module foundations before integration (`src/config.rs` →
  `src/tls.rs` → `src/main.rs`).
- Story complete (incl. its docs task) before the next priority starts,
  unless run in parallel per the map above.

### Parallel Opportunities

- T003 + T004 (US1 tests, different files) in one batch.
- T012 (US3 tests) written while US2 stream T009–T011 runs.
- Two-developer split after Phase 2: **Stream A** = US1 (T003–T008) then US3
  (T012–T014); **Stream B** = US2 (T009–T011) then T015.
- T015 (docs) parallel with T016 (bench) — different files.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. T001–T002 (setup + fixtures)
2. T003–T004 RED → T005–T007 GREEN → T008 docs
3. **STOP and VALIDATE**: quickstart.md §2 + §3 (HTTP-unchanged row) pass —
   HTTPS serving is usable end to end; plain-HTTP users see zero change.

### Incremental Delivery

1. MVP (US1) → validate → ship-able minor release
2. + US2 → chart consumers get end-to-end encryption; validate §4
3. + US3 → production-grade diagnostics; validate §3 matrix fully
4. Polish: docs breadth (T015), measured performance (T016), full gates
   (T017)

### Parallel Team Strategy

Two developers: A takes the code stream (US1 → US3), B takes the deployment
stream (US2 → compose docs); T017 is a joint final gate.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Error messages are a contract: implement the exact shapes from
  contracts/configuration.md; tests assert them verbatim
- Key material must never appear in args values, env values, logs, or errors
  (data-model.md INV-4) — every story's tasks include this check implicitly;
  T014 greps the module explicitly
- Verify RED before GREEN for every test task; commit after each task or
  logical group
- Stop at any checkpoint to validate the story independently

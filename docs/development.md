# Development

This page covers how to build, test, lint, and benchmark the project.

## Build

```bash
cargo build --release
```

The release profile enables LTO and a single codegen unit for smaller, faster
binaries.

## Run the test suite

```bash
cargo test
```

This runs unit tests, contract tests, integration tests, and observability
tests. Some tests may download a small model from Hugging Face on first run.

## Check formatting

```bash
cargo fmt -- --check
```

To apply formatting:

```bash
cargo fmt
```

## Run clippy

The CI treats warnings as errors:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Run benchmarks

```bash
cargo bench
```

Benchmarks are located in `benches/embeddings.rs` and use Criterion with async
Tokio support.

### Transport benchmark (TLS vs plain HTTP)

Run this manually before opening a PR after transport-relevant changes
(`src/`, `benches/`, dependency bumps). The gate fails when the TLS
median-latency or throughput delta versus plain HTTP exceeds 10% (SC-005);
p99 is reported alongside for review, since it swings with runner noise on
sub-millisecond loopback samples:

```bash
TLS_BENCH_MAX_DELTA_PCT=10 cargo bench --bench embeddings -- \
  --measurement-time 3 --warm-up-time 1 --sample-size 10 "transport"
```

The benchmark is not part of CI — a steady-state TLS comparison needs a
release-profile build that would slow every PR down.

## Validate Helm

```bash
bash tests/helm/lint_test.sh
bash tests/helm/template_test.sh
```

## Validate the docs site

```bash
cd docs
npm install
npm run docs:build
npm run docs:preview
```

## Before committing

Run the full quality check locally:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
bash tests/helm/lint_test.sh
bash tests/helm/template_test.sh
```

## Code style reminders

- Do not introduce `unsafe` blocks (`unsafe_code = "forbid"`).
- Avoid `unwrap` in production code (`unwrap_used = "deny"`).
- Add doc comments for new public items (`missing_docs = "warn"`).
- Keep handlers thin and move validation into small, testable functions.

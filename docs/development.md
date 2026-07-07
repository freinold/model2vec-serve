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

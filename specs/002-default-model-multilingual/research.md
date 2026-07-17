# Research: Default Model Update and Spec Drift Check

## Decision: Default production model

**Decision**: Use `minishlab/potion-multilingual-128M` as the default model
identifier when the user does not supply one.

**Rationale**:
- The multilingual variant supports many languages out of the box, which matches
  the most common expectation for a general-purpose embedding service.
- It is part of the same model2vec family already used by the project, so no new
  inference code or dependencies are required.
- The model is hosted on Hugging Face Hub and can be loaded by the existing
  `model2vec-rs` `StaticModel::from_pretrained` path.
- The previous default, `minishlab/potion-base-2M`, is English-centric and less
  useful for international deployments.

**Alternatives considered**:
- Keep `minishlab/potion-base-2M` as the default — rejected because it is
  English-centric and does not align with the goal of broad multilingual support.
- Use a larger potion-base model (e.g., 32M or 128M) as the default — rejected
  because the multilingual model is specifically requested and is a better fit
  for general use.
- Require the user to always supply a model — rejected because a sensible default
  lowers the barrier to entry and is expected by the Helm chart and quick-start
  examples.

## Decision: Test fixture model

**Decision**: Keep automated tests and benchmarks on a small, fast model
(`minishlab/potion-base-2M`) rather than switching them to the new default.

**Rationale**:
- `potion-multilingual-128M` is significantly larger than the 2M fixture, so
  downloading and loading it in CI would increase build time and resource usage.
- Test correctness does not depend on the specific model weights; it depends on
  the loading, inference, and response-shape logic.
- Keeping fixtures small preserves the fast feedback loop required by the
  Constitution's performance focus.

**Alternatives considered**:
- Switch test fixtures to the new default — rejected because it would slow CI
  without improving test coverage.
- Use a separate small multilingual model if one exists — rejected because no
  smaller multilingual model in the potion family is known at planning time, and
  the existing 2M fixture is sufficient.

## Decision: Hugging Face Hub integration audit

**Decision**: Update the existing `specs/001-model2vec-embedding-api/research.md`
to note that direct Hugging Face Hub crate usage is limited to test fixtures and
benchmarks, and that those fixtures use the `hf-hub` v1.x API.

**Rationale**:
- The runtime model loading path goes through `model2vec-rs`, which internally
  uses its own `hf-hub` feature flag for Hub downloads.
- Tests and benchmarks also depend directly on `hf-hub` (dev-dependency only) to
  download a fixture snapshot before the server starts.
- The `hf-hub` crate reached version 1.x and its synchronous API changed from the
  older `api::sync::Api` pattern to `HFClientSync` with `snapshot_download()` and
  a builder-style `.send()` call.
- The existing research.md mentions `hf-hub` only as a `model2vec-rs` feature
  flag, which is accurate but incomplete. Adding a short note removes ambiguity
  for future maintainers.

**Alternatives considered**:
- Leave the existing research.md unchanged — rejected because the user
  explicitly asked to check for spec drift and update if needed; the gap between
  the documented feature flag and the actual direct test usage is a form of drift.
- Rewrite the research.md inference section to detail the `hf-hub` v1.x API —
  rejected because the runtime does not use that API directly; a brief note is
  simpler and sufficient.

## Decision: Scope of documentation updates

**Decision**: Update every public-facing example that presents the old default
(`minishlab/potion-base-2M`) as the default model, while preserving examples that
intentionally show how to specify a non-default model.

**Rationale**:
- Consistency across README, AGENTS.md, Helm chart README, VitePress docs, and
  the feature 001 quickstart reduces user confusion.
- Examples that demonstrate passing `--model` explicitly are still useful for
  teaching configuration and do not need to be removed.

**Files affected**:
- `src/config.rs` — add `default_value` to the `--model` argument
- `helm/model2vec-serve/values.yaml` — update default `model` value
- `helm/model2vec-serve/README.md` — update examples and default-value table
- `README.md` — update quick-start commands
- `AGENTS.md` — update local run and Docker run examples
- `docs/getting-started.md`, `docs/configuration.md`, `docs/deployment/docker.md`,
  `docs/deployment/helm.md`, `docs/api/openai.md`, `docs/api/tei.md` — update
  default examples
- `specs/001-model2vec-embedding-api/quickstart.md` — update examples
- `specs/001-model2vec-embedding-api/contracts/openai_embeddings.md` and
  `specs/001-model2vec-embedding-api/contracts/tei.md` — update example model
  identifiers where they imply the default

**Alternatives considered**:
- Update only `src/config.rs` and `values.yaml` — rejected because inconsistent
  documentation violates the feature's success criteria.
- Replace every mention of `potion-base-2M` everywhere — rejected because some
  examples intentionally show how to select a specific model; only default
  examples should change.

## Open questions resolved

None. The specification contained no `[NEEDS CLARIFICATION]` markers.

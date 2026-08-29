# Bug Assessment: TEI /info endpoint only shows one model in multi-model mode

- **Slug**: tei-info-single-model
- **Created**: 2026-08-29
- **Source**: https://github.com/freinold/model2vec-serve/issues/105
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

Fetched from GitHub (host `github.com` — `allowlisted` per URL Trust Policy; fetched without prompting, no redirects followed). Issue #105, state: open, label: `bug`, assignee: `freinold`, comments: 0, created 2026-08-27.

**Title**: "tei model info endpoint only shows one model"

**Body (verbatim)**:

> Having two models configured /info only shows one (the default i guess), but both can be used via /embed?model=potion-base-32M and /embed?model=potion-multilingual-128M

No instruction-like or suspicious content was present in the fetched page; nothing was acted upon beyond summarizing.

## Symptom

With two models configured, `GET /info` returns metadata for only one model (the default), while both models are fully usable via `/embed?model=<id>`. The reporter expected `/info` to surface all configured models. Note: `/info?model=<id>` does select a non-default model, but this is apparently unknown to the reporter — a discoverability gap rather than broken selection.

## Reproduction

1. Start the service with two models:
   `cargo run --release -- --model minishlab/potion-base-32M --model minishlab/potion-multilingual-128M --default-model minishlab/potion-base-32M --port 8080`
2. `curl http://localhost:8080/info` → returns a single `ModelInfo` object for the default model only.
3. `curl "http://localhost:8080/info?model=potion-multilingual-128M"` → returns the second model's metadata (undiscovered workaround).
4. `curl http://localhost:8080/v1/models` → lists both models (OpenAI-style discovery works).

The reporter's model IDs (`potion-base-32M`, `potion-multilingual-128M`) lack an org prefix, suggesting locally mounted models; this does not change the analysis.

[NEEDS CLARIFICATION: exact invocation (Docker/Helm vs. cargo) — not needed for diagnosis; behavior is confirmed from code.]

## Suspected Code Paths

- `src/routes/tei.rs:82-93` (`tei_info`) — resolves exactly one model via `registry.resolve(query.model.as_deref())` and serializes a single `ModelInfo`; no enumeration of the registry.
- `src/model/mod.rs:173-178` (`ModelRegistry::resolve`) — falls back to `default_model_id` when no `?model=` is passed, which is why plain `/info` always reports the default.
- `src/model/mod.rs:180-183` (`ModelRegistry::iter`) — an iterator over all loaded models already exists but is unused by `tei_info`.
- `src/routes/dto.rs:121-132` (`ModelInfo`) — single-model DTO shape with no list/availability field.
- `specs/003-multi-model-serving/contracts/tei.md:38-56` — the documented contract explicitly defines `GET /info` as returning the default model's metadata, or the `?model=`-selected model's. The implementation matches this contract; the contract itself encodes the design gap.
- `tests/tei_contract.rs:51,136,160` and `tests/tei_integration.rs:77` — existing tests lock in the current single-object behavior, including the `?model=` selector.

## Root Cause Hypothesis

**Confidence: high.** This is not an implementation defect against the spec — `tei_info` faithfully implements the documented single-object TEI contract. The root cause is a design gap in the multi-model feature: TEI's real `/info` is inherently single-model (TEI serves one model per process), and when multi-model serving was added, `GET /info` was kept single-model with a `?model=` query selector for selection, while model *discovery* was left to the OpenAI-style `GET /v1/models`. A TEI-ecosystem user has no discoverable way to learn that `?model=` exists or what model IDs are valid, so plain `/info` looks like it "only shows one model."

## Proposed Remediation

**Preferred**: Keep the TEI-compatible single-object response shape, but make the multi-model surface discoverable from `/info` itself. Extend `ModelInfo` (`src/routes/dto.rs:121-132`) with an additive, optional field `available_models: Vec<String>` listing all loaded model IDs (populated via the existing `ModelRegistry::iter()`), and keep `model_id` as the default (or `?model=`-selected) model. Additive JSON fields are ignored by most lenient clients (serde/pydantic defaults), so TEI client compatibility is preserved while the response now answers "what else can I use?". Wire it in `tei_info` (`src/routes/tei.rs:82-93`), update the utoipa schema, and update the contract (`specs/003-multi-model-serving/contracts/tei.md`), README, and VitePress docs per the repo's docs-sync rules. As a secondary aid, consider including valid model IDs in the `ModelNotFound` error message — but only after checking `specs/003-multi-model-serving/contracts/errors.md` for message-shape rules.

**Alternatives** (optional):
- Add an opt-in full listing, e.g. `GET /info?all=true` or a dedicated `GET /info/all` returning `Vec<ModelInfo>` — richer per-model metadata (max_input_length, dimension, pooling per model) without touching the default response shape, at the cost of extra non-TEI surface to document.
- Docs-only fix: advertise `/info?model=` and `/v1/models` in README/docs — cheapest, but leaves the discovery gap the reporter actually hit.

**Files likely to change**:
- `src/routes/dto.rs` — extend `ModelInfo`
- `src/routes/tei.rs` — populate `available_models` in `tei_info`
- `tests/tei_contract.rs`, `tests/tei_integration.rs`
- `specs/003-multi-model-serving/contracts/tei.md`
- `docs/` (TEI API page) and `README.md` if endpoint examples are present

**Tests to add or update**:
- `tests/tei_contract.rs`: assert `/info` response includes `available_models` containing all loaded model IDs in a multi-model setup, and that the top-level fields still match the TEI shape.
- `tests/tei_integration.rs`: assert `available_models` lists both models and `model_id` remains the default when `?model=` is omitted.
- Confirm single-model deployments keep a clean TEI-shape response (decide and pin whether `available_models` is always present or only when >1 model is loaded).

## Risks & Considerations

- **API shape change (additive)**: strict clients deserializing `/info` with `deny_unknown_fields` could break; most serde/pydantic-based TEI clients ignore extra fields. Verify against TEI client expectations before shipping.
- **Contract/docs drift**: the multi-model TEI contract, OpenAPI schema (utoipa), README, and docs site must all be updated together per repo convention.
- **Semantics decision**: whether `available_models` appears in single-model mode and whether it reflects only successfully loaded models (failed models are excluded from the registry) — needs a documented rule.
- **Low urgency**: workarounds exist (`/info?model=`, `/v1/models`); no incorrect embeddings, data risk, or availability impact.

## Open Questions

- Should `available_models` be always present, or only emitted when more than one model is loaded? [NEEDS CLARIFICATION]
- Does the reporter need per-model metadata for all models (favors `?all=true` alternative) or is a simple ID list sufficient (favors preferred fix)? [NEEDS CLARIFICATION]
- Are any downstream TEI clients consuming this service's `/info` with strict deserialization? [NEEDS CLARIFICATION]

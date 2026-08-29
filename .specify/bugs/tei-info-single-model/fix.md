# Bug Fix: TEI /info endpoint only shows one model in multi-model mode

- **Slug**: tei-info-single-model
- **Fixed**: 2026-08-29
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

`GET /info` now includes an `available_models` array listing all loaded model IDs (sorted ascending) alongside the existing TEI-compatible fields, so multi-model deployments are discoverable from a plain `GET /info`. `model_id` continues to report the default (or `?model=`-selected) model, preserving the TEI single-object response shape.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/routes/dto.rs` | modified | Added `available_models: Vec<String>` to `ModelInfo` with doc comment |
| `src/routes/tei.rs` | modified | `tei_info` populates `available_models` from `registry.iter()`, sorted with `sort_unstable()` for deterministic output |
| `specs/003-multi-model-serving/contracts/tei.md` | modified | `/info` response example and contract text updated |
| `docs/api/tei.md` | modified | Response example and field-description table updated |
| `tests/tei_contract.rs` | modified | `/info` assertions extended: field present, non-empty, contains `model_id` |
| `tests/tei_integration.rs` | modified | New multi-model test `tei_info_lists_all_available_models` |

## Diff Highlights

`src/routes/tei.rs`:

```rust
let mut available_models: Vec<String> = state
    .registry
    .iter()
    .map(|model| model.model_id.clone())
    .collect();
available_models.sort_unstable();
```

`src/routes/dto.rs`:

```rust
/// Identifiers of all loaded models, sorted ascending.
pub available_models: Vec<String>,
```

## Tests Added or Updated

- `tests/tei_contract.rs::tei_info_returns_model_metadata` — pins that `/info` includes `available_models` (array, non-empty, contains `model_id`) in single-model mode.
- `tests/tei_contract.rs::tei_info_without_model_uses_default` — pins that the default model's ID appears in `available_models`.
- `tests/tei_integration.rs::tei_info_lists_all_available_models` — with two models loaded: exactly 2 entries, contains the default `model_id`, contains `alt-model`, and the list is sorted ascending.

## Local Verification

- Commands run:
  - `cargo fmt -- --check` → OK
  - `cargo clippy --all-targets --all-features -- -D warnings` → clean (after fixing one `clippy::len_zero` in a new test assertion)
  - `cargo test --test tei_contract --test tei_integration` → 11 passed, 0 failed
  - `cargo test` (full suite) → 40 passed, 0 failed, 1 ignored
- Manual checks: test fixture model (`minishlab/potion-base-2M`) was already in the HF cache, so the suite ran without network downloads.

## Deviations from Assessment

- **Semantic decision pinned**: the assessment left open whether `available_models` should be always present or only when >1 model is loaded. Decision: **always present** (single-element array in single-model mode) — simplest semantics to document, test, and consume.
- **Scope stayed as listed**; the assessment's optional secondary aid (including valid model IDs in the `ModelNotFound` error message) was intentionally **not** implemented, as the assessment gated it on an errors-contract check.
- Sorting (`sort_unstable()`) was added beyond the letter of the assessment because `ModelRegistry` stores models in a `HashMap`; unsorted output would be nondeterministic across requests.

## Follow-ups

- Consider enriching the `model_not_found` error message with the valid model IDs (per assessment alternative; check `specs/003-multi-model-serving/contracts/errors.md` first).
- Verify real-world TEI clients tolerate the additive field (most serde/pydantic clients ignore unknown fields; risk assessed as low in the assessment).
- If richer per-model metadata is ever needed (per-model `max_input_length`/dimension), revisit the `?all=true` alternative from the assessment.

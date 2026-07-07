<!--
Sync Impact Report
- Version change: initial adoption from template (no prior version) → 1.0.0
- Modified principles: none (all principles are new)
- Added sections:
  - I. Code Quality
  - II. Test Coverage
  - III. API Conformity
  - IV. Simplicity Over Complexity
  - V. Performance Focus
  - Quality & Performance Standards
  - Development Workflow
  - Governance
- Removed sections: none
- Templates requiring updates:
  - .specify/templates/tasks-template.md → updated to make tests mandatory
  - .specify/templates/plan-template.md → reviewed, no changes required
  - .specify/templates/spec-template.md → reviewed, no changes required
  - .specify/templates/checklist-template.md → reviewed, no changes required
  - .specify/templates/constitution-template.md → reviewed, no changes required
- Follow-up TODOs:
  - TODO(RATIFICATION_DATE): original adoption date is not recorded; set to the
    formal ratification date when known.
-->

# model2vec-serve Constitution

## Core Principles

### I. Code Quality

All code MUST be correct, idiomatic, and maintainable.

- Every change MUST pass the project's lint, format, and type-check gates before merge.
- Errors and edge cases MUST be handled explicitly; panics, bare `except`, and
  `unwrap`/`expect` are only permitted for truly impossible states that are
  documented inline.
- Dead code, unused dependencies, and commented-out logic MUST be removed before review.
- Public APIs and modules MUST have clear, consistent names and a single, obvious responsibility.
- Every non-trivial function or module MUST be reviewed by at least one other contributor.

Rationale: High code quality keeps the embedding server reliable and reduces the
long-term cost of changes as the API surface grows.

### II. Test Coverage

All behavior MUST be verifiable by automated tests.

- Every feature, bug fix, and API change MUST include tests that fail before the
  change and pass after it.
- Critical paths (request handling, model loading, tokenization, serialization)
  MUST have tests that exercise success, error, and boundary cases.
- The project MUST maintain a minimum line coverage of 80% for Python and Rust
  components, with 100% coverage for API contract paths.
- All tests MUST pass in CI before merge; flaky tests MUST be fixed or removed.
- Contract tests MUST verify that `/v1/embeddings` and `/v1/models` responses
  conform to the OpenAI API shape.

Rationale: Test coverage protects the OpenAI-compatible contract and makes
refactors safe.

### III. API Conformity

The server MUST expose an OpenAI-compatible embeddings API and document every deviation.

- Request and response shapes for `/v1/embeddings` and `/v1/models` MUST match
  the documented OpenAI contract unless an explicit, user-visible exception is approved.
- API changes that break existing clients MUST trigger a MAJOR version bump of
  the server release.
- Input validation MUST return standard HTTP status codes and JSON error bodies.
- OpenAPI documentation (`/docs`) MUST stay in sync with the implementation.
- Backward-incompatible contract changes MUST include a migration note.

Rationale: API conformity lets users swap model2vec-serve for OpenAI-compatible
clients without surprises.

### IV. Simplicity Over Complexity

Choose the simplest solution that meets the requirement.

- New dependencies MUST be justified: prefer the standard library or existing
  stack before adding a new package or crate.
- Abstractions, interfaces, and configuration options MUST solve a concrete,
  current problem; speculative generality is prohibited.
- Every feature plan MUST include a complexity-tracking section that records
  rejected simpler alternatives.
- Refactoring that removes code without losing functionality is preferred over
  adding code.
- Code MUST be readable enough that a new contributor can understand the intent
  without extensive domain knowledge.

Rationale: A minimal OpenAI-compatible server stays fast to build, easy to
operate, and cheap to maintain.

### V. Performance Focus

Performance is a first-class requirement and MUST be measured.

- Every feature MUST define concrete performance goals (e.g., p99 latency,
  throughput, peak memory) in its implementation plan.
- Critical request paths MUST be benchmarked before release; regressions beyond
  10% MUST be justified or fixed.
- Model loading and inference MUST avoid unnecessary copies, locks, or blocking
  operations on the hot path.
- Profiling MUST be used to guide optimization; optimizations without
  measurements require explicit approval.
- Resource usage MUST be monitored in CI or release notes; memory leaks and
  unbounded queues are not acceptable.

Rationale: Embedding servers are judged by throughput and latency; measured
performance focus keeps the project competitive.

## Quality & Performance Standards

- Language and tooling: Python code uses `uv`, `ruff`, and `mypy`; Rust code
  uses `cargo`, `clippy`, and `rustfmt`.
- Static checks and the full test suite MUST pass on every pull request.
- API contract tests MUST run against both local and containerized builds.
- Performance benchmarks MUST be reproducible and committed with their invocation commands.
- Security: untrusted input MUST be validated; secrets and model paths MUST be
  configurable, never hard-coded.

## Development Workflow

- Work is organized per feature branch with a `spec.md`, `plan.md`, `tasks.md`,
  and `checklist.md` derived from the project templates.
- A feature is not complete until its tests, static checks, and documentation
  are updated.
- Code review MUST verify compliance with each Core Principle; reviewers MAY
  request simplification or benchmark evidence.
- The `/speckit.*` commands and project templates MUST be used to keep planning,
  tasks, and checks consistent.
- Releases MUST update version strings, changelog, and performance notes together.

## Governance

This constitution is the highest-level guidance for model2vec-serve. When a
practice conflicts with these principles, the constitution wins.

- Amendments MUST be proposed via a dedicated PR that updates this file,
  increments `CONSTITUTION_VERSION`, and explains the impact.
- `CONSTITUTION_VERSION` follows SemVer: MAJOR for incompatible principle
  redefinitions or removals, MINOR for new principles or material guidance,
  PATCH for wording clarifications.
- Compliance review MUST occur before each minor or major release to confirm
  the project still follows every principle.
- Any deferred placeholder MUST be tracked in the Sync Impact Report until resolved.

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): original adoption date not recorded; set when formally ratified | **Last Amended**: 2026-07-07

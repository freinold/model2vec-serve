# Contract: Documentation (README + Docs Site)

This contract defines the documentation deliverables for the compose feature
(spec FR-010, FR-011, FR-012; SC-003, SC-004). It follows the documentation
update rules from `AGENTS.md` (README, VitePress docs, and chart/deployment
docs move together).

## README.md

1. **MUST** gain a `## Docker Compose` section positioned between the existing
   `## Container` and `## Helm` sections (escalating deployment complexity:
   plain Docker → local stack → Kubernetes).
2. The section **MUST** contain:
   - A one-sentence statement of what the compose path provides (two models,
     persisted model cache, one command).
   - The launch command `docker compose up -d` (from the repo root).
   - The default served model ids and which is the default model.
   - The model-cache location note (`./models`, survives restarts).
   - A pointer to the full docs page (relative link to
     `docs/deployment/compose.md`, consistent with how the README links
     `docs/deployment/docker.md`).
3. The `## Features` list **MUST** gain one bullet mentioning one-command
   two-model local deployment via Docker Compose.
4. **MUST NOT** duplicate the full guide — the README is a teaser + link
   (SC-004's "one click" requirement).

## Docs site

1. **MUST** add `docs/deployment/compose.md` with, in order:
   1. Intro paragraph: what the stack starts, which image it uses.
   2. **Prerequisites**: Docker with Compose v2; network access for the first
      model download; ~2 GB disk for the two default models.
   3. **Quick start**: `docker compose up -d`, wait for healthy
      (`docker compose ps`), example request against both models (OpenAI and
      TEI examples, mirroring the README style).
   4. **Served models**: the two defaults, default-model semantics, how to
      change them via `MODEL`/`DEFAULT_MODEL`.
   5. **Model cache / volume mounting**: host path `./models`, effective
      location `models/.cache/huggingface/hub`, `HOME=/models` rationale
      (link to the Helm persistence docs for the shared pattern), changing
      the path or using a named volume, cache warm restart + offline restart
      behavior.
   6. **Configuration**: complete variable table (identical to
      `data-model.md` → *Customization Surface*), `.env.example` usage,
      empty-vs-unset note for `API_KEY`.
   7. **Operations**: logs (`docker compose logs -f`), stop/teardown, updating
      the image (`docker compose pull && docker compose up -d`), health-check
      behavior and the "image must be built after this feature" caveat.
   8. **Relation to other deployment paths**: when to choose compose vs plain
      `docker run` vs Helm (link to both pages).
   9. **Troubleshooting**: port in use, unwritable cache directory, model
      download failure, switching from an old image without health check.
2. `docs/.vitepress/config.ts` sidebar **MUST** gain
   `{ text: 'Docker Compose', link: '/deployment/compose' }` between the
   Docker and Helm entries.
3. `docs/deployment/docker.md` **MUST** gain a short cross-link to the compose
   page (and the compose page links back).

## Documentation quality rules

1. **Every command documented MUST be executable as written** on a machine
   meeting the stated prerequisites (FR-012, SC-003). Commands are validated
   by walking through `quickstart.md`, which uses the same commands.
2. **Single source of truth for the variable table**: the canonical table
   lives in this feature's `data-model.md`; `docs/deployment/compose.md`
   reproduces it and `AGENTS.md`-driven doc-sync rules apply to later changes.
3. Model ids in docs **MUST** match the compose file exactly
   (`minishlab/potion-multilingual-128M`,
   `minishlab/potion-code-16M-v2`).
4. Docs **MUST** state that the compose path targets local evaluation,
   development, and demos — Kubernetes/Helm remains the production path
   (spec Assumptions).

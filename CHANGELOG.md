# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/freinold/model2vec-serve/compare/v0.5.4...v0.6.0) - 2026-09-10

### Fixed

- collapse nested if-let to satisfy the new clippy collapsible_if lint
- adapt to axum-server 0.8 and reqwest 0.13
- *(deps)* update rust dependencies
- *(ci)* use the full image reference when reading the manifest digest ([#145](https://github.com/freinold/model2vec-serve/pull/145))

### Other

- *(deps)* track the stable toolchain as MSRV (1.98)
- *(deps)* raise MSRV to 1.88 and update time to 0.3.55
- Merge pull request #136 from freinold/release-plz-2026-09-10T18-52-01Z

## [0.5.4](https://github.com/freinold/model2vec-serve/compare/v0.5.3...v0.5.4) - 2026-09-10

### Other

- *(deps)* update rust:1.98-slim docker digest to bce1476

## [0.5.3](https://github.com/freinold/model2vec-serve/compare/v0.5.2...v0.5.3) - 2026-09-10

### Added

- add optional TLS/HTTPS serving with end-to-end encryption

### Fixed

- try all resolved addresses when binding the TLS listener

### Other

- *(deps)* update rust crate time to v0.3.47 [security] ([#128](https://github.com/freinold/model2vec-serve/pull/128))
- pin bench certificate SANs and add the SC-005 delta gate
- pin self-signed test roots instead of disabling TLS verification
- document TLS options, cert-manager flows, and compose TLS usage

## [0.5.2](https://github.com/freinold/model2vec-serve/compare/v0.5.1...v0.5.2) - 2026-09-01

### Added

- *(ci)* automate helm chart bump and release on app releases

### Other

- *(ci)* address CodeRabbit review comments on chart automation

## [0.5.1](https://github.com/freinold/model2vec-serve/compare/v0.5.0...v0.5.1) - 2026-09-01

### Added

- *(compose)* add two-model docker compose deployment with model cache volume
- *(docker)* add curl and HEALTHCHECK to the runtime image

### Fixed

- *(ci)* enable git_only for release-plz so app releases are processed
- *(compose)* address review feedback
- *(ci)* scope permissions to content readonly

### Other

- *(docker)* bump runtime base to debian:trixie-slim and align image docs
- add docker compose guide and readme section
- *(spec)* add feature 006 docker compose support specs

## [0.5.0](https://github.com/freinold/model2vec-serve/compare/v0.3.0...v0.5.0) - 2026-08-31

### Added

- *(helm)* publish chart, add persistence and ingress templates ([#99](https://github.com/freinold/model2vec-serve/pull/99))
- *(tei)* add model path aliases and registry path identifiers
- *(tei)* add per-model embed/info endpoints and retire model query param
- *(helm)* add modelAliases value for TEI per-model paths

### Fixed

- *(tei)* harden alias validation and per-model info attribution

### Other

- *(deps)* update rust crate clap to v4.6.5 ([#86](https://github.com/freinold/model2vec-serve/pull/86))
- *(deps)* lock file maintenance ([#87](https://github.com/freinold/model2vec-serve/pull/87))
- *(deps)* update debian:bookworm-slim docker digest to abd67ff ([#89](https://github.com/freinold/model2vec-serve/pull/89))
- *(deps)* update rust:1.97-slim docker digest to 3b28790 ([#90](https://github.com/freinold/model2vec-serve/pull/90))
- *(deps)* update rust dependencies ([#91](https://github.com/freinold/model2vec-serve/pull/91))
- *(deps)* update rust:1.97-slim docker digest to 8e8cf8f ([#92](https://github.com/freinold/model2vec-serve/pull/92))
- *(deps)* update rust crate uuid to v1.25.0 ([#100](https://github.com/freinold/model2vec-serve/pull/100))
- *(deps)* update rust docker tag to v1.98 ([#101](https://github.com/freinold/model2vec-serve/pull/101))
- upgrade speckit and add extions for bugfix and idea assessment ([#106](https://github.com/freinold/model2vec-serve/pull/106))
- upgrade opencode workflow ([#107](https://github.com/freinold/model2vec-serve/pull/107))
- *(deps)* update debian:bookworm-slim docker digest to 8820086 ([#108](https://github.com/freinold/model2vec-serve/pull/108))
- *(deps)* update rust:1.98-slim docker digest to 17d1ba8 ([#109](https://github.com/freinold/model2vec-serve/pull/109))
- *(deps)* update rust crate uuid to v1.26.0 ([#110](https://github.com/freinold/model2vec-serve/pull/110))
- *(deps)* update actions/checkout action to v7 ([#111](https://github.com/freinold/model2vec-serve/pull/111))
- *(release)* bump version to 0.5.0
- add per-model embed route benchmark
- document TEI per-model endpoints and 0.5.0 migration
- sync speckit state
- add HTTP-level per-model route benchmarks
- fix quickstart conflict example and refresh image tags
- *(helm)* release chart 0.3.0 with appVersion 0.5.0
- *(helm)* update template assertions for chart 0.3.0 and app 0.5.0
- untrack local speckit feature state
- automate app releases via release-plz PAT and v-tag image builds ([#115](https://github.com/freinold/model2vec-serve/pull/115))

## [0.3.0](https://github.com/freinold/model2vec-serve/compare/v0.2.0...v0.3.0) - 2026-08-01

### Added

- serve multiple model2vec models in parallel

### Fixed

- *(deps)* update rust dependencies ([#83](https://github.com/freinold/model2vec-serve/pull/83))

### Other

- *(deps)* lock file maintenance ([#80](https://github.com/freinold/model2vec-serve/pull/80))
- *(deps)* update rust:1.97-slim docker digest to 5c6f46a ([#82](https://github.com/freinold/model2vec-serve/pull/82))
- *(release)* disable semver-checks for unpublished crate
- *(release)* bump version to 0.3.0

## [0.2.0](https://github.com/freinold/model2vec-serve/compare/v0.1.0...v0.2.0) - 2026-07-17

### Added

- default to minishlab/potion-multilingual-128M

### Fixed

- *(deps)* update rust dependencies ([#61](https://github.com/freinold/model2vec-serve/pull/61))

### Other

- *(deps)* pin dependencies ([#59](https://github.com/freinold/model2vec-serve/pull/59))
- *(deps)* update actions/checkout action to v7 ([#62](https://github.com/freinold/model2vec-serve/pull/62))
- *(deps)* update actions/deploy-pages action to v5 ([#63](https://github.com/freinold/model2vec-serve/pull/63))
- *(deps)* update actions/setup-node action to v7 ([#64](https://github.com/freinold/model2vec-serve/pull/64))
- *(deps)* update azure/setup-helm action to v5 ([#66](https://github.com/freinold/model2vec-serve/pull/66))
- *(deps)* update docker/build-push-action action to v7 ([#68](https://github.com/freinold/model2vec-serve/pull/68))
- *(deps)* update docker/login-action action to v4 ([#69](https://github.com/freinold/model2vec-serve/pull/69))
- *(deps)* update docker/metadata-action action to v6 ([#70](https://github.com/freinold/model2vec-serve/pull/70))
- *(deps)* update docker/setup-buildx-action action to v4 ([#71](https://github.com/freinold/model2vec-serve/pull/71))
- *(deps)* update dependency node to v24 ([#67](https://github.com/freinold/model2vec-serve/pull/67))
- *(deps)* update actions/upload-pages-artifact action to v5 ([#65](https://github.com/freinold/model2vec-serve/pull/65))
- *(deps)* update rust docker tag to v1.97 ([#60](https://github.com/freinold/model2vec-serve/pull/60))
- *(deps)* lock file maintenance ([#73](https://github.com/freinold/model2vec-serve/pull/73))
- *(deps)* update rust crate hf-hub to v1
- *(deps)* update rust:1.97-slim docker digest to 34fb2f1 ([#74](https://github.com/freinold/model2vec-serve/pull/74))
- *(specs)* add feature 002 specification and planning artifacts
- update default model references across documentation
- *(research)* document hf-hub v1.x test fixture usage
- *(release)* bump version to 0.2.0

## [0.1.0](https://github.com/freinold/model2vec-serve/releases/tag/v0.1.0) - 2026-07-07

### Added

- initial version
- implement model2vec-embedding-api with axum, utoipa, tests, docker and helm

### Fixed

- include benches in Docker build context

### Other

- move python files to subdir
- bootstrap speckit governance, opencode commands, and project templates
- *(spec)* add model2vec embedding API specification v1.0
- *(plan)* add implementation plan, research, data model, contracts and quickstart
- *(tasks)* add implementation task breakdown for model2vec-embedding-api
- update ignore files and add agent guide
- add Helm lint and template validation to CI
- add Renovate configuration for automated dependency updates
- add VitePress docs deployment to GitHub Pages
- add Helm lint and template validation scripts
- add VitePress documentation site
- mark implementation tasks as complete
- mark quickstart validation task complete
- add GHCR image build workflow and release-plz automation ([#55](https://github.com/freinold/model2vec-serve/pull/55))
- apply original model2vec colors and logo ([#56](https://github.com/freinold/model2vec-serve/pull/56))
- fix release-plz PR job by removing self-ignoring .gitignore rule
- tag container images with only version and latest

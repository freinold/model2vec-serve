# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.5](https://github.com/freinold/model2vec-serve/compare/v0.5.4...v0.5.5) - 2026-09-10

### Fixed

- collapse nested if-let to satisfy the new clippy collapsible_if lint
- adapt to axum-server 0.8 and reqwest 0.13
- *(deps)* update rust dependencies

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

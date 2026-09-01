# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

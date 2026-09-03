# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0](https://github.com/Dalvany/rphonetic/compare/v3.1.0...v4.0.0) - 2026-09-03

### Added

- [**breaking**] remove lazy_static ([#84](https://github.com/Dalvany/rphonetic/pull/84))

### Fixed

- *(cologne)* do not let H separate two identical codes ([#85](https://github.com/Dalvany/rphonetic/pull/85))
- fix release workflow
- *(ci)* fix ci
- *(ci)* fix release workflow

## [3.1.0](https://github.com/Dalvany/rphonetic/compare/v3.0.6...v3.1.0) - 2026-08-21

### Added

- use thiserror ([#73](https://github.com/Dalvany/rphonetic/pull/73))
- configure clippy
- try use trusted publisher

### Fixed

- *(ci)* tries to fix release jobs
- *(ci)* improve job
- *(ci)* add permission for trusted publisher support
- *(metaphone)* region_match matching unanchored instead of at the index ([#79](https://github.com/Dalvany/rphonetic/pull/79)).
  Thanks to [KBS](https://github.com/youdie006)
- remove some unwrap ([#74](https://github.com/Dalvany/rphonetic/pull/74))
- auto fix warnings

### Other

- add one contributing requirement
- improve contributing guidlines
- Bump actions/checkout from 6 to 7 ([#78](https://github.com/Dalvany/rphonetic/pull/78))
- Bump codecov/codecov-action from 6 to 7 ([#76](https://github.com/Dalvany/rphonetic/pull/76))
- Bump codecov/codecov-action from 5 to 6 ([#70](https://github.com/Dalvany/rphonetic/pull/70))

## [3.0.6](https://github.com/Dalvany/rphonetic/compare/v3.0.5...v3.0.6) - 2026-01-18

### Fixed

- fix documentation generation

## [3.0.5](https://github.com/Dalvany/rphonetic/compare/v3.0.4...v3.0.5) - 2026-01-18

### Fixed

- underflow panic in double_metaphone::condition_l0 function ([#67](https://github.com/Dalvany/rphonetic/pull/67)). Thanks to [Oliver Coleman](https://github.com/OliverColeman)

### Other

- apply fmt
- bump dependencies

## [3.0.4](https://github.com/Dalvany/rphonetic/compare/v3.0.3...v3.0.4) - 2025-08-02

### Other

- bump dependencies ([#62](https://github.com/Dalvany/rphonetic/pull/62))

## [3.0.3](https://github.com/Dalvany/rphonetic/compare/v3.0.2...v3.0.3) - 2025-04-09

### Other

- copyedit English in README ([#60](https://github.com/Dalvany/rphonetic/pull/60))

## [3.0.2](https://github.com/Dalvany/rphonetic/compare/v3.0.1...v3.0.2) - 2025-02-18

### Fixed

- fix compilation
- compilation errors

### Other

- bump dependencies

## [3.0.1](https://github.com/Dalvany/rphonetic/compare/v3.0.0...v3.0.1) - 2024-12-04

### Other

- bump and fixes ([#56](https://github.com/Dalvany/rphonetic/pull/56))

## [3.0.0](https://github.com/Dalvany/rphonetic/compare/v2.2.1...v3.0.0) - 2024-10-15

### Added

- feat([#49](https://github.com/Dalvany/rphonetic/pull/49))!: make max_code_length optional in metaphone and double metaphone ([#50](https://github.com/Dalvany/rphonetic/pull/50))

max_code_length is now optional for metaphone and double metaphone algorithms.

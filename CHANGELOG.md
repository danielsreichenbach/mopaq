# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial project structure with three crates: storm, storm-ffi, storm-cli
- Basic module structure for MPQ functionality
- Error types and result aliases
- CI/CD pipeline with GitHub Actions
- Documentation structure
- Development tooling (Makefile, scripts)

### Changed

- CLI binary renamed from `storm` to `storm-cli` to avoid naming conflicts with the library crate

### Technical Details

- Using Rust edition 2021 with MSRV 1.86
- Dual-licensed under MIT and Apache 2.0
- StormLib-compatible FFI interface planned

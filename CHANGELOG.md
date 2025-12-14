# Changelog

All notable changes to Fusabi will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.35.0] - 2025-12-14

### Added
- Multi-file module system with `#load` directive (RFC-003, #273)
  - `#load "path/to/file.fsx"` to include other Fusabi files
  - Circular dependency detection via loading set
  - File caching for efficient re-evaluation
  - Path resolution (relative, absolute)
  - `FileLoader` API for programmatic loading
- Async Tokio integration for real non-blocking I/O (RFC-004, #274)
  - `AsyncRuntime` bridging VM to Tokio
  - `TaskId`, `AsyncState`, `AsyncValue` types for task management
  - `Async.sleep` - non-blocking sleep via Tokio
  - `Async.parallel` - run async tasks concurrently
  - `Async.withTimeout` - timeout wrapper for async operations
  - `Async.catch` - error handling for async tasks
  - `Async.cancel` - cancel running tasks
  - Feature-gated behind `async` feature flag

### Changed
- `Program` AST now includes `directives` field for `#load` statements
- `Value` enum now includes `Async` variant (when `async` feature enabled)
- `Vm` now supports async methods (`enable_async`, `exec_async`, `await_async`, etc.)

## [0.34.0] - 2025-12-13

### Added
- Type providers integration with compiler and LSP (#250)
- `fusabi-type-providers` crate for typed data access

### Changed
- Documentation cleanup and versioning (#241)
- Ecosystem & Tools documentation section (#236)

### Fixed
- Debug statements removed and VM return handling fixed (#230)

## [0.33.0] - 2025-XX-XX

### Added
- Nav module for Scarab navigation/keymap integration (#229)

## Previous Versions

See git history for changes before 0.33.0.

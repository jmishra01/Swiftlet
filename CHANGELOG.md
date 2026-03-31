# Changelog

This changelog is derived from git tags and the commits since the latest tag.

### v0.1.5 - 2026-03-31

Changes in `v0.1.4..v0.1.5`:

### Added

- Terminal priority syntax for grammar terminals, enabling definitions like `TOKEN.10: ...`.
- Public lexer debugging APIs in Rust:
  `Swiftlet::tokens(text)` for structured token inspection.
  `Swiftlet::print_tokens(text)` for human-readable token traces.
- Matching lexer debugging APIs in Python:
  `Swiftlet.tokens(text)` and `Swiftlet.print_tokens(text)`.
- Earley `LexerMode::Dynamic` for parser-guided terminal matching at input offsets.
- Initial Earley `LexerMode::Scannerless` mode.

### Changed

- Added grammar and binding tests covering terminal priority behavior and token-stream debugging.
- Added HTTP and column expression examples.
- Added more lexer tests and token-awareness related parser improvements.
- Added SQL example improvements and import error messaging updates.
- Added Python package metadata homepage URL.
- Exposed `lexer_mode` in the Python bindings with `basic`, `dynamic`, and `scannerless` options.
- Added Rust and Python tests covering contextual terminal parsing with Earley dynamic and scannerless modes.

## Releases

### v0.1.4 - 2026-03-29

Changes in `v0.1.3..v0.1.4`:

#### Added

- Python example support.
- Additional parser and grammar tests.
- Improved grammar loading and parser build error surfacing.

#### Changed

- Removed duplicate terminal definitions.
- Removed default debug behavior.
- Updated versioning and README content.
- Upgraded the Python library packaging/version metadata.

### v0.1.3 - 2026-03-26

Changes in `v0.1.2..v0.1.3`:

#### Added

- Python `gr_*.py` example files.
- `preclude` usage support in Rust examples.
- Improved token priority behavior between overlapping tokens.

#### Changed

- Removed unused modules.
- Upgraded Rust and Python library versions.

### v0.1.2 - 2026-03-25

Changes in `v0.1.1..v0.1.2`:

#### Changed

- Revised installation instructions in the README.
- Simplified the README examples section.
- Bumped package versions for the `0.1.2` release.

### v0.1.1 - 2026-03-25

Changes in `v0.1.0..v0.1.1`:

#### Changed

- Updated CI to use newer macOS runners and removed macOS 13.
- Updated Cargo and Python project configuration.

### v0.1.0 - 2026-03-25

Initial tagged release.

#### Changed

- Refactored the Python package publish workflow.

# Changelog

### v0.2.0 - 2026-04-04

Changes in `v0.1.5..v0.2.0`

### Added
- **Refactored Rust parser construction to use `Swiftlet::from_str(...)` / `Swiftlet::from_file(...)` plus `grammar.parser(...)` instead of direct string/file parser constructors.**
- **File-based Rust grammar loading now returns `SwiftletError::GrammarFileReadError` when the grammar file cannot be read, instead of panicking.**
- Kept the Python-facing constructor API unchanged while updating the bindings to use the new Rust grammar source API internally.
- **Context-aware tokenization support, including parser-guided terminal selection for ambiguous token sets.**
- Revert `LexerMode`, `Swiftlet::tokens(text)`, and `Swiftlet::print_tokens(text)` changes.
- Renamed the public error type from ParserError to SwiftletError. 
- Split error handling into nested domain-specific enums: GrammarError, LexerError, and ParseError. 
- Updated Rust parser, grammar-loading, and transformation paths to return structured SwiftletError variants. 
- Preserved Python-facing error behavior while routing binding errors through the new Rust error hierarchy. 
- Added and updated Rust and Python tests to cover the refactored error structure, including grammar file read failures.


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

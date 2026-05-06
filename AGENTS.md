# AGENTS.md

## Commands
- Build: `cargo build`, `cargo build --release`
- Test: `cargo test`, `cargo test <test_name>` (specific test)
- Lint: `cargo clippy` (Rust linter)
- Typecheck: `cargo check`
- CLI: `cargo run -- <args>` (e.g., `--check <file.org>`, `--fix <file.org>`)

## Architecture
- Entry point: `src/main.rs`
- `Document::from_content()` for tests (no file path needed)
- Rules: `Rule` trait (check), `Fixable` trait (fix). Fixable rules use `impl_fixable_rule!` macro.
- Rules in `src/rules/` include their own tests.

## Conventions
- SOLID principles required
- No single-letter variables; all names meaningful
- No comments unless asked
- Rule tests use `Document::from_content()`; I/O tests use `tempfile::TempDir`

## Gotchas
- Do not commit unless explicitly asked
- Run `cargo clippy` and `cargo check` after code changes
- ORG013 (bare URL) ignores URLs in `#+BEGIN_SRC`/`#+END_SRC` and `#+BEGIN_EXAMPLE`/`#+END_EXAMPLE` blocks
- New rules: add to `all_rules()` (and `fixable_rules()` if applicable) in `src/rules/mod.rs`

# AGENTS.md

## Build, Lint, and Test Commands
- **Build:** `cargo build`
- **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Format:** `cargo fmt --all`
- **Test (all):** `cargo test`
- **Test (single):** `cargo test <testname>`

## Code Style Guidelines
- **Imports:** Group std, external, and internal modules separately. Use explicit imports.
- **Formatting:** Use `cargo fmt` (default Rust style).
- **Types:** Prefer explicit types for public APIs. Use `anyhow::Result` for error handling.
- **Naming:** Use `snake_case` for functions/variables, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Error Handling:** Use `anyhow::Result` and `?` for propagation. Add context with `.context()` where helpful.
- **Modules:** Organize code into modules in `src/`. Re-export public APIs in `lib.rs`.
- **Testing:** Add tests in `#[cfg(test)]` modules within each file. Use `cargo test` to run.

No Cursor or Copilot rules are present. Follow idiomatic Rust and these guidelines for consistency.

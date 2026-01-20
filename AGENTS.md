# Repository Guidelines

## Project Structure & Module Organization
- CLI entrypoint lives in `src/main.rs`; shared helpers sit in `clipboard.rs`, `keyring.rs`, `logger.rs`, `utils.rs`, and `keepass.rs`.
- Subcommands are in `src/commands/` (e.g., `init.rs`, `pwd.rs`, `totp.rs`, `list.rs`, `show.rs`, `add.rs`, `completion.rs`); match new behavior to this layout.
- Integration-style tests are under `tests/` with fixtures in `tests/files/`. Keep test assets small and free of real secrets (use placeholder databases like `tests/files/test.kdbx`).

## Build, Test, and Development Commands
- `just fmt` → format the workspace with nightly rustfmt.
- `just check` → formatting check plus `cargo clippy --all-features --all-targets -D warnings`.
- `just test` → run the Rust test suite.
- `just all` → run fmt, check, and test in sequence.
- Direct cargo equivalents: `cargo +nightly fmt --all`, `cargo clippy --all-features --all-targets -D warnings`, `cargo test`.
- `just cli-md` regenerates CLI help snippets for documentation; run when changing flags or command descriptions.

## Coding Style & Naming Conventions
- Rust 2024 edition with standard 4-space indentation; rely on rustfmt and keep diffs clean.
- Prefer snake_case for functions/modules, CamelCase for types, SCREAMING_SNAKE_CASE for constants.
- Keep subcommand definitions cohesive: argument parsing via `clap`, business logic in helper functions to ease testing.
- Clippy must be clean; address warnings rather than allowing suppressions unless justified.

## Testing Guidelines
- Tests use `rstest` and `assert_cmd` for CLI flows. Name tests with intent (e.g., `test_help`, `test_totp_raw`).
- Integration fixtures live in `tests/files/`; avoid adding new binary fixtures unless essential and documented.
- Run `just test` (or `cargo test`) before submitting; add focused tests for new behaviors and edge cases (invalid database path, missing key file, keyring off/on).

## Commit & Pull Request Guidelines
- Follow Conventional Commit-style prefixes seen in history (`build:`, `chore:`, `docs:`, etc.); include scope when helpful.
- Keep commits small and logically grouped; include why-driven messages for behavioral changes.
- Pull requests should describe the change, mention affected commands/flags, and note any doc/test updates. Link issues when available and add screenshots only if UI output changes meaningfully (e.g., help text).

## Security & Configuration Tips
- Never commit real KDBX files, key files, or secrets. Use placeholders for examples and tests.
- Respect env-driven settings: `KDBX_DATABASE`, `KDBX_KEY_FILE`, `KDBX_LOG`, `KDBX_LOG_STYLE`. Document defaults when altering behavior.
- For clipboard-related changes, ensure timeout/cleanup paths are covered and cross-platform considerations are noted (`clipboard` feature is default, optional in `Cargo.toml`).

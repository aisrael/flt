# General Guidelines

- This repo has a tracked pre-commit hook at `.githooks/pre-commit` that runs `cargo fmt --check` and `cargo clippy`. New clones/worktrees must run `git config core.hooksPath .githooks` once to activate it.
- When working in a Git worktree, as a final step make sure `cargo clippy` and `cargo test` both pass.
- When generating commit messages, if the staged changes are limited to one file, keep the commit message to one line.
- After a major change or refactoring, run `cargo +nightly fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` to ensure linting and tests pass

# Language Features

- Whenever planning or implementing a new language feature, start by writing the documentation under `docs/`, with examples. Then write Cucumber features (under `features/`) and unit tests that match those examples. Only then write the implementation.

# ADRs

- When key decisions are made by the user, offer to create an ADR if the user hasn't explicitly told you to do so
- Each new ADR should only refer to previous ADRs
- Forward-looking statements are ok, just not references to future ADRs

# Rust Code Guidelines

- When writing imports, use one `use` statement per line. DO NOT use grouped `use` statements
- When writing `println!()` statements, use named arguments whenever possible.
- When running `cargo fmt`, use `cargo +nightly fmt`

# General Guidelines

- This repo has a tracked pre-commit hook at `.githooks/pre-commit` that runs `cargo fmt --check` and `cargo clippy`. New clones/worktrees must run `git config core.hooksPath .githooks` once to activate it.
- When working in a Git worktree, as a final step make sure `cargo clippy` and `cargo test` both pass.
- When generating commit messages, if the staged changes are limited to one file, keep the commit message to one line.
- After a major change or refactoring, run `cargo +nightly fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` to ensure linting and tests pass

# Rust Code Guidelines

- When writing imports, use one `use` statement per line. DO NOT use grouped `use` statements
- When writing `println!()` statements, use named arguments whenever possible.
- When running `cargo fmt`, use `cargo +nightly fmt`

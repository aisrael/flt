# AGENTS.md

This file defines project-specific instructions for coding agents working in this repository.

## General Workflow

- When working in a Git worktree, as a final step make sure `cargo clippy` and `cargo test` both pass.
- When generating commit messages, if the staged changes are limited to one file, keep the commit message to one line.
- After a major change or refactoring, run:
  - `cargo +nightly fmt`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`

## Rust Code Guidelines

- Write imports as one `use` statement per line.
- Do not use grouped `use` statements.
- When writing `println!()` statements, use named arguments whenever possible.
- When running `cargo fmt`, use `cargo +nightly fmt`.

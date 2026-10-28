# General Guidelines

- This repo has a tracked pre-commit hook at `.githooks/pre-commit` that runs `cargo fmt --check` and `cargo clippy`. New clones/worktrees must run `git config core.hooksPath .githooks` once to activate it.
- When working in a Git worktree, as a final step make sure `cargo clippy` and `cargo test` both pass.
- When generating commit messages, if the staged changes are limited to one file, keep the commit message to one line.
- After a major change or refactoring, run `cargo +nightly fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` to ensure linting and tests pass

# Language Features

- Whenever planning or implementing a new language feature, start by writing the documentation under `docs/`, with examples. Then write Cucumber features (under `features/`) and unit tests that match those examples. Only then write the implementation.
- In general, keep `docs/spec.md` as broad as possible, and not runtime-specific. Describe what any flt runtime does, and leave details of the reference implementation (`SimpleRuntime`), such as its built-ins, to the other docs.
- In `docs/spec.md`, don't quote error messages. Name the `RuntimeError` variant instead, e.g. "raises an `UnexpectedKeywordArgument` error" rather than "raises `Function <name> got an unexpected keyword argument: <key>`". Leave the message text to the other docs.

# ADRs

- When key decisions are made by the user, offer to create an ADR if the user hasn't explicitly told you to do so
- Each new ADR should only refer to previous ADRs
- Forward-looking statements are ok, just not references to future ADRs

# Releases

Before bumping the version:

- Mark every ADR in the release as Accepted.
- Update `README.md` (feature summary, examples, Public API) and `docs/README.md` (contents and its "Scope" version) to match the release. The README is the crate's page on crates.io and docs.rs, and a published version can't be changed.
- Run `cargo doc --no-deps` and check that the README renders as the crate's front page.

Then, in a release PR:

- Bump the version in `Cargo.toml`, `Cargo.lock`, the README's dependency snippet and `features/cli.feature`.
- Rename the CHANGELOG's "Unreleased" section to `vX.Y.Z`.
- Run `cargo +nightly fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo publish --dry-run`.

After merging: tag `main` with an annotated `vX.Y.Z` tag ("Release vX.Y.Z"), push the tag, run `cargo publish`, then tell dependents (e.g. datu) to switch to the published version.

# Rust Code Guidelines

- When writing imports, use one `use` statement per line. DO NOT use grouped `use` statements
- When writing `println!()` statements, use named arguments whenever possible.
- When running `cargo fmt`, use `cargo +nightly fmt`

# Changelog

## v0.2.0

### Breaking Changes

- **`None` replaces `Unit`**: An `if` without `else` whose condition is `false` evaluates to `None` instead of `()`. `Unit` and `()` are gone.
- **Operator precedence**: Comparisons bind looser than arithmetic, so `1 + 2 == 3` is `true`. `|>` binds tighter than comparisons and logical operators, as in Elixir, so `x |> f == 3` means `(x |> f) == 3`.
- **Bindings can't shadow types, functions or keywords**: `let if = 1` is a parse error, and `let Number = 1` or `let typeof = 1` raises `Cannot bind <name>: …`. The same names are refused when a host binds variables or registers functions.
- **Built-ins are ordinary functions**: `typeof` is a native function like any other, rather than a special case, so hosts look it up the way they look up their own functions.
- **The `Runtime` trait is the host API**: A host works with any runtime through `flt::runtime::Runtime`, so it needs `use flt::runtime::Runtime` to register functions and native types or to read and bind variables. `runtime.eval` takes flt source, e.g. `runtime.eval("1 + 2")`, and what was `eval(&statement)` is `eval_statement`. `SimpleRuntime.global_scope` is no longer public.

### New Features

#### Language

- **Types as values**: A type name evaluates to that type, so `typeof(1) == Number` is `true` and `typeof(Number)` is `Type`.
- **`None` is its own type**: `typeof(None)` is `None`, and the `None` value equals the `None` type.
- **Union types**: A value can have type `A | B`, e.g. an `if` without `else` has type `T | None`. Unions are kept simple: `Number | Number` is `Number`. They can't be written in flt source yet.
- **Functions as values**: A function's name evaluates to a function value of type `fn(...) -> R`, which can be bound to another name and called through it, e.g. `let t = typeof; t(1)`. `Any` stands for any value in a signature, so `typeof` has type `fn(Any) -> Type`.
- **Pipe operator**: `x |> f(a)` evaluates as `f(x, a)`, and `x |> f` as `f(x)`, so `read("x") |> filter("a > 1") |> count()` reads left to right.
- **REPL**: `/inspect` shows a function's type, and `/unset` explains why a type or function can't be unset.

#### Embedding flt

- **Native functions**: A host registers Rust functions with the runtime, with signatures such as `fn(String) -> Number`, and flt source calls and pipes into them like any function.
- **Native types and values**: A host can make its own Rust types available to flt, e.g. a date or a lazy query plan. flt source can pass these values between native functions, print them, compare them with `==` and `!=`, and get their type (`typeof(d) == Date`), but only the host creates them or looks inside them. Function types can name them, e.g. `fn(Date, Date) -> Number`.
- **Host errors**: A native function can fail with the host's own error, which is shown exactly as the host writes it.
- **Evaluating source**: `runtime.eval` parses and runs flt source, with several statements separated by newlines or `;`, and returns the last value. A syntax error runs nothing. `flt::parser::parse_statements` parses such source without running it.
- **REPL on the host's runtime**: `FltRepl::with_runtime` runs flt's REPL on a runtime the host has set up, with its functions and native types.
- **Numbers to and from Rust**: A host reads number arguments as Rust integers or floats, with clear errors such as `Expected Number, found String` or `Cannot convert -1 to u32`, and returns Rust integers as flt numbers.

#### Docs

- A one-page language spec (`docs/spec.md`), a guide to native functions, types and values (`docs/native-functions-and-values.md`), a guide to the `Runtime` trait (`docs/runtime-and-limitations.md`), and ADRs 001 to 008.

## v0.1.2

- **REPL commands**: `/parse`, `/unset`, `/inspect` (alias `/i`), `/quit`, and `/help`.
- **REPL refactor**: Generic `Repl<H: ReplHandler>` with flt-specific logic in `FltRepl`, and a `SlashCommands` registry.
- **Custom prompts**: `ReplHandler::prompt()` lets handlers vary the prompt.
- **Configurable history**: `Repl::new` takes an optional history path (`None` disables history); `default_history_path()` is now public.
- **Env vars**: `FLT_HISTORY_PATH` overrides the history file location; `FLT_NO_HISTORY` disables history.
- **Dev**: Tracked pre-commit hook (`.githooks/pre-commit`) runs `cargo fmt --check` and `cargo clippy`.

## v0.1.1

### Highlights

- **Dot-access syntax for maps**: `u.foo` reads a field off a map value, supported at every expression precedence level (including unary and `if` conditions), with a runtime error for missing fields or non-map operands.
- **`typeof` builtin**: Runtime type introspection for values.

### Improvements

- **Fix**: Map literals (`{ key: value }`) now evaluate to `Value::Map` instead of erroring.
- **Fix**: Array literals (`[ expr, ... ]`) now evaluate to `Value::Array` instead of erroring.

## v0.1.0

- **REPL**: Interactive read-eval-print loop and `flt version` CLI.
- **Literals**: Numbers (arbitrary precision), strings, booleans, and symbols (`:foo`, `:"quoted"`).
- **Expressions**: Identifiers, unary (`!`, `+`, `-`), binary (arithmetic, logical, bitwise, pipe `|>`, string concat `<>`), function calls with optional key-value arguments, parenthesized expressions, map literals `{ key: value }`, and array literals `[ expr, ... ]`.
- **String interpolation**: `"Hello, {expr}!"` with arbitrary expressions inside `{}`.
- **Conditionals**: `if <cond> { <then> } else { <else> }` (block branches) and `if <cond> <then_expr> else <else_expr>` (expression branches). The `else` branch is optional.
- **Assignment / Keywords**: `let x = expr;` and `x = expr;` (semicolon/newline handling is parser-dependent) plus core keywords like `let`, `if`, and `else`.
- **Comments**: Line comments from `#` to end of line.
- **Evaluation**: Literals, unary and binary operators (arithmetic, logical, string concat), parenthesized expressions, and `if` expressions. Function calls and map/array literals are not yet evaluated (runtime error if reached).

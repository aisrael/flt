# Changelog

## v0.2.0

### Breaking Changes

- **`None` replaces `Unit`**: An `if` without `else` whose condition is `false` now evaluates to `None` instead of `()`. `Unit` and `()` are removed.
- **Operator precedence**: Comparisons bind looser than arithmetic, so `1 + 2 == 3` is `true`. `|>` binds tighter than comparisons and logical operators, as in Elixir, so `x |> f == 3` is `(x |> f) == 3`.
- **`SimpleRuntime.built_in_functions` is removed**: Built-ins such as `typeof` are native functions in the global scope; read them with `global_scope.get_function`.
- **Bindings can't shadow types, functions or keywords**: A reserved keyword as a binding name (`let if = 1`) is a parse error. Binding a built-in type name (`None`, `Number`, …, `Any`) or the name of a function, built-in (`typeof`) or registered by the host, raises an error. `GlobalScope::set_variable` returns a `Result`, `register_function` panics on a built-in type name or a name that's already a variable, and `GlobalScope`'s `functions` and `variables` maps are private, so hosts go through those methods.
- **New enum variants**: `Type::Native`, `Value::Native`, `Error::Native`, `RuntimeError::UnexpectedType`, `RuntimeError::CannotConvert` and `ProtectedName::NativeType`, so hosts that match on these enums exhaustively need a new arm.
- **`SimpleRuntime` has a private field**: It holds the registered native types, so it can't be built with a struct literal; use `SimpleRuntime::default()`.

### Highlights

- **`None` is its own type**: `typeof(None)` is `None`, and the `None` value equals the `None` type.
- **Types as values**: A built-in type name evaluates to that type, so `typeof(1) == Number` is `true`, and `typeof(Number)` is `Type`. Built-in type names can't be rebound.
- **`Any`**: A placeholder used only in signatures; `typeof` has type `fn(Any) -> Type`.
- **Union types**: The runtime has union types `A | B`, kept simplified: nested unions are flattened, duplicates are removed keeping the first, and a single member collapses, so `Number | Number` is `Number`. Union types can't be written in flt source yet.
- **Functions as values**: Hosts can register native functions in the global scope. A function's name evaluates to a function value of type `fn(...) -> R`, which can be bound to another name and called through it. Calling an unknown name raises `Undefined function: <name>`.
- **Pipe operator**: `x |> f(a)` evaluates as `f(x, a)`, and `x |> f` as `f(x)`. Piping into anything else raises `Cannot pipe into <expr>`. Calls next to `|>` need parentheses: `1 |> f 2` and `f 1 |> g()` are parse errors.
- **`typeof` is a function value**: `typeof` is a native function in the global scope, so `typeof(typeof)` is `fn(Any) -> Type` and `let t = typeof` works. In the REPL, `/inspect` reports a function's type.
- **Native types and values**: A host's Rust type that implements `NativeType` (a name, plus `Display` and `PartialEq`) can be registered with the runtime (`SimpleRuntime::register_native_type`), as can native functions (`SimpleRuntime::register_function`, alongside `GlobalScope::register_function`). Native functions create and read its values (`Value::native`, `get_native`, `downcast_native`), which flt source can bind, pass, pipe, print and compare with `==`/`!=`, but not create or look inside. A native type's name evaluates to that type, can't be bound, and can appear in function types, e.g. `fn(Date, Date) -> Number`.
- **Host errors**: Native functions can fail with the host's own error (`Error::native`), which is shown unchanged.
- **Numbers to and from Rust**: `Value::from` accepts `u64`, `i64` and `usize`, and `i32`, `i64`, `u32`, `u64`, `usize` and `f64` implement `TryFrom<&Value>`, raising `Expected Number, found <type>` or `Cannot convert <number> to <type>`.
- **Docs**: A one-page language spec (`docs/spec.md`), and ADRs 001, 002, 003, 004, 005, 006 and 007.

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

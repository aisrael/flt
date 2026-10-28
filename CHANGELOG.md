# Changelog

## Unreleased

### Breaking Changes

- **Handlers read their arguments by name**: A native function's handler takes `&Arguments` instead of `&[Value]` (and `&Map`), and reads each argument by the name its signature gives it, converted to a Rust type: `args.get::<&str>("path")?`, `args.get::<usize>("n")?` or `args.native::<Date>("from")?`. The name is looked up in the overload that matched, whether the call passed it positionally or as a keyword. A variadic argument's values come from `args.rest()`, and the keywords from `args.keywords()` ([ADR 014](https://github.com/aisrael/flt/blob/main/docs/adrs/014-native-function-arguments.md)).
- **Errors name the closest overload**: When several overloads take a call's number of arguments and none matches, the error is the one for the overload that passes the most checks before one fails, so `head(plan: Plan, n: Number)` and `head(path: String, n: Number)` give `read("data.parquet") |> head("x")` the error `Argument n of head expects Number, found String`. When overloads first fail at the same keyword, one that accepts it with another type is closer than one that doesn't accept it. `No overload of … accepts (…)` is left for overloads that are equally close. Hosts that compare error messages may need to update them ([ADR 015](https://github.com/aisrael/flt/blob/main/docs/adrs/015-closest-overload-errors.md)).
- **One constructor**: `NativeFunction::new` takes every handler, with or without keywords. `NativeFunction::with_keywords` is gone, and `new` no longer panics on a definition that declares keywords.
- **Handler errors**: A new kind of error, `Error::HandlerError`, for a handler using flt's API wrongly: `Handler Error: Function <name> has no argument <argument>` for reading a name the matching overload doesn't declare, and `Handler Error: Argument <argument> of <name> is variadic; read it with Arguments::rest` for reading a variadic argument as one value.

## v0.3.0

### Breaking Changes

- **Function calls are type-checked**: Every call checks its positional arguments, variadic values and keyword values against the declared types, and the function's result against its return type, so `pad(1, 2)` raises `Argument s of pad expects String, found Number` before `pad` runs. `Any` accepts every value, so functions declared with `Any` behave as before. Hosts need declared types that match what their handlers accept and return. New errors: `Argument … expects …`, `Keyword argument … expects …`, `No overload of … accepts (…)` and `Function … returned …, expected …` ([ADR 013](https://github.com/aisrael/flt/blob/main/docs/adrs/013-type-checked-function-calls.md)).
- **Overloads are picked by arity, types and keywords**: Overloads are tried by arity, longest first, and the first whose types and keywords match is called, so declaration order no longer decides which one runs. Among matching overloads of the same arity, `SimpleRuntime` calls the one declared first.
- **One error for the wrong number of arguments**: `RuntimeError::ArgumentCountMismatch`'s `expected` is an `ArgumentCounts`, listing every count the function's overloads take, e.g. `Function f expected 1 or at least 3 argument(s), found 2`, and `RuntimeError::TooFewArguments` is gone. A function with one signature prints the same messages as before.
- **`FunctionDefinition::resolve` takes the call's values and keywords** instead of a count, and `FunctionDefinition::keyword` panics on a default of the wrong type.
- **Variadic parameters print as `*T`**: Function types show a variadic parameter as `*T` instead of `...T`, matching `**T` for rest keywords, so `typeof(sprintf)` prints `fn(String, *Any) -> String` ([ADR 012](https://github.com/aisrael/flt/blob/main/docs/adrs/012-variadic-notation.md)).

### New Features

- **`flt::ast::KeywordArgs`**: The type of `FunctionCall.keyword_args` has a name, so a host can write it in its own signatures, e.g. `fn select_items(keywords: &KeywordArgs)`, without depending on `indexmap` itself. It's an alias for `IndexMap<String, Expr>`, so code that already writes that type keeps compiling.

## v0.2.0

### Breaking Changes

- **`None` replaces `Unit`**: An `if` without `else` whose condition is `false` evaluates to `None` instead of `()`. `Unit` and `()` are gone.
- **Operator precedence**: Comparisons bind looser than arithmetic, so `1 + 2 == 3` is `true`. `|>` binds tighter than comparisons and logical operators, as in Elixir, so `x |> f == 3` means `(x |> f) == 3`.
- **Bindings can't shadow types, functions or keywords**: `let if = 1` is a parse error, and `let Number = 1` or `let typeof = 1` raises `Cannot bind <name>: …`. The same names are refused when a host binds variables or registers functions.
- **Built-ins are ordinary functions**: `typeof` is a native function like any other, rather than a special case, so hosts look it up the way they look up their own functions.
- **The `Runtime` trait is the host API**: A host works with any runtime through `flt::runtime::Runtime`, so it needs `use flt::runtime::Runtime` to register functions and native types or to read and bind variables. `runtime.eval` takes flt source, e.g. `runtime.eval("1 + 2")`, and what was `eval(&statement)` is `eval_statement`. Registering a function or native type returns a `Result`: a name that's already in use is a `Cannot register …` error rather than a panic. `SimpleRuntime.global_scope` is no longer public.
- **Map keys are symbols**: `{b: 1}` has the key `:b`, and a quoted key `{"c d": 1}` the quoted symbol `:"c d"`, so `{b: 1} == {"b": 1}`. Keyword arguments get symbol keys too, and `m.b` reads `:b`. Maps print keys as they'd be written, e.g. `{b: 1, "c d": 2}`. `Value::Map` holds a `flt::runtime::Map` keyed by `MapKey`. Symbols print quoted when their name isn't a bare name, e.g. `:"hello world"`.
- **Maps keep insertion order**: In `SimpleRuntime`, maps keep insertion order, and print in insertion order instead of sorted, e.g. `{b: 1, a: 2}` prints `{b: 1, a: 2}`. Equality still ignores order.
- **Keyword arguments aren't a trailing map**: A function gets a call's keyword arguments by name, separately from its positional arguments, instead of as one map at the end. A keyword the function doesn't declare raises `Function <name> got an unexpected keyword argument: <key>`, and a repeated keyword, as in `f(n: 1, n: 2)`, doesn't parse: `Duplicate keyword argument n in the call to f`. `Expr::FunctionCall` holds a `FunctionCall`, with keyword arguments in `keyword_args` instead of a trailing `MapLiteral`. `FunctionSignature` and `FunctionType` gain public fields.
- **Variadic arguments**: `Argument` and `FunctionType` gain a public `variadic` field, so hosts that build them with struct literals break.
- **Parser errors**: The parsers in `flt::parser` fail with `flt::parser::ParseError`, which can say what's wrong in flt's terms, instead of nom's `nom::error::Error`.

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
- **Variadic arguments**: `Argument::variadic` lets a native function take any number of trailing positional arguments. Function types show them as `...T`, e.g. `fn(String, ...Any) -> String`, and a call with too few raises `Function <name> expected at least <minimum> argument(s), found <found>`.
- **Keyword arguments**: A native function declares optional keyword arguments with defaults (`FunctionDefinition::keyword`), or accepts any keywords (`rest_keywords`), and its handler gets them as a map (`NativeFunction::with_keywords`). Function types show them, e.g. `fn(String, greeting: String) -> String`.
- **Evaluating source**: `runtime.eval` parses and runs flt source, with several statements separated by newlines or `;`, and returns the last value. A syntax error runs nothing. `flt::parser::parse_statements` parses such source without running it.
- **REPL on the host's runtime**: `FltRepl::with_runtime` runs flt's REPL on a runtime the host has set up, with its functions and native types.
- **Numbers to and from Rust**: A host reads number arguments as Rust integers or floats, with clear errors such as `Expected Number, found String` or `Cannot convert -1 to u32`, and returns Rust integers as flt numbers.
- **Ordered maps**: A native function sees a map's entries in source order. Hosts build maps with `flt::runtime::Map`, with symbol or string keys (`MapKey`).

#### Docs

- A one-page language spec (`docs/spec.md`), a guide to native functions, types and values (`docs/native-functions-and-values.md`), a guide to the `Runtime` trait (`docs/runtime-and-limitations.md`), and ADRs 001 to 011.

### Fixes

- **Printed symbols keep their `:`**: An expression with a symbol prints as flt source again, so `select(:id, :"full name")` prints as written instead of `select(id, full name)`.

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

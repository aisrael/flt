[![Crates.io](https://img.shields.io/crates/v/flt.svg)](https://crates.io/crates/flt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/aisrael/flt/blob/main/LICENSE)

# flt

A lightweight functional language with a parser, REPL, and a rudimentary interpreter/runtime.

## Overview

`flt` (pronounced "flight") is a single Rust crate that provides:

- a parser + AST (`flt::parser`, `flt::ast`)
- a REPL (`flt::repl`) backed by a simple interpreter/runtime (`flt::runtime`)

The language supports literals, identifiers, unary/binary operators, function calls, interpolation, comments, and an Elixir-style pipe operator.
The runtime currently evaluates a subset of that syntax.

## Crate version

```toml
[dependencies]
flt = "0.2.0"
```

## Quick start

```rust
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;

fn main() {
    let mut runtime = SimpleRuntime::default();
    let value = runtime.eval("let x = 1 + 2; x * 2").expect("evaluation should succeed");
    println!("{value}");
}
```

A host works with a runtime only through the `Runtime` trait, and names `SimpleRuntime`, the reference implementation, only to create one. See [The `Runtime` Trait](https://github.com/aisrael/flt/blob/main/docs/runtime-and-limitations.md#the-runtime-trait), and [Native Functions, Types and Values](https://github.com/aisrael/flt/blob/main/docs/native-functions-and-values.md) for extending flt from Rust.

## Language syntax

### Literals

- Number: `42`, `3.14`, `+7`, `-2`
- String: `"hello"`, `"path\\to\\file"`
- Boolean: `true`, `false`
- Symbol: `:name`, `:"display name"`

### Identifiers

Identifiers start with a letter, then can include letters, digits, `_`, or `-`.

- Valid: `foo`, `foo_1`, `foo-1`, `READ`
- Invalid: `_foo`, `-foo`, `123foo`

### Function calls

Both forms are supported:

- Parenthesized: `add(1, 2)`
- Without parentheses: `add 1, 2`

### String interpolation

Interpolated strings support `{expr}` and compile into concatenation (`<>`) expressions.

```txt
"Hello, {name}!"
"Answer: {1 + 2}"
```

Inside interpolated strings, `\{` escapes a literal `{`.

### Comments

`#` starts a comment that runs to end-of-line.

```txt
1 + 2 # inline comment
# full line comment
```

### Operators

Unary operators:

- `!` (logical not)
- `+` (unary plus)
- `-` (unary minus)

Binary operators:

- Arithmetic: `+`, `-`, `*`, `/`
- String concat: `<>`
- Logical: `&&`, `||`, `^^`
- Bitwise: `&`, `|`, `^`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Pipe: `|>` (`x |> f(a)` evaluates as `f(x, a)`)

### Precedence (lowest to highest)

`||` → `&&` → `^^` → `==`/`!=` → `<`/`>`/`<=`/`>=` → `|` → `^` → `&` → `|>` → `+`/`-`/`<>` → `*`/`/`

Parentheses override precedence as expected.

## Parse examples

```rust
use flt::parser::parse_expr;

parse_expr("42");
parse_expr(r#""hello""#);
parse_expr(":id");
parse_expr("add(1, 2)");
parse_expr("add 1, 2");
parse_expr(r#""Hello, {who}!""#);
parse_expr(r#""foo" <> "bar""#);
parse_expr("1 + 2 * 3");
parse_expr("(1 + 2) * 3");
parse_expr("READ(\"input\") |> WRITE(\"output\")");
parse_expr("1 # comment\n+ 2");
```

## Binary / REPL

Run the REPL:

```bash
cargo run
```

Print version:

```bash
cargo run -- version
```

The runtime evaluates expressions and statements: literals (including maps and arrays), identifiers, unary/binary operators, `if` expressions, `let` bindings, function calls, and pipes. Bitwise operators and standalone keyword expressions are parsed but not yet evaluated. See [`docs/runtime-and-limitations.md`](https://github.com/aisrael/flt/blob/main/docs/runtime-and-limitations.md).

## Public API

- `flt::parser`:
  - `parse_expr`
  - `parse_statement`
  - `parse_statements` (all of a source's statements, separated by newlines or `;`)
  - `parse_literal`
  - `parse_identifier`
  - `parse_number`
  - `parse_string`
  - `parse_symbol`
  - `parse_binary_op`
  - `parse_unary_op`
- `flt::ast`:
  - `Expr`
  - `FunctionCall` (a call's name, positional arguments and keyword arguments)
  - `KeywordArgs` (a call's keyword arguments, in source order)
  - `Literal`
  - `Identifier`
  - `Numeric`
  - `BinaryOp`
  - `UnaryOp`
- `flt::repl`:
  - `Repl`
  - `FltRepl` (`FltRepl::with_runtime` runs the REPL on a host's runtime)
- `flt::runtime`:
  - `Runtime` (the runtime API: `eval`, `eval_statement`, registering native functions and types, and global variables)
  - `SimpleRuntime` (the reference implementation)
  - `Value`
- `flt::eval`:
  - `eval` (evaluate a parsed `Expr` with a fresh runtime)
- `flt::Error` and `flt::errors::RuntimeError`

## Development

```bash
cargo test
cargo clippy --all-targets --all-features
```

## License

MIT

# flt

A lightweight functional language and parser.

## Overview

`flt` (pronounced "flight") is a Rust workspace with:

- `flt`: parser + AST library
- `flt-cli`: REPL and expression evaluator

The language supports literals, identifiers, unary/binary operators, function calls, interpolation, comments, and an Elixir-style pipe operator.

## Crate version

```toml
[dependencies]
flt = "0.0.2"
```

## Quick start

```rust
use flt::parser::parse_expr;

fn main() {
    let input = "1 + 2";

    match parse_expr(input) {
        Ok((remainder, expr)) if remainder.trim().is_empty() => {
            println!("Parsed expression: {expr}");
        }
        Ok((remainder, _)) => {
            eprintln!("Unconsumed input: {:?}", remainder);
        }
        Err(err) => {
            eprintln!("Parse error: {:?}", err);
        }
    }
}
```

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
- Pipe: `|>`

### Precedence (lowest to highest)

`|>` -> `||` -> `&&` -> `^^` -> `|` -> `^` -> `&` -> `+`/`-`/`<>` -> `*`/`/`

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

## CLI (workspace binary)

Run the REPL:

```bash
cargo run -p flt-cli
```

Print version:

```bash
cargo run -p flt-cli -- version
```

The CLI evaluates literals and supported unary/binary expressions. Function calls and pipe execution are parsed but currently not executed by the evaluator.

## Public API

- `flt::parser`:
  - `parse_expr`
  - `parse_literal`
  - `parse_identifier`
  - `parse_number`
  - `parse_string`
  - `parse_symbol`
  - `parse_binary_op`
  - `parse_unary_op`
- `flt::ast`:
  - `Expr`
  - `Literal`
  - `Identifier`
  - `Numeric`
  - `BinaryOp`
  - `UnaryOp`
- `flt::Error` and `flt::errors::RuntimeError`

## Development

```bash
cargo test
cargo clippy --all-targets --all-features
```

## License

MIT

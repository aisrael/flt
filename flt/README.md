# flt

A simple, functional programming language.

## Overview

flt (pronounced "functional") is a lightweight functional language implementation. It provides an expression parser and abstract syntax tree (AST) for a language with literals, identifiers, operators, function calls, and an Elixir-style pipe operator.

## Features

- **Literals**: numbers (arbitrary precision via `BigDecimal`), strings, booleans, and symbols (`:foo`, `:"hello"`)
- **Operators**:
  - Unary: `!`, `+`, `-`
  - Binary: `+`, `-`, `*`, `/`, `&`, `&&`, `|`, `||`, `^`, `^^`, `|>` (pipe)
- **Function calls**: `foo()`, `bar(1)`, `add(1, 2)`
- **Pipe operator**: `a |> b |> c` — passes the left value as the first argument to the right
- **Operator precedence** (lowest to highest): `||`, `&&`, `^^`, `|`, `^`, `&`, `+`/`-`, `*`, `/`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
flt = "0.0.1"
```

## Usage

```rust
use flt::parser::parse_expr;

fn main() {
    let input = "1 + 2 * 3";
    match parse_expr(input) {
        Ok((remainder, expr)) => {
            if remainder.is_empty() {
                println!("Parsed: {:?}", expr);
            } else {
                eprintln!("Unconsumed input: {:?}", remainder);
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

### Parsing expressions

```rust
use flt::parser::parse_expr;

// Numbers
parse_expr("42");        // Literal number
parse_expr("3.14");      // Decimal

// Strings and symbols
parse_expr(r#""hello""#);  // String literal
parse_expr(":foo");        // Symbol

// Booleans
parse_expr("true");
parse_expr("false");

// Function calls
parse_expr("foo()");
parse_expr("add(1, 2)");

// Pipe operator
parse_expr("1 |> add(2)");
parse_expr(r#"READ("input") |> SELECT(:id) |> WRITE("output")"#);
```

## Public API

- **`parser`**: `parse_expr`, `parse_literal`, `parse_identifier`, `parse_number`, `parse_string`, `parse_symbol`, `parse_binary_op`, `parse_unary_op`
- **`ast`**: `Expr`, `Literal`, `Identifier`, `BinaryOp`, `UnaryOp`
- **`Error`**: Error types for parsing and runtime

## License

MIT

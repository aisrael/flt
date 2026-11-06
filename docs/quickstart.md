# Quickstart

`flt` is a lightweight expression language with a REPL and CLI in the `flt` crate.

## Check Version

```bash
cargo run -- version
```

Expected output:

```text
flt version 0.3.0
```

## Run a Program with `-e`

Use `-e` to evaluate a string as an flt program, then exit. Statements are separated by newlines or by `;`. `flt` prints the value of the last statement.

```bash
cargo run -- -e '1 + 2'
```

Expected output:

```text
3
```

```bash
cargo run -- -e 'let x = 2; x * 3'
```

Expected output:

```text
6
```

A parse error or an evaluation error goes to standard error, and `flt` exits with status 1. Warnings go to standard error as `Warning: <message>`.

## Start the REPL

```bash
cargo run
```

You will get a prompt:

```text
>
```

## Try a Few Expressions

```flt
1 + 2 * 3
"Hello, " <> "world"
true && false
"Answer: {1 + 2}"
```

## Try bindings

```flt
let x = 10
x + 1
```

## Try maps and field access

```flt
let u = { name: "Ada", age: 36 }
u.name
typeof(u.age)
```

## Notes

- The REPL parses one statement per input line (an expression or `let` / `name = expr`).
- Input with unconsumed trailing text after a complete statement is rejected.
- Press `Ctrl+C` or `Ctrl+D` to exit.

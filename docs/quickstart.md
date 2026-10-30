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

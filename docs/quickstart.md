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

## Run a Program File

Give `flt` the path of a program file. By convention, the file name ends in `.flt`. `flt` reads the file, parses it, runs it, and prints the value of the last statement, like `-e`.

Save this as `hello.flt`:

```flt
# hello.flt
println("Hello, world!")
```

```bash
cargo run -- hello.flt
```

Expected output:

```text
Hello, world!
None
```

`println` prints its argument. `flt` then prints the value of the last statement, and `println` evaluates to `None`.

A first line that starts with `#!` is a comment, so a program file can start with a shebang line, such as `#!/usr/bin/env flt`.

If `flt` cannot read the file, or the program has a parse error or an evaluation error, the message goes to standard error and `flt` exits with status 1:

```text
Error: cannot read missing.flt: No such file or directory (os error 2)
```

Notes:

- You cannot use `-e` and a program file together.
- `version` is a command, so to run a file with that name, write `./version`.

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

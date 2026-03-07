# Quickstart

`flt` is a lightweight expression language with a CLI in `flt-cli`.

## Check Version

```bash
cargo run -p flt-cli -- version
```

Expected output:

```text
flt version 0.0.2
```

## Start the REPL

```bash
cargo run -p flt-cli
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

## Notes

- The REPL parses one expression per input line.
- Expressions with unconsumed trailing text are rejected.
- Press `Ctrl+C` or `Ctrl+D` to exit.

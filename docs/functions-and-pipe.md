# Function Calls and Pipe

## Function Call Forms

`flt` supports two call styles.

### Parenthesized form

```flt
foo()
bar(1)
add(1, 2)
```

### Whitespace form

In this form, at least one argument is required.

```flt
add 1
add 1, 2
```

## Pipe Operator

The pipe operator is written as `|>`.

```flt
a |> b |> c
READ("input") |> SELECT(:id) |> WRITE("output")
```

The parser treats this as a left-associative binary operator chain.

## Current Semantics Note

- Parsing for function calls and pipe expressions is implemented.
- Runtime evaluation for function calls and pipe is not implemented yet in `flt-cli`.

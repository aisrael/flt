# Function Calls and Pipe

## Function Call Forms

`flt` supports two call styles.

### Parenthesized form

```flt
foo()
bar(1)
add(1, 2)
```

### Keyword arguments

In the parenthesized form, arguments may include trailing **keyword arguments**: key-value pairs with the form `key: value`. All positional arguments must come first; after the first keyword argument, no further positional arguments are allowed.

```flt
foo(1, bar: true)
baz(a: 1, b: 2)
qux(1, 2, option: "value")
```

Keys follow the same rules as map keys (bare identifier or quoted string). The parser collects keyword arguments into a single map and passes them as the final argument to the call.

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

- Parsing for function calls (including keyword arguments) and pipe expressions is implemented.
- Runtime evaluation for function calls and pipe is not implemented yet in `flt-cli`.

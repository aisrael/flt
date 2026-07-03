## Literals

### Numbers

Numbers are parsed as arbitrary-precision decimals.

Examples:

```flt
42
3.14
+7
-0.5
42.
```

### Strings

Strings use double quotes.

Examples:

```flt
"hello"
"say \"hello\""
"path\\to\\file"
```

### Interpolated Strings

Expressions inside `{ ... }` are parsed and concatenated into the surrounding string.

Examples:

```flt
"Hello, {who}!"
"Answer: {1 + 2}"
```

Use `\{` to include a literal `{`.

### Booleans

```flt
true
false
```

### None

`None` is the sentinel literal for an empty [`Option`](./types.md#option).

```flt
None
```

### Symbols

Symbols are prefixed with `:` and support two forms:

- Bare symbols: `:` followed by one or more `a-z`, `A-Z`, `0-9`, `_`, or `-`
- Quoted symbols: `:"..."` (string escapes are supported)

Examples of valid bare symbols include `:id`, `:abc123`, `:_tmp`, and `:foo-bar`.

Examples:

```flt
:id
:user_name
:"hello world"
```

### Maps

A literal map is a dictionary-like collection with unique keys mapping to their associated values. They start with `{` and end with `}`. Keys are similar to symbols, they can be bare keys (a valid identifier sequence), or quoted, then followed by a `:`. The value is any valid expression.

Examples:

```flt
{ foo: "bar" }
```

```flt
{ abc123: 456 }
```

```flt
{ "spaced out": (1 + 1) }
```

Fields can be read back off a map with [dot-access syntax](./field-access.md), e.g. `{ foo: "bar" }.foo`.

### Arrays

A literal array is an ordered collection of values. They start with `[` and end with `]`, with elements separated by commas. A trailing comma is allowed, and elements can be any valid expression.

Examples:

```flt
[]
[ 1, 2, 3 ]
[ "a", "b", ]
[ 1 + 2, foo() ]
```

### KeywordArgs

`KeywordArgs` are a special case of `Map`: trailing `key: value` pairs in a function call are collected into a single map and passed as the call's final argument. See [Keyword Arguments](./functions-and-pipe.md#keyword-arguments) for the call syntax, e.g.:

```flt
foo(1, bar: true)
warn("Value is out of range: {value}", value: (x - 1), request_id: request_id)
```

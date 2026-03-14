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

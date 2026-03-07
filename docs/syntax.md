# Syntax

This page covers the core surface syntax supported by the parser.

## Expression Forms

`flt` currently parses these expression forms:

- Literal values (`number`, `string`, `boolean`, `symbol`)
- Identifiers
- Unary expressions
- Binary expressions
- Function calls
- Parenthesized expressions

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

## Identifiers

Identifiers are parsed as one or more of:

- letters and digits
- `_`
- `-`

In practice, expression parsing prefers literals before identifiers. For example, `true` and `false` parse as booleans, and a leading numeric form is parsed as a number first.

Examples:

```flt
x
abc123
user_name
foo-bar
_tmp
```

## Comments and Whitespace

Comments start with `#` and continue to the end of the line.

Examples:

```flt
42 # inline comment

# full line comment
1 + 2
```

Comments are allowed where whitespace is allowed, including between function arguments.

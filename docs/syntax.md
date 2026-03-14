# Syntax

This page covers the core surface syntax supported by the parser.

## Expression Forms

`flt` currently parses these expression forms:

- [Literals](./literals.md)
- Identifiers
- Unary expressions
- Binary expressions
- Function calls
- Parenthesized expressions

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

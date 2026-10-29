# Syntax

This page covers the core surface syntax supported by the parser.

## Expression Forms

`flt` currently parses these expression forms:

- [Literals](./literals.md), including [map](./literals.md#maps) and [array](./literals.md#arrays) literals
- Identifiers
- Unary expressions
- Binary expressions
- [Field access](./field-access.md) (`u.foo`)
- Function calls (including [keyword arguments](./functions-and-pipe.md#keyword-arguments))
- Parenthesized expressions
- `if` expressions
- Reserved keywords as expressions (e.g. `return`, `fn`)

## Statements

The parser supports **let bindings**:

```flt
let x = 1
let name = "flt"
let foo = 2 + 3
```

A `let` binding can declare the variable's [type](./types.md#type-annotations), after a `:`:

```flt
let n: Number = 1
let s: String | None = None
```

A binding without `let` is an **assignment**: `x = 2`. `let` declares a variable, replacing any earlier declaration of it, and an assignment changes its value. If the variable was declared with a type, an assignment must keep to it:

```flt
let n: Number = 1
n = 2                # ok
n = "two"            # error: Variable n expects Number, found String
let n = "two"        # ok: declares n again, without a type
```

Only `let` takes a type, so `x: Number = 1` is a parse error.

- A statement may be followed by an optional `;`.
- If a statement ends on a newline, the semicolon is not required.
- Two statements on the same line require `;` after the first: `let x = 1; let y = 2`.

The REPL parses one statement per line (an expression or a `let` / assignment binding).

## Reserved Keywords

The following words are reserved and recognized with word boundaries (e.g. `if` is a keyword, but `iffy` is an identifier):

| Keyword | Keyword | Keyword |
| --- | --- | --- |
| `if` | `else` | `return` |
| `and` | `or` | `not` |
| `for` | `in` | `let` |
| `while` | `do` | `fn` |

A reserved keyword can't be the name in a binding, so `let if = 1` and `fn = 1` are parse errors. Words that only start with a keyword, such as `iffy`, can be bound.

## Identifiers

Identifiers are parsed as one or more of:

- letters and digits
- `_`
- `-`

In practice, expression parsing prefers literals before identifiers. For example, `true` and `false` parse as booleans, and a leading numeric form is parsed as a number first. Reserved keywords (e.g. `if`, `let`) are parsed as keywords when they appear as whole words; identifiers like `iffy` or `input` do not match the `if` or `in` keyword.

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

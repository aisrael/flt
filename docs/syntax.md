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
let n: Integer = 1
let s: String | None = None
```

A binding without `let` is an **assignment**: `x = 2`. `let` declares a variable, replacing any earlier declaration of it, and an assignment changes its value. If the variable was declared with a type, an assignment must keep to it:

```flt
let n: Integer = 1
n = 2                # ok
n = "two"            # error: Variable n expects Integer, found String
let n = "two"        # ok: declares n again, without a type
```

An assignment can't declare a type, so `x: Integer = 1` is a parse error. Use `let x: Integer = 1`.

A **function declaration** starts with `def`, and its body ends with `end` ([ADR 019](./adrs/019-def-function-declarations.md)):

```flt
def greet(name: String, greeting: String = "Hello") -> String
  greeting <> ", " <> name
end
```

A declaration has a name, the parameters in parentheses, an optional `-> type`, and a body. See [Declaring Functions](./functions-and-pipe.md#declaring-functions).

- A statement may be followed by an optional `;`.
- If a statement ends on a newline, the semicolon is not required.
- Two statements on the same line require `;` after the first: `let x = 1; let y = 2`.

The REPL parses one statement per line (an expression, a `let` / assignment binding or a `def`). A `def` must be written on one line in the REPL, e.g. `def double(n: Number) -> Number n * 2 end`.

## Reserved Keywords

The following words are reserved and recognized with word boundaries (e.g. `if` is a keyword, but `iffy` is an identifier):

| Keyword | Keyword | Keyword  |
| ------- | ------- | -------- |
| `if`    | `else`  | `return` |
| `and`   | `or`    | `not`    |
| `for`   | `in`    | `let`    |
| `while` | `do`    | `fn`     |
| `def`   | `end`   |          |

A reserved keyword can't be the name in a binding or in a `def`, so `let if = 1`, `fn = 1` and `def end() end` are parse errors. Words that only start with a keyword, such as `iffy`, can be bound.

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

Comments are allowed where whitespace is allowed, including between the arguments of a call. One exception: a call without parentheses takes its first argument from the same line as the name, so a comment can follow that call but not sit between the name and its first argument:

```flt
add(1, # first
    2)           # ok
add 1, # first
    2            # ok
add # name
  1              # not a call: `add` and `1` are two statements
```

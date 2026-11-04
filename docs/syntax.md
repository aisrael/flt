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
- [`if` expressions](#if-expressions), compact and expanded, with `elsif`
- [Brace blocks](#brace-blocks) (`{ let a = 1; a + 1 }`)
- Reserved keywords as expressions (e.g. `return`, `fn`)

## Brace Blocks

A **brace block** holds statements between `{` and `}`. It is an expression. It evaluates to its last statement. An empty block evaluates to `None` ([ADR 020](./adrs/020-brace-blocks.md)).

```flt
let r = {
  let a = 1
  a + 1
}
r                    # 2
{ let a = 1; a * 2 } # 2
```

A `{` starts a map if the next token is `}`, or a key followed by `:`. Otherwise it starts a block. As an arm of an `if`, `{}` is an empty block. Everywhere else, `{}` is an empty map:

```flt
if true {}           # None
if true ({})         # {}: parentheses make the arm a plain expression
```

Each brace arm of an `if` is a block. An arm without braces is a plain expression. A block has its own scope. See [block scope](./runtime.md#block-scope).

## If Expressions

An `if` has two forms ([ADR 022](./adrs/022-expanded-if.md)). The **compact form** has braces or one expression per arm. The **expanded form** has statements in each arm and ends with `end`. Both are expressions:

```flt
if ready { "yes" } else { "no" }
if ready "yes" else "no"

if ready
  let n = 2
  n * 3
else
  0
end
```

`if c { 1 } else { 2 }` and the expanded `if c`, `1`, `else`, `2`, `end` give the same AST. A `{` after the condition starts the compact form. A line break, a comment or a `;` after the condition starts the expanded form, which needs `end`. So `if x; a; else b; end` is valid on one line. An arm with no statements gives `None`, and so does an `if` without `else` when no condition is true.

`elsif` adds more conditions. It is sugar for `else` followed by a nested `if`, and one `end` closes the whole chain:

```flt
if n < 0 { "negative" } elsif n == 0 { "zero" } else { "positive" }

if n < 0
  "negative"
elsif n == 0
  "zero"
else
  "positive"
end
```

A `let` in an arm is gone after the `if`. See [block scope](./runtime.md#block-scope).

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

A declaration has a name, the parameters in parentheses, an optional `-> type`, and a body. In the reference runtime, a `def` is allowed only at the top level, not in a block or in a function body. See [Declaring Functions](./functions-and-pipe.md#declaring-functions).

A **return statement** is `return` with an optional expression. It stops the call of the enclosing function and gives its value. It is allowed only in the body of a `def` and the blocks inside it ([ADR 023](./adrs/023-return-statement.md)):

```flt
def sign(n: Integer) -> Integer
  if n < 0
    return -1
  end
  return 1
end
```

See [Returning Early](./functions-and-pipe.md#returning-early).

- A statement may be followed by an optional `;`.
- If a statement ends on a newline, the semicolon is not required.
- Two statements on the same line require `;` after the first: `let x = 1; let y = 2`.

The REPL parses one statement per line (an expression, a `let` / assignment binding or a `def`). A `def` or an expanded `if` must be written on one line in the REPL, e.g. `def double(n: Number) -> Number n * 2 end` and `if x; a; elsif y; b; else c; end`.

## Reserved Keywords

The following words are reserved and recognized with word boundaries (e.g. `if` is a keyword, but `iffy` is an identifier):

| Keyword  | Keyword | Keyword |
| -------- | ------- | ------- |
| `if`     | `else`  | `elsif` |
| `return` | `and`   | `or`    |
| `not`    | `for`   | `in`    |
| `let`    | `while` | `do`    |
| `fn`     | `def`   | `end`   |

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

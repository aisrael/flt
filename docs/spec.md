# flt Minimal Language Specification (v0.1.2)

`flt` is a small, dynamically typed, expression-oriented language. A program is a sequence of
statements; every statement evaluates to a value.

## 1. Lexical Structure

- **Whitespace / comments**: `#` starts a comment that runs to end of line. Comments may appear
  anywhere whitespace may.
- **Identifiers**: one or more of `A-Z a-z 0-9 _ -` (e.g. `x`, `user_name`, `foo-bar`). Literals win
  over identifiers, so `true`, `false`, `None` and leading-digit forms are not identifiers.
- **Reserved keywords** (matched on word boundaries, so `iffy` is an identifier):
  `if else return and or not for in let while do fn`

## 2. Grammar

```ebnf
program    = { statement [ ";" ] } ;           (* ";" required only between statements on one line *)
statement  = [ "let" ] ident "=" expr          (* binding; "let" is optional *)
           | expr ;
expr       = binary ;
binary     = unary { binop unary } ;          (* precedence table below; all left-assoc *)
unary      = ( "!" | "+" | "-" ) unary         (* no space between operator and operand *)
           | postfix ;
postfix    = primary { "." ident } ;          (* no space around "." *)
primary    = literal | ident | call | if | "(" expr ")" | keyword ;
call       = ident "(" [ args ] ")"
           | ident args ;                      (* whitespace form: at least one argument *)
args       = expr { "," expr } [ "," kwargs ] | kwargs ;
kwargs     = key ":" expr { "," key ":" expr } ;
if         = "if" expr branch [ "else" branch ] ;
branch     = "{" expr "}" | expr ;
key        = ident | string ;
```

## 3. Literals

| Kind      | Examples                                   | Notes                                              |
| --------- | ------------------------------------------ | -------------------------------------------------- |
| Number    | `42` `3.14` `-0.5` `42.`                   | Arbitrary-precision decimal                        |
| String    | `"hi"` `"say \"hi\""` `"Answer: {1 + 2}"` | `{expr}` interpolates; `\{` is a literal `{`       |
| Boolean   | `true` `false`                             |                                                    |
| None      | `None`                                     | The empty `Option`                                 |
| Symbol    | `:id` `:foo-bar` `:"hello world"`          | Bare or quoted                                     |
| Array     | `[]` `[1, 2, 3,]`                          | Trailing comma allowed                             |
| Map       | `{ foo: 1, "spaced out": 2 }`              | Keys are bare identifiers or strings; unique       |

## 4. Types

`Unit` (`()`), `Boolean`, `Number`, `String`, `Symbol`, `Array`, `Map`, `Type` (types are
first-class values), and `Option` (a value or `None`). Keyword arguments are passed as a `Map`.

## 5. Operators (lowest → highest precedence)

| Level | Operators                                  | Meaning                     |
| ----- | ------------------------------------------ | --------------------------- |
| 1     | `\|>`                                      | pipe                        |
| 2     | `\|\|`                                     | logical or                  |
| 3     | `&&`                                       | logical and                 |
| 4     | `^^`                                       | logical xor                 |
| 5–7   | `\|`, `^`, `&`                             | bitwise or / xor / and      |
| 8     | `+` `-` `<>`                               | add, subtract, concatenate  |
| 9     | `*` `/` `==` `!=` `<` `>` `<=` `>=`        | multiply, divide, compare   |
| —     | unary `!` `+` `-`, then postfix `.field`   | bind tightest               |

Comparisons share a level with `*` and `/`, so `1 + 2 == 3` parses as `1 + (2 == 3)`. Use
parentheses: `(1 + 2) == 3`.

## 6. Functions

### Calls

```flt
add(1, 2)            # parenthesized: zero or more arguments
add 1, 2             # whitespace: at least one argument
foo(1, bar: true)    # trailing keyword arguments
```

Positional arguments come first; after the first keyword argument, no positional argument may
follow. Keyword keys follow map-key rules. The parser collects keyword arguments into a single `Map`
passed as the final argument, so `foo(1, bar: true)` receives `1, { bar: true }`.

### Pipe

`a |> b |> c` is a left-associative chain at the lowest precedence, e.g.
`READ("input") |> SELECT(:id) |> WRITE("output")`. It is parsed but not yet evaluated; evaluating it
raises `Function calls not yet supported`.

### Built-ins

The runtime registers one built-in, `typeof(v)`, which returns `v`'s `Type` (`typeof(42)` is
`Number`). The wrong arity raises `Function typeof expected 1 argument(s), found N`; `typeof(None)` is
not yet implemented. Calling any other name raises `Function calls not yet supported`.

### Definitions

A function is identified by its name and one or more signatures (overloads).

A signature has positional and keyword parameters and an optional return type.

Each parameter has a name, an optional type, and an optional default value. Undeclared keyword arguments can be captured by a final `**args` parameter.

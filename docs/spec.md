# flt Minimal Language Specification (v0.2.0)

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
| None      | `None`                                     | No value; also its own type                        |
| Symbol    | `:id` `:foo-bar` `:"hello world"`          | Bare or quoted                                     |
| Array     | `[]` `[1, 2, 3,]`                          | Trailing comma allowed                             |
| Map       | `{ foo: 1, "spaced out": 2 }`              | Keys are bare identifiers or strings; unique       |

## 4. Types

`None`, `Boolean`, `Number`, `String`, `Symbol`, `Array`, `Map`, and `Type`. `Any` is a placeholder used only in signatures (e.g. `typeof` is `fn(Any) -> Type`); no
expression evaluates to `Any`, and `Any` is not a value. Types are first-class values: a built-in type name evaluates to that type, so
`typeof(1) == Number` is `true`, and a `let` binding shadows it. The `None` value equals the `None` type. Keyword arguments are passed as a `Map`.

A union `A | B` is a value of either type (see [union types](types/union_types.md)). An `if` without
`else` evaluates to `None` when its condition is `false`, so it has type `T | None`.

## 5. Operators (lowest → highest precedence)

| Level | Operators                                  | Meaning                     |
| ----- | ------------------------------------------ | --------------------------- |
| 1     | `\|\|`                                     | logical or                  |
| 2     | `&&`                                       | logical and                 |
| 3     | `^^`                                       | logical xor                 |
| 4     | `==` `!=`                                  | equality                    |
| 5     | `<` `>` `<=` `>=`                          | ordering                    |
| 6–8   | `\|`, `^`, `&`                             | bitwise or / xor / and      |
| 9     | `\|>`                                      | pipe                        |
| 10    | `+` `-` `<>`                               | add, subtract, concatenate  |
| 11    | `*` `/`                                    | multiply, divide            |
| —     | unary `!` `+` `-`, then postfix `.field`   | bind tightest               |

Comparisons bind looser than arithmetic, `<>` and bitwise operators, so `1 + 2 == 3` is `true` and
`x & 1 == 0` is `(x & 1) == 0`. As in Elixir, `|>` binds looser than `+` `-` `<>` and tighter than
everything else, so `x |> f == 3` is `(x |> f) == 3` and `a || b |> f` is `a || (b |> f)`.

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

`x |> f(a, ...)` evaluates as `f(x, a, ...)`, and `x |> f` as `f(x)`: the left side becomes the
**first** argument, as in Elixir. Pipes are left-associative, so `a |> f |> g(1)` is `g(f(a), 1)`,
e.g. `READ("input") |> SELECT(:id) |> WRITE("output")`. `|>` sits at level 9 above.

The left side is evaluated first, then the right side's arguments. The right side must be a call or a
bare name, resolved and arity-checked like any call, with the piped value counted:
`1 |> typeof(2)` raises `Function typeof expected 1 argument(s), found 2`. Anything else raises
`Cannot pipe into <expr>`. Keyword arguments stay last: `x |> f(a, k: v)` passes `x, a, { k: v }`.
So `f 1`, `f(1)`, `1 |> f` and `1 |> f()` are equivalent.

The whitespace call form isn't allowed next to `|>`. The right side is parsed like an `if` condition,
where a name followed by whitespace isn't a call, so `1 |> f 2` is a parse error. On the left, a
whitespace-form call can't take an unparenthesized pipe as an argument, so `f 1 |> g()` (which would
mean `f(1 |> g())`) is a parse error too.

### Built-ins

The runtime registers one built-in, `typeof(v)` of type `fn(Any) -> Type`, which returns `v`'s type (`typeof(42)` is
`Number`, `typeof(None)` is `None`). The wrong arity raises
`Function typeof expected 1 argument(s), found N`.

### Native functions and function values

A host can register native functions in the global scope (see
[native functions](functions-and-pipe.md#native-functions)). A function's name evaluates to a
function value of type `fn(P1, P2) -> R`, displayed as `<fn name>`, which can be bound with `let`
and called through any name it's bound to. Two function values are equal only if they're the same
function.

Identifiers and call names resolve in the order variables, then global functions, then built-in type
names. Calls raise:

- `Undefined function: <name>` when the name isn't bound;
- `<name> is not a function` when the name is bound to a value that isn't a function;
- `Function <name> expected <expected> argument(s), found <found>` when no signature of the
  function takes that many arguments.

## 7. Bindings

A variable is a binding of an identifier: to a value at run time, or to an expression for static
analysis (e.g. type inference). `let x = expr` and `x = expr` both bind `x`.

- **Bindings are global.** There are no scopes, so a binding is visible to every statement after it.
  Once scopes exist, a binding will be limited to its scope.
- **Re-binding replaces the binding, including its type.** After `let x = 1` then `x = "a"`,
  `typeof(x)` is `String`.
- **A binding shadows a built-in type name.** After `let Number = 5`, `Number` is `5`.
- Referencing an identifier with no binding raises `Unbound identifier: <name>`.

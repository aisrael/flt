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
| Map       | `{ foo: 1, "spaced out": 2 }`              | Keys are symbols, bare or quoted; unique           |

## 4. Types

`None`, `Boolean`, `Number`, `String`, `Symbol`, `Array`, `Map`, and `Type`. `Any` is a placeholder used only in signatures (e.g. `typeof` is `fn(Any) -> Type`); no
expression evaluates to `Any`, and `Any` is not a value. Types are first-class values: a built-in type name evaluates to that type, so
`typeof(1) == Number` is `true`. The `None` value equals the `None` type.

A union `A | B` is a value of either type (see [union types](types/union_types.md)). An `if` without
`else` evaluates to `None` when its condition is `false`, so it has type `T | None`.

A runtime may let its host define **native types** (see [native types](native-functions-and-values.md#native-types-and-values)). A
native type's name evaluates to that type. Its values come only from the host's functions: flt source
can bind, pass, compare and print them, but can't create them or look inside them. They print as the
host chooses, and `==` and `!=` compare them as the host defines. Values of different native types
are never equal, and native values can't be ordered.

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
follow. Keyword keys follow map-key rules, so each is a symbol, and a call can't repeat one: a
duplicate keyword is a syntax error.

Keyword arguments are passed by name, separately from positional ones, so `foo(1, bar: true)`
passes `1` positionally and `bar` by keyword, while `foo(1, { bar: true })` passes two positional
arguments. A function declares the keywords it accepts, each with a default that a call may leave
out. A positional argument never fills a keyword. A function can also accept any other keywords,
as rest keywords of a given type. A call's keywords help pick the signature it calls (see
[Type-checked calls](#type-checked-calls)), and a keyword no signature declares raises an
`UnexpectedKeywordArgument` error. Positional arguments are evaluated
left to right, then keyword values left to right. A function
type lists its keywords after its positional parameters, e.g.
`fn(String, greeting: String) -> String`.

A function's last positional parameter may be **variadic**: it takes zero or more positional
arguments, never keywords. Keywords are declared separately, so a function can have both. A
function with `k` positional parameters before the variadic one accepts any call with at least `k`
positional arguments. A function type shows the variadic parameter as `*T`, after the other
positional parameters and before any keywords, e.g. `fn(String, *Any) -> String` or
`fn(Any, *Any, k: Number) -> Any`.

### Type-checked calls

A function may have several signatures, or **overloads**. Every call is checked against them by
type. A signature **matches** a call if:

- it takes that many positional arguments;
- each positional argument, and each value of a variadic parameter, is of its parameter's type;
- each keyword the call passes is one the signature declares, with a value of that keyword's type,
  or is accepted by the signature's rest keywords, with a value of their type. A keyword the call
  leaves out takes its default.

A value is of type `Any` always; of a built-in type if it has that type; of `A | B` if it's of
either; of `Option<T>` if it's `None` or of `T`; of a native type only if it's a value of that same
native type, whatever its name; and of a function type if the function's type, or one of its
signatures' types, is exactly that type.

A signature's **arity** is the number of its positional parameters before any variadic one. The
signatures are tried in order of arity, longest first, and the first that matches is called, so the
order they were declared in doesn't matter. If several signatures of the same arity match, the spec
doesn't say which one is called. The function's result must be of the called signature's return
type.

```
foo(*args: Any) -> String
foo(a: Number, *args: Any) -> String
```

```flt
foo(1, :b)       # the second: arity 1 is tried before arity 0
foo(:b)          # the first
```

### Pipe

`x |> f(a, ...)` evaluates as `f(x, a, ...)`, and `x |> f` as `f(x)`: the left side becomes the
**first** argument, as in Elixir. Pipes are left-associative, so `a |> f |> g(1)` is `g(f(a), 1)`,
e.g. `READ("input") |> SELECT(:id) |> WRITE("output")`. `|>` sits at level 9 above.

The left side is evaluated first, then the right side's arguments. The right side must be a call or a
bare name, resolved and checked like any call, with the piped value as its first argument:
`1 |> typeof(2)` raises an `ArgumentCountMismatch` error. Anything else raises an
`InvalidPipeTarget` error. The piped value is positional: `x |> f(a, k: v)` passes `x` and `a`
positionally and `k` by keyword.
So `f 1`, `f(1)`, `1 |> f` and `1 |> f()` are equivalent.

The whitespace call form isn't allowed next to `|>`. The right side is parsed like an `if` condition,
where a name followed by whitespace isn't a call, so `1 |> f 2` is a parse error. On the left, a
whitespace-form call can't take an unparenthesized pipe as an argument, so `f 1 |> g()` (which would
mean `f(1 |> g())`) is a parse error too.

### Built-ins

A runtime may register any number of built-in, native functions. The reference implementation
(`SimpleRuntime`) registers only `typeof(Any) -> Type`, which returns its argument's type.

### Native functions and function values

A host can register native functions in the global scope (see
[native functions](native-functions-and-values.md#native-functions)). A function's name evaluates to a
function value of type `fn(P1, P2) -> R`, displayed as `<fn name>`, which can be bound with `let`
and called through any name it's bound to. Two function values are equal only if they're the same
function. A function type can name native types (§4), e.g. `fn(Date, Date) -> Number`. A native
function can fail with the host's own error, which is reported unchanged.

Identifiers and call names resolve in the order variables, then global functions, then built-in type
names, then native type names. Calls raise:

- an `UndefinedFunction` error when the name isn't bound;
- a `NotCallable` error when the name is bound to a value that isn't a function;
- an `ArgumentCountMismatch` error when no signature of the function takes that many positional
  arguments;
- when one signature takes that many positional arguments but doesn't match, an error for the first
  thing that fails, checking positional arguments first, then keywords in call order:
  - an `ArgumentTypeMismatch` error for a positional argument of the wrong type;
  - an `UnexpectedKeywordArgument` error for a keyword the signature doesn't accept;
  - a `KeywordTypeMismatch` error for a keyword value of the wrong type;
- when several signatures take that many positional arguments and none matches, the error for the
  closest one, as if it were the only one. The closest is the signature that passes the most
  checks, in the order above, before one fails; if several first fail at the same keyword, one that accepts the
  keyword but not its value is closer than one that doesn't accept it;
- a `NoMatchingOverload` error when several signatures are equally close;
- a `ReturnTypeMismatch` error when the result isn't of the called signature's return type.

## 7. Bindings

A variable is a binding of an identifier: to a value at run time, or to an expression for static
analysis (e.g. type inference). `let x = expr` and `x = expr` both bind `x`.

- **Bindings are global.** There are no scopes, so a binding is visible to every statement after it.
  Once scopes exist, a binding will be limited to its scope.
- **Re-binding replaces the binding, including its type.** After `let x = 1` then `x = "a"`,
  `typeof(x)` is `String`.
- Referencing an identifier with no binding raises an `UnboundIdentifier` error.

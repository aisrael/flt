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
statement  = "let" ident [ ":" type ] "=" expr  (* declaration *)
           | ident "=" expr                       (* assignment *)
           | expr ;
type       = type_atom { "|" type_atom } ;
type_atom  = ident | "None" | "(" type ")" ;
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
| Integer   | `42` `-7` `+7` `1_000`                     | Arbitrary-precision whole number; no `.`           |
| Number    | `3.14` `-0.5` `42.0` `42.`                 | Arbitrary-precision decimal; has a `.`             |
| String    | `"hi"` `"say \"hi\""` `"Answer: {1 + 2}"` | `{expr}` interpolates; `\{` is a literal `{`       |
| Boolean   | `true` `false`                             |                                                    |
| None      | `None`                                     | No value; also its own type                        |
| Symbol    | `:id` `:foo-bar` `:"hello world"`          | Bare or quoted                                     |
| Array     | `[]` `[1, 2, 3,]`                          | Trailing comma allowed                             |
| Map       | `{ foo: 1, "spaced out": 2 }`              | Keys are symbols, bare or quoted; unique           |

## 4. Types

`None`, `Boolean`, `Integer`, `Number`, `String`, `Symbol`, `Array`, `Map`, and `Type`. `Any` is a placeholder used only in signatures (e.g. `typeof` is `fn(Any) -> Type`); no
expression evaluates to `Any`, and `Any` is not a value. Types are first-class values: a built-in type name evaluates to that type, so
`typeof(1) == Integer` is `true`. The `None` value equals the `None` type.

`Integer` and `Number` are both arbitrary-precision. An `Integer` is a whole number, and a `Number`
a decimal. A `Number` with no digits after its decimal point prints with a trailing `.0`, e.g.
`2.0`, so a printed `Number` never looks like an `Integer`. Wherever a value is checked against a
type (§6, §7), `Number` accepts an `Integer` too, and the `Integer` is converted to a `Number`.
This is the only implicit conversion between types. `Integer` doesn't accept a `Number`, even a
whole one such as `1.0`.

A union `A | B` is a value of either type (see [union types](types/union_types.md)). An `if` without
`else` evaluates to `None` when its condition is `false`, so it has type `T | None`.

flt source writes a type only in a `let` declaration (§7). A type there is a built-in type name
(including `Any` and `None`), a native type's name, a union `A | B` of types, or a type in
parentheses.

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
| 11    | `*` `/` `//`                               | multiply, divide, floor-divide |
| —     | unary `!` `+` `-`, then postfix `.field`   | bind tightest               |

Comparisons bind looser than arithmetic, `<>` and bitwise operators, so `1 + 2 == 3` is `true` and
`x & 1 == 0` is `(x & 1) == 0`. As in Elixir, `|>` binds looser than `+` `-` `<>` and tighter than
everything else, so `x |> f == 3` is `(x |> f) == 3` and `a || b |> f` is `a || (b |> f)`.

### Arithmetic

| Operation      | `Integer`, `Integer` | `Integer` and `Number`, either way round | `Number`, `Number` |
| -------------- | -------------------- | ---------------------------------------- | ------------------ |
| `+` `-` `*`    | `Integer`            | `Number`                                 | `Number`           |
| `/`            | `Integer`, floored   | `Number`                                 | `Number`           |
| `//`           | `Integer`, floored   | `InvalidOperandType` error               | `InvalidOperandType` error |
| unary `+` `-`  | `Integer`            |                                          | `Number`           |

- **Floored:** `/` and `//` on two integers round the quotient toward negative infinity, so `7 / 2`
  is `3`, `-7 / 2` is `-4` and `7 // -2` is `-4`. Unary `-` binds tighter, so `-7 / 2` is
  `(-7) / 2`. `7.0 / 2` is `3.5`.
- **`//` takes only integers.** Any other operand raises an `InvalidOperandType` error.
- **Dividing by zero** raises a `DivisionByZero` error, for `/` with a divisor of `0` or `0.0`,
  and for `//` with a divisor of `0` once its operands' types are checked.
- **Comparisons are by value.** `==`, `!=`, `<`, `>`, `<=` and `>=` compare an `Integer` with a
  `Number` by value, so `1 == 1.0` and `1 < 1.5` are `true`. Equality compares the elements of
  arrays and the values of maps the same way, so `[1] == [1.0]` is `true`.
- **Integers don't overflow.** `9223372036854775807 + 1` is `9223372036854775808`.
- Any other operand to an arithmetic operator, such as `1 + true`, raises an `InvalidOperandType`
  error.

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
`fn(Any, *Any, k: Number) -> Any`. A union after `*` or `**` is bracketed, e.g.
`fn(*(String | Symbol)) -> String`, as is a union return type, e.g. `fn(T) -> (T | None)`.

### Type-checked calls

A function may have several signatures, or **overloads**. Every call is checked against them by
type. A signature **matches** a call if:

- it takes that many positional arguments;
- each positional argument, and each value of a variadic parameter, is of its parameter's type;
- each keyword the call passes is one the signature declares, with a value of that keyword's type,
  or is accepted by the signature's rest keywords, with a value of their type. A keyword the call
  leaves out takes its default.

A value is of type `Any` always; of a built-in type if it has that type; of `Number` if it's a
`Number` or an `Integer`; of `A | B` if it's of either; of `Option<T>` if it's `None` or of `T`; of
a native type only if it's a value of that same native type, whatever its name; and of a function
type if the function's type, or one of its signatures' types, is exactly that type.

An `Integer` that's of a type only because `Number` accepts it is converted to a `Number`: an
argument or keyword value before the function is called, and a result before it's returned. A union
or `Option` converts it only if none of its members accepts it as it is, so `Integer | Number` and
`Any` leave it an `Integer`, while `Number | String` converts it. The signature that's called is
chosen first, and its types decide the conversion.

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
analysis (e.g. type inference).

- **`let` declares a variable.** `let x = expr` declares `x` without a type, and
  `let x: T = expr` declares it with the type `T`. The value must be of type `T`, by the same rules
  as a function argument (§6), or the declaration raises a `VariableTypeMismatch` error. It's
  converted the same way too, so `let n: Number = 1` binds `1.0`. A declaration replaces any
  earlier declaration of `x`, including its type.
- **`x = expr` assigns.** If `x` was declared with a type, the value must be of that type, or the
  assignment raises a `VariableTypeMismatch` error and `x` keeps its value, and it's converted as
  in a declaration. Otherwise the value replaces the binding, including its type: after
  `let x = 1` then `x = "a"`, `typeof(x)` is `String`. Assigning to a name with no binding declares
  it without a type.
- **The name and the type are checked before the value is evaluated.** A type name the runtime
  doesn't know raises an `UnknownType` error.
- **A binding evaluates to the bound value.** `typeof` reports the value's type, not the declared
  one.
- **Bindings are global.** There are no scopes, so a binding is visible to every statement after it.
  Once scopes exist, a binding will be limited to its scope.
- Referencing an identifier with no binding raises an `UnboundIdentifier` error.

```flt
let n: Number = 1         # n is 1.0
n = 2                     # ok: n is 2.0
n = "two"                 # VariableTypeMismatch
let i: Integer = 1.0      # VariableTypeMismatch
let n = "two"             # ok: declares n again, without a type
let x: Number | None = None
let y: Foo = 1            # UnknownType
```

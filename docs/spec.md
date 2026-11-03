# flt Minimal Language Specification (v0.2.0)

`flt` is a small, dynamically typed, expression-oriented language. A program is a sequence of statements. Each statement gives a value.

## 1. Lexical Structure

- **Whitespace and comments**: `#` starts a comment. The comment continues to the end of the line. A comment can occur where whitespace can occur.
- **Identifiers**: one or more of `A-Z a-z 0-9 _ -` (for example `x`, `user_name`, `foo-bar`). Literals have priority over identifiers. Thus `true`, `false`, `None` and names that start with a digit are not identifiers.
- **Reserved keywords** (matched on word boundaries, so `iffy` is an identifier): `if else return and or not for in let while do fn def end`

## 2. Grammar

```ebnf
program    = { statement [ ";" ] } ;           (* ";" required only between statements on one line *)
statement  = "let" ident [ ":" type ] "=" expr  (* declaration *)
           | ident "=" expr                       (* assignment *)
           | "def" ident "(" [ params ] ")" [ "->" type ] body  (* function declaration *)
           | expr ;
params     = param { "," param } ;
param      = ident ":" type                       (* positional *)
           | "*" ident ":" type                   (* variadic *)
           | ident ":" type "=" expr              (* keyword, with default *)
           | "**" ident ":" type ;                (* rest keywords *)
body       = { statement [ ";" ] } "end" ;
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

| Kind | Examples | Notes |
| --- | --- | --- |
| Integer | `42` `-7` `+7` `1_000` | Arbitrary-precision whole number; no `.` |
| Number | `3.14` `-0.5` `42.0` | Arbitrary-precision decimal; digits after a `.` |
| String | `"hi"` `"say \"hi\""` `"Answer: {1 + 2}"` | `{expr}` interpolates; `\{` is a literal `{` |
| Boolean | `true` `false` |  |
| None | `None` | No value; also its own type |
| Symbol | `:id` `:foo-bar` `:"hello world"` | Bare or quoted |
| Array | `[]` `[1, 2, 3,]` | Trailing comma allowed |
| Map | `{ foo: 1, "spaced out": 2 }` | Keys are symbols, bare or quoted; unique |

## 4. Types

The types are `None`, `Boolean`, `Integer`, `Number`, `String`, `Symbol`, `Array`, `Map` and `Type`. `Any` is a placeholder. It occurs only in signatures (for example, `typeof` is `fn(Any) -> Type`). No expression gives `Any`, and `Any` is not a value.

Types are first-class values. A built-in type name gives that type. Thus `typeof(1) == Integer` is `true`. The `None` value is equal to the `None` type.

`Integer` and `Number` both have arbitrary precision. An `Integer` is a whole number. A `Number` is a decimal. A `Number` with no digits after its decimal point prints with a trailing `.0`, for example `2.0`. Thus a printed `Number` never looks like an `Integer`.

Some places check a value against a type (§6, §7). In these places, `Number` also accepts an `Integer`, and the runtime converts the `Integer` to a `Number`. This is the only implicit conversion between types. `Integer` does not accept a `Number`, even a whole one such as `1.0`.

A union `A | B` is a value of either type (see [union types](types/union_types.md)). An `if` without `else` gives `None` when its condition is `false`. Thus it has type `T | None`.

flt source writes a type only in a `let` declaration (§7) and in a `def` declaration (§6). A type in these declarations is one of these:

- a built-in type name (including `Any` and `None`);
- the name of a native type;
- a union `A | B` of types;
- a type in parentheses.

A runtime can let its host define **native types** (see [native types](native-functions-and-values.md#native-types-and-values)). The name of a native type gives that type. Its values come only from the functions of the host. flt source can bind, pass, compare and print native values. flt source cannot create them or look inside them. The host decides how they print. The host also decides how `==` and `!=` compare them. Values of different native types are never equal. Native values cannot be ordered.

## 5. Operators (lowest → highest precedence)

| Level | Operators | Meaning |
| --- | --- | --- |
| 1 | `\|\|` | logical or |
| 2 | `&&` | logical and |
| 3 | `^^` | logical xor |
| 4 | `==` `!=` | equality |
| 5 | `<` `>` `<=` `>=` | ordering |
| 6–8 | `\|`, `^`, `&` | bitwise or / xor / and |
| 9 | `\|>` | pipe |
| 10 | `+` `-` `<>` | add, subtract, concatenate |
| 11 | `*` `/` `//` | multiply, divide, floor-divide |
| — | unary `!` `+` `-`, then postfix `.field` | bind tightest |

Comparisons bind less tightly than arithmetic, `<>` and bitwise operators. Thus `1 + 2 == 3` is `true`, and `x & 1 == 0` is `(x & 1) ==  0`.

As in Elixir, `|>` binds less tightly than `+`, `-` and `<>`. It binds more tightly than all other operators. Thus `x |> f == 3` is `(x |> f) == 3`, and `a || b |> f` is `a || (b |> f)`.

### Arithmetic

| Operation | `Integer`, `Integer` | `Integer` and `Number`, either way round | `Number`, `Number` |
| --- | --- | --- | --- |
| `+` `-` `*` | `Integer` | `Number` | `Number` |
| `/` | `Integer`, floored | `Number` | `Number` |
| `//` | `Integer`, floored | `InvalidOperandType` error | `InvalidOperandType` error |
| unary `+` `-` | `Integer` |  | `Number` |

- **Floored:** `/` and `//` on two integers round the quotient toward negative infinity. Thus `7 / 2` is `3`, `-7 / 2` is `-4` and `7 // -2` is `-4`. Unary `-` binds more tightly, so `-7 / 2` is `(-7) / 2`. `7.0 / 2` is `3.5`.
- **`//` takes only integers.** Any other operand raises an `InvalidOperandType` error.
- **Division by zero** raises a `DivisionByZero` error. This applies to `/` with a divisor of `0` or `0.0`. It also applies to `//` with a divisor of `0`, after the runtime checks the types of the operands.
- **Comparisons are by value.** The operators `==`, `!=`, `<`, `>`, `<=` and `>=` compare an `Integer` with a `Number` by value. Thus `1 == 1.0` and `1 < 1.5` are `true`. Equality compares the elements of arrays and the values of maps in the same way. Thus `[1] == [1.0]` is `true`.
- **Integers do not overflow.** `9223372036854775807 + 1` is `9223372036854775808`.
- Any other operand to an arithmetic operator raises an `InvalidOperandType` error. An example is `1 + true`.

## 6. Functions

Functions are the building blocks of `flt`. A function is a self-contained, reusable block of code. Instead of writing the same statements over and over, declare a function once and call it multiple times.

Functions can accept _parameters_. Values passed when calling a function are its _arguments_. A function may also return a value. The type of that value is the function's _return type_.

### Function parameters

A function has three kinds of parameter: **positional**, **keyword** and **variadic**. A call can pass positional arguments, variadic arguments, then keyword arguments (see [Calls](#calls)).

A function _signature_ is comprised of:

- its _name_
- the _parameters_ it can accept, and
- its _return type_

A function declaration starts with the `def` keyword, followed by the function _name_, then the parameters enclosed in parentheses (`()`), then an optional return type, then the body. The body ends with `end`.

> `def` _name_ `(` [ _parameter spec_ [`,` ...] ] `)` [ `->` _type_ ] _body_ `end`

A function can have no parameters.

```flt
def flip_a_coin() -> Boolean
  true
end
```

A function can have no return value. Without a return type, the return type is `None`.

```flt
def say_hi(name: String) # implicit -> `None`
  None
end
```

A function's `type` is its signature without its name or parameter names.

```flt
typeof(flip_a_coin) # => fn() -> Boolean
typeof(say_hi)      # => fn(String) -> None
```

## Declaring a function

The parameters of a declaration come in this order:

1. positional parameters;
2. at most one variadic parameter, `*name: T`;
3. keyword parameters, `name: T = default`;
4. at most one rest keywords parameter, `**name: T`.

A syntax error occurs when a declaration has its parameters in another order. A syntax error also occurs when a parameter name occurs twice in one declaration.

Every parameter declares its type. Use `Any` to accept every value. The runtime checks each argument against this type, as §6 describes for calls.

Every keyword parameter has a default expression. A call that omits the keyword evaluates the default expression for that call. A default can use global bindings, but not other parameters. The runtime checks the value of the default against the type of the keyword.

The body is a sequence of statements. The value of the function is the value of the last statement. An empty body gives `None`. The result must be of the return type. If it is not, the call raises a `ReturnTypeMismatch` error.

```flt
def greet(name: String, greeting: String = "Hello") -> String
  greeting <> ", " <> name
end

def log(label: String, *values: Any, level: Symbol = :info)
  None
end
```

A call to a function creates a scope for its body:

- Each parameter is bound in this scope, with its declared type. A variadic parameter is bound to an `Array`. A rest keywords parameter is bound to a `Map`.
- A `let` in the body declares a variable in this scope only. The variable is gone when the call ends.
- A reference to a name that this scope does not bind looks in the global scope.
- An assignment `x = expr` assigns to the nearest binding of `x`. If there is none, it declares `x` in the scope of the call.

A `def` statement binds its name in the global scope to a function value, and gives `None`. The name is bound before the body runs, so a function can call itself.

A second `def` with the same name adds an overload. It raises a `DuplicateOverload` error if its parameters have the same types as an existing overload (see [Type-checked calls](#type-checked-calls)). A `def` raises a `NameAlreadyBound` error if its name is a built-in type or a variable. It also raises this error if the name is a function that a host registered. A reserved keyword cannot be a function name. This is a syntax error.

#### Positional parameters

A positional parameter gets its value from the position of the argument in the call. The first argument fills the first parameter, the second argument fills the second parameter, and so on. A call must pass one argument for each positional parameter. Each argument must match the type of its parameter.

```flt
def mod(n: Integer, d: Integer) -> Integer
  n - (n // d) * d
end

mod(3, 2)        # 1, n is 3, d is 2
mod 3, 2         # the same call in whitespace form
mod(2, 3)        # 2, the order of the arguments matters
```

Wrong calls raise errors:

```flt
mod(1)           # ArgumentCountMismatch: one argument is missing
mod(1, 2, 3)     # ArgumentCountMismatch: one argument is extra
mod(1, "two")    # ArgumentTypeMismatch: "two" is not an Integer
```

#### Keyword parameters

A keyword parameter gets its value from a name in the call. Each keyword parameter has a default value, so a call can omit it. The order of keyword arguments does not matter. A positional argument never fills a keyword parameter. The signature of a function lists its keyword parameters after the positional parameters.

```text
greet: fn(String, greeting: String) -> String    # greeting defaults to "Hello"
```

```flt
greet("Ann")                       # "Hello, Ann"
greet("Ann", greeting: "Hi")       # "Hi, Ann"
greet(greeting: "Hi", "Ann")       # syntax error: a positional argument cannot follow a keyword
```

Wrong calls raise errors:

```flt
greet("Ann", greeting: 1)          # KeywordTypeMismatch: 1 is not a String
greet("Ann", tone: "loud")         # UnexpectedKeywordArgument: greet does not declare tone
greet("Ann", "Hi")                 # ArgumentCountMismatch: "Hi" does not fill greeting
```

A function can also accept other keywords. These are rest keywords, and they have one type. The signature shows them as `**T`.

```text
options: fn(**Any) -> Map
```

```flt
options()                          # {}
options(color: :red, size: 3)      # { color: :red, size: 3 }
```

#### Variadic parameters

A variadic parameter takes zero or more positional arguments. It must be the last positional parameter of the function. It never takes keywords. Each value must be of the type of the parameter. A function with `k` positional parameters before the variadic one accepts any call with at least `k` positional arguments. The signature shows the variadic parameter as `*T`.

```text
sum: fn(*Number) -> Number
join: fn(String, *Any) -> String            # one separator, then any values
log: fn(String, *Any, level: Symbol) -> None   # a variadic parameter and a keyword
```

```flt
sum()                              # 0.0: no values
sum(1, 2, 3)                       # 6.0
join(",", 1, :a, "b")              # "1,:a,b"
log("started", 1, 2, level: :info) # a call with both kinds of argument
```

Wrong calls raise errors:

```flt
join()                             # ArgumentCountMismatch: the separator is required
sum(1, "two")                      # ArgumentTypeMismatch: "two" is not a Number
sum(1, total: 2)                   # UnexpectedKeywordArgument: sum declares no keywords
```

### Calls

```flt
add(1, 2)            # parenthesized: zero or more arguments
add 1, 2             # whitespace: at least one argument
foo(1, bar: true)    # trailing keyword arguments
```

Positional arguments come first. After the first keyword argument, no positional argument can follow. Keyword keys follow the rules for map keys, so each key is a symbol. A call cannot repeat a key. A duplicate keyword is a syntax error.

Keyword arguments are passed by name, separately from positional arguments. Thus `foo(1, bar: true)` passes `1` by position and `bar` by keyword. `foo(1, { bar: true })` passes two positional arguments.

The keywords of a call help to select the signature to call (see [Type-checked calls](#type-checked-calls)). If no signature declares a keyword, the call raises an `UnexpectedKeywordArgument` error.

The runtime evaluates positional arguments from left to right. Then it evaluates keyword values from left to right.

### Function types

A function type lists its keywords after its positional parameters, for example `fn(String, greeting: String) -> String`.

A function type shows the variadic parameter as `*T`. It comes after the other positional parameters and before any keywords, for example `fn(String, *Any) -> String` or `fn(Any, *Any, k: Number) -> Any`. A union after `*` or `**` is in parentheses, for example `fn(*(String | Symbol)) -> String`. A union return type is also in parentheses, for example `fn(T) -> (T | None)`.

### Type-checked calls

A function can have several signatures, or **overloads**. The runtime checks each call against the signatures by type. A signature **matches** a call if all of these are true:

- It takes that number of positional arguments.
- Each positional argument, and each value of a variadic parameter, is of the type of its parameter.
- Each keyword in the call is declared by the signature, with a value of the type of that keyword. Or the rest keywords of the signature accept it, with a value of their type. A keyword that the call omits takes its default.

A value is of a type in these cases:

- Every value is of type `Any`.
- A value is of a built-in type if it has that type.
- A value is of type `Number` if it is a `Number` or an `Integer`.
- A value is of type `A | B` if it is of either type.
- A value is of type `Option<T>` if it is `None` or of type `T`.
- A value is of a native type only if it is a value of that same native type, whatever its name.
- A value is of a function type if the type of the function, or the type of one of its signatures, is exactly that type.

Sometimes an `Integer` is of a type only because `Number` accepts it. Then the runtime converts the `Integer` to a `Number`. It converts an argument or a keyword value before it calls the function. It converts a result before it returns the result.

A union or `Option` converts the `Integer` only if none of its members accepts the `Integer` as it is. Thus `Integer | Number` and `Any` leave it an `Integer`. `Number | String` converts it. The runtime first chooses the signature to call. The types of that signature decide the conversion.

The **arity** of a signature is the number of its positional parameters before any variadic one. If several signatures match a call, which one the runtime calls is unspecified and left to the runtime. The result of the function must be of the return type of the called signature.

```
foo(*args: Any) -> String
foo(a: Number, *args: Any) -> String
```

```flt
foo(1, :b)       # the second: arity 1 is tried before arity 0
foo(:b)          # the first
```

### Pipe

`x |> f(a, ...)` evaluates as `f(x, a, ...)`. `x |> f` evaluates as `f(x)`. The left side becomes the **first** argument, as in Elixir. Pipes are left-associative. Thus `a |> f |> g(1)` is `g(f(a), 1)`. For example: `READ("input") |> SELECT(:id) |> WRITE("output")`. `|>` has level 9 in the table above.

The runtime evaluates the left side first. Then it evaluates the arguments of the right side. The right side must be a call or a bare name. The runtime resolves and checks it like any call, with the piped value as its first argument. For example, `1 |> typeof(2)` raises an `ArgumentCountMismatch` error. Any other right side raises an `InvalidPipeTarget` error.

The piped value is positional. `x |> f(a, k: v)` passes `x` and `a` by position and `k` by keyword. Thus `f 1`, `f(1)`, `1 |> f` and `1 |> f()` are equivalent.

The whitespace call form is not allowed next to `|>`. The parser reads the right side like an `if` condition. In an `if` condition, a name followed by whitespace is not a call. Thus `1 |> f 2` is a parse error.

On the left side, a whitespace-form call cannot take an unparenthesized pipe as an argument. Thus `f 1 |> g()` (which would mean `f(1 |> g())`) is also a parse error.

### Built-ins

A runtime can register any number of built-in, native functions. The reference implementation registers only `typeof(Any) -> Type`. It returns the type of its argument.

### Native functions and function values

A host can register native functions in the global scope (see [native functions](native-functions-and-values.md#native-functions)). The name of a function gives a function value of type `fn(P1, P2) -> R`. It displays as `<fn name>`. You can bind it with `let`. You can call it through any name that is bound to it.

Two function values are equal only if they are the same function. A function type can name native types (§4), for example `fn(Date, Date) -> Number`. A native function can fail with the host's own error. The runtime reports this error without change.

The runtime resolves identifiers and call names in this order:

1. variables
2. global functions
3. built-in type names
4. native type names

A call raises one of these errors:

- an `UndefinedFunction` error when the name is not bound;
- a `NotCallable` error when the name is bound to a value that is not a function;
- an `ArgumentCountMismatch` error when no signature of the function takes that number of positional arguments;
- when one signature takes that number of positional arguments but does not match, an error for the first check that fails. The runtime checks positional arguments first. Then it checks keywords in the order of the call:
  - an `ArgumentTypeMismatch` error for a positional argument of the wrong type;
  - an `UnexpectedKeywordArgument` error for a keyword that the signature does not accept;
  - a `KeywordTypeMismatch` error for a keyword value of the wrong type;
- when several signatures take that number of positional arguments and none matches, the error for the closest signature, as if it were the only one;
  - The closest signature is the one that passes the most checks, in the order above, before one fails.
  - If several signatures first fail at the same keyword, a signature that accepts the keyword but not its value is closer than one that does not accept it;
- a `NoMatchingOverload` error when several signatures are equally close;
- a `ReturnTypeMismatch` error when the result is not of the return type of the called signature.

## 7. Bindings

A variable is a binding of an identifier. At run time, it binds the identifier to a value. For static analysis (for example, type inference), it binds the identifier to an expression.

- **`let` declares a variable.** `let x = expr` declares `x` without a type. `let x: T = expr` declares `x` with the type `T`. The value must be of type `T`, by the same rules as for a function argument (§6). If it is not, the declaration raises a `VariableTypeMismatch` error. The runtime also converts the value in the same way. Thus `let n: Number = 1` binds `1.0`. A declaration replaces any earlier declaration of `x`, including its type.
- **`x = expr` assigns.** If `x` was declared with a type, the value must be of that type. If it is not, the assignment raises a `VariableTypeMismatch` error, and `x` keeps its value. The runtime converts the value as in a declaration. If `x` has no declared type, the value replaces the binding, including its type. After `let x = 1` and then `x = "a"`, `typeof(x)` is `String`. An assignment to a name with no binding declares the name without a type.
- **The runtime checks the name and the type before it evaluates the value.** A type name that the runtime does not know raises an `UnknownType` error.
- **A binding evaluates to the bound value.** `typeof` reports the type of the value, not the declared type.
- **Bindings are global, except in a function call.** A binding outside a function body is visible to every statement after it. The only other scope is the scope of a function call (§6). A binding in that scope is limited to the call.
- A reference to an identifier with no binding raises an `UnboundIdentifier` error.

```flt
let n: Number = 1         # n is 1.0
n = 2                     # ok: n is 2.0
n = "two"                 # VariableTypeMismatch
let i: Integer = 1.0      # VariableTypeMismatch
let n = "two"             # ok: declares n again, without a type
let x: Number | None = None
let y: Foo = 1            # UnknownType
```

# flt Minimal Language Specification (v0.2.0)

`flt` is a small, dynamically typed, functional language. A program is a sequence of statements. Each statement gives a value.

This page describes the syntax of flt and the semantics that every runtime must give. A runtime decides everything else. For example, it decides which functions it provides, how it supports overloads, and how values print. The behavior of the reference runtime is in [`runtime.md`](runtime.md). This page names the errors that a runtime raises. Each name links to [`errors.md`](errors.md), which gives the message and the cause in the reference runtime.

## 1. Lexical Structure

- **Whitespace and comments**: `#` starts a comment. The comment continues to the end of the line. A comment can occur where whitespace can occur.
- **Identifiers**: one or more of `A-Z a-z 0-9 _ -` (for example `x`, `user_name`, `foo-bar`). Literals have priority over identifiers. Thus `true`, `false`, `None` and names that start with a digit are not identifiers.
- **Reserved keywords** (matched on word boundaries, so `iffy` is an identifier): `if else elsif return and or not for in let const while do fn def end`

## 2. Grammar

```ebnf
program    = { statement [ ";" ] } ;           (* ";" required only between statements on one line *)
statement  = "let" ident [ ":" type ] "=" expr  (* declaration *)
           | "const" ident [ ":" type ] "=" expr  (* constant declaration *)
           | ident "=" expr                       (* assignment *)
           | "def" ident "(" [ params ] ")" [ "->" type ] body  (* function declaration *)
           | "return" [ expr ]                    (* only in a function body *)
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
primary    = literal | ident | call | if | block | "(" expr ")" | keyword ;
call       = ident "(" [ args ] ")"
           | ident args ;                      (* whitespace form: at least one argument *)
args       = expr { "," expr } [ "," kwargs ] | kwargs ;
kwargs     = key ":" expr { "," key ":" expr } ;
if         = "if" expr ( compact | expanded ) ;
compact    = branch { "elsif" expr branch } [ "else" branch ] ;
expanded   = sep statements { "elsif" expr sep statements } [ "else" statements ] "end" ;
branch     = block | expr ;
sep        = NEWLINE | ";" ;                  (* a comment can come before the line break *)
statements = { statement [ ";" ] } ;
block      = "{" { statement [ ";" ] } "}" ;
key        = ident | string ;
```

### Blocks

A **brace block** starts with `{` and ends with `}`. It holds statements, and it is an expression ([ADR 020](adrs/020-brace-blocks.md)). It evaluates to the value of its last statement. A block with no statements evaluates to `None`. As at the top level, a line break or a `;` ends a statement.

```flt
{ 1 }                          # 1
{ let a = 1; a * 2 }           # 2
{
  let a = 1
  a + 1
}                              # 2
```

A `{` starts a map if the next token is `}`, or a key followed by `:`. Otherwise it starts a block. Thus `{ a: 1 }` is a map, and `{ a }` is a block. For `{}` the context decides: as an arm of an `if`, `{}` is an empty block, which evaluates to `None`. Anywhere else, `{}` is an empty map. An arm in parentheses is an expression, so `if true ({})` gives an empty map.

Each brace arm of an `if` is a block. Thus `if c { 1 } else { 2 }` has two blocks, and `if true {}` has an empty block. An arm without braces (`if c 1 else 2`) is a plain expression.

The language does not say which names a block can see or declare. A runtime decides (see [`runtime.md`](runtime.md#block-scope)).

### The `if` expression

An `if` has two forms, and both are expressions ([ADR 022](adrs/022-expanded-if.md)). The **compact form** has one arm in braces or one expression per arm. The **expanded form** has statements in each arm and ends with `end`:

```flt
if ready { "yes" } else { "no" }

if ready
  let n = 2
  n * 3
else
  0
end
```

An `if` evaluates to the value of the last statement of the arm that ran. An `if` without `else` evaluates to `None` when no condition is true. An arm with no statements is an empty block, and it evaluates to `None`.

The form is chosen by the first token after the condition:

- A `{` on the same line means the compact form with a brace block.
- A line break, a comment, or a `;` before the first token of the arm means the expanded form. It must end with `end`.
- Anything else means the compact form with a one-expression arm.

A `;` is the same as a line break here, so `if x; a; else b; end` is valid. The first statement after `else` can follow it on the same line. A list of statements ends at `else`, `elsif` or `end`. The forms do not mix: an expanded `if` always ends with `end`, and a compact `if` never does. Both forms of `if c { 1 } else { 2 }` give the same AST.

#### `elsif`

`elsif` is sugar for an `else` arm that holds one nested `if`. Thus these two are the same:

```flt
if a { 1 } elsif b { 2 } else { 3 }
if a { 1 } else { if b { 2 } else { 3 } }
```

An `elsif` has a condition and an arm in the form of the `if` it belongs to. There can be any number of `elsif` arms, and they come before `else`. The conditions are tested in order. The first true condition runs its arm, and the later conditions are not evaluated. In the expanded form, one `end` closes the whole chain:

```flt
if a
  "first"
elsif b
  "second"
else
  "third"
end
```

A written-out `else` followed by `if` needs its own `end` for the inner `if`. `elsif` does not.

The keywords `else`, `elsif` and `end` never start an argument of a call in the whitespace form.

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

The built-in types are `None`, `Boolean`, `Integer`, `Number`, `String`, `Symbol`, `Array`, `Map` and `Type`. `Any` is a placeholder that accepts every value. It occurs only in signatures. No expression gives `Any`, and `Any` is not a value. A runtime can define more types.

Types are first-class values. A built-in type name gives that type. The `None` value is equal to the `None` type.

`Integer` and `Number` both have arbitrary precision. An `Integer` is a whole number. A `Number` is a decimal.

Some places check a value against a type (§6, §7). In these places, `Number` also accepts an `Integer`, and the runtime converts the `Integer` to a `Number`. This is the only implicit conversion between types. `Integer` does not accept a `Number`, even a whole one such as `1.0`.

A union `A | B` is a value of either type (see [union types](types/union_types.md)). An `if` without `else` gives `None` when its condition is `false`. Thus it has type `T | None`.

flt source writes a type only in a `let` or `const` declaration (§7) and in a `def` declaration (§6). A type in these declarations is one of these:

- a built-in type name (including `Any` and `None`);
- the name of a type that the runtime defines;
- a union `A | B` of types;
- a type in parentheses.

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
| `//` | `Integer`, floored | [`InvalidOperandType`](errors.md#invalidoperandtype) error | [`InvalidOperandType`](errors.md#invalidoperandtype) error |
| unary `+` `-` | `Integer` |  | `Number` |

- **Floored:** `/` and `//` on two integers round the quotient toward negative infinity. Thus `7 / 2` is `3`, `-7 / 2` is `-4` and `7 // -2` is `-4`. Unary `-` binds more tightly, so `-7 / 2` is `(-7) / 2`. `7.0 / 2` is `3.5`.
- **`//` takes only integers.** Any other operand raises an [`InvalidOperandType`](errors.md#invalidoperandtype) error.
- **Division by zero** raises a [`DivisionByZero`](errors.md#divisionbyzero) error. This applies to `/` with a divisor of `0` or `0.0`. It also applies to `//` with a divisor of `0`, after the runtime checks the types of the operands.
- **Comparisons are by value.** The operators `==`, `!=`, `<`, `>`, `<=` and `>=` compare an `Integer` with a `Number` by value. Thus `1 == 1.0` and `1 < 1.5` are `true`. Equality compares the elements of arrays and the values of maps in the same way. Thus `[1] == [1.0]` is `true`.
- **Integers do not overflow.** `9223372036854775807 + 1` is `9223372036854775808`.
- Any other operand to an arithmetic operator raises an [`InvalidOperandType`](errors.md#invalidoperandtype) error. An example is `1 + true`.

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

A function has a type: its signature without its name or parameter names. How this type displays is up to the runtime.

## Declaring a function

The parameters of a declaration come in this order:

1. positional parameters;
2. at most one variadic parameter, `*name: T`;
3. keyword parameters, `name: T = default`;
4. at most one rest keywords parameter, `**name: T`.

A syntax error occurs when a declaration has its parameters in another order. A syntax error also occurs when a parameter name occurs twice in one declaration. A reserved keyword cannot be a function name. This is also a syntax error.

Every parameter declares its type. Use `Any` to accept every value. The runtime checks each argument against this type (see [Type-checked calls](#type-checked-calls)).

Every keyword parameter has a default expression. A call that omits the keyword evaluates the default expression for that call. The runtime checks the value of the default against the type of the keyword.

The body is a sequence of statements. The value of the function is the value of the last statement, or the value of a `return` (see [Returning early](#returning-early)). An empty body gives `None`. The result must be of the return type. If it is not, the call raises a [`ReturnTypeMismatch`](errors.md#returntypemismatch) error.

```flt
def greet(name: String, greeting: String = "Hello") -> String
  greeting <> ", " <> name
end

def log(label: String, *values: Any, level: Symbol = :info)
  None
end
```

A call to a function runs its body in a scope of its own. The parameters are bound in this scope. A variable that the body declares is gone when the call ends.

A `def` statement binds its name to a function value, and gives `None`. The name is bound before the body runs, so a function can call itself. A runtime decides if a second `def` with the same name adds an overload, and what happens when the name is already in use.

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

In the whitespace form, a `{` starts an argument only if it starts a map. Thus `f { a: 1 }` passes a map, but in `if x { 1 }` the braces are the arm of the `if`, not an argument of `x`.

Wrong calls raise errors:

```flt
mod(1)           # ArgumentCountMismatch: one argument is missing
mod(1, 2, 3)     # ArgumentCountMismatch: one argument is extra
mod(1, "two")    # ArgumentTypeMismatch: "two" is not an Integer
```

#### Keyword parameters

A keyword parameter gets its value from a name in the call. Each keyword parameter has a default value, so a call can omit it. The order of keyword arguments does not matter. A positional argument never fills a keyword parameter.

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

A function can also accept other keywords. These are rest keywords, and they have one type. The parameter is `**name: T`. The function gets the extra keywords as a `Map`.

```flt
options()                          # {}
options(color: :red, size: 3)      # { color: :red, size: 3 }
```

#### Variadic parameters

A variadic parameter takes zero or more positional arguments. It must be the last positional parameter of the function. It never takes keywords. Each value must be of the type of the parameter. A function with `k` positional parameters before the variadic one accepts any call with at least `k` positional arguments. The function gets the values as an `Array`.

```flt
# sum takes any number of Numbers
sum()                              # 0.0: no values
sum(1, 2, 3)                       # 6.0
# join takes one separator, then any values
join(",", 1, :a, "b")              # "1,:a,b"
# log takes a String, any values, and the keyword level
log("started", 1, 2, level: :info) # a call with both kinds of argument
```

Wrong calls raise errors:

```flt
join()                             # ArgumentCountMismatch: the separator is required
sum(1, "two")                      # ArgumentTypeMismatch: "two" is not a Number
sum(1, total: 2)                   # UnexpectedKeywordArgument: sum declares no keywords
```

### Returning early

`return expr` stops the call of the nearest enclosing function ([ADR 023](adrs/023-return-statement.md)). The value of the call is the value of `expr`. A `return` with no expression gives `None`. The expression is on the same line as `return`. A line break, a `;` or `end` after `return` ends it.

```flt
def check(x: Integer) -> String
  if x // 2 * 2 == x
    return "even"
  end

  check(x * 3 + 1)
end
```

- A `return` at the end of a body is optional. A body that ends with `return e` gives the same value as one that ends with `e`. A body that ends with `return` alone gives `None`.
- A `return` stops the call at once, also from inside a block or an arm of an `if`. The rest of the statement or expression that holds it is not evaluated.
- The value of a `return` must be of the return type, as the value of the last statement must. If it is not, the call raises a [`ReturnTypeMismatch`](errors.md#returntypemismatch) error.
- A `return` can occur only in the body of a `def`, and in the blocks inside it. A `return` at the top level, also inside a top-level block, is a syntax error. A `return` in a default of a parameter is a syntax error too.
- Statements after a `return` in the same block are allowed. They never run.
- A runtime that is given a `return` outside a call, for example by its host, raises a [`ReturnOutsideFunction`](errors.md#returnoutsidefunction) error.

### Calls

```flt
add(1, 2)            # parenthesized: zero or more arguments
add 1, 2             # whitespace: at least one argument
foo(1, bar: true)    # trailing keyword arguments
```

Positional arguments come first. After the first keyword argument, no positional argument can follow. Keyword keys follow the rules for map keys, so each key is a symbol. A call cannot repeat a key. A duplicate keyword is a syntax error.

Keyword arguments are passed by name, separately from positional arguments. Thus `foo(1, bar: true)` passes `1` by position and `bar` by keyword. `foo(1, { bar: true })` passes two positional arguments.

The runtime evaluates positional arguments from left to right. Then it evaluates keyword values from left to right.

A call raises one of these errors:

- an [`UndefinedFunction`](errors.md#undefinedfunction) error when the name is not bound;
- a [`NotCallable`](errors.md#notcallable) error when the name is bound to a value that is not a function;
- an [`ArgumentCountMismatch`](errors.md#argumentcountmismatch) error when the function does not take that number of positional arguments;
- an [`ArgumentTypeMismatch`](errors.md#argumenttypemismatch) error for a positional argument of the wrong type;
- an [`UnexpectedKeywordArgument`](errors.md#unexpectedkeywordargument) error for a keyword that the function does not accept;
- a [`KeywordTypeMismatch`](errors.md#keywordtypemismatch) error for a keyword value of the wrong type;
- a [`ReturnTypeMismatch`](errors.md#returntypemismatch) error when the result is not of the return type.

### Type-checked calls

The runtime checks each argument against the type of its parameter. A value is of a type in these cases:

- Every value is of type `Any`.
- A value is of a built-in type if it has that type.
- A value is of type `Number` if it is a `Number` or an `Integer`.
- A value is of type `A | B` if it is of either type.

Sometimes an `Integer` is of a type only because `Number` accepts it. Then the runtime converts the `Integer` to a `Number`. It converts an argument or a keyword value before it calls the function. It converts a result before it returns the result.

A runtime can let a function have several signatures, or **overloads**. It then chooses the signature to call from the number and the types of the arguments. How it chooses, and which error it raises when none matches, is up to the runtime (see [`runtime.md`](runtime.md#type-checked-calls)).

### Pipe

`x |> f(a, ...)` evaluates as `f(x, a, ...)`. `x |> f` evaluates as `f(x)`. The left side becomes the **first** argument, as in Elixir. Pipes are left-associative. Thus `a |> f |> g(1)` is `g(f(a), 1)`. For example: `READ("input") |> SELECT(:id) |> WRITE("output")`. `|>` has level 9 in the table above.

The runtime evaluates the left side first. Then it evaluates the arguments of the right side. The right side must be a call or a bare name. The runtime resolves and checks it like any call, with the piped value as its first argument. For example, calling a function of one parameter as `1 |> f(2)` raises an [`ArgumentCountMismatch`](errors.md#argumentcountmismatch) error. Any other right side raises an [`InvalidPipeTarget`](errors.md#invalidpipetarget) error.

The piped value is positional. `x |> f(a, k: v)` passes `x` and `a` by position and `k` by keyword. Thus `f 1`, `f(1)`, `1 |> f` and `1 |> f()` are equivalent.

The whitespace call form is not allowed next to `|>`. The parser reads the right side like an `if` condition. In an `if` condition, a name followed by whitespace is not a call. Thus `1 |> f 2` is a parse error.

On the left side, a whitespace-form call cannot take an unparenthesized pipe as an argument. Thus `f 1 |> g()` (which would mean `f(1 |> g())`) is also a parse error.

## 7. Bindings

A variable is a binding of an identifier. At run time, it binds the identifier to a value.

- **`let` declares a variable.** `let x = expr` declares `x` without a type. `let x: T = expr` declares `x` with the type `T`. The value must be of type `T`, by the same rules as for a function argument (§6). If it is not, the declaration raises a [`VariableTypeMismatch`](errors.md#variabletypemismatch) error. The runtime also converts the value in the same way. Thus `let n: Number = 1` binds `1.0`. A declaration replaces any earlier declaration of `x`, including its type.
- **`const` declares a constant.** `const X = expr` and `const X: T = expr` bind `X` as `let` does, and follow the same rules for the type. A constant is a binding that the program does not intend to change. A runtime decides what happens when a program rebinds one. By convention, the name of a constant is in `ALL_CAPS`, such as `DEFAULT`. The language does not enforce this.
- **`x = expr` assigns.** If `x` was declared with a type, the value must be of that type. If it is not, the assignment raises a [`VariableTypeMismatch`](errors.md#variabletypemismatch) error, and `x` keeps its value. The runtime converts the value as in a declaration. If `x` has no declared type, the value replaces the binding, including its type. After `let x = 1` and then `x = "a"`, `x` has type `String`. An assignment to a name with no binding declares the name without a type.
- **The runtime checks the name and the type before it evaluates the value.** A type name that the runtime does not know raises an [`UnknownType`](errors.md#unknowntype) error.
- **A binding evaluates to the bound value.**
- **Bindings are global, except in a function call.** A binding outside a function body is visible to every statement after it. A binding in the scope of a function call is limited to the call (§6). A runtime decides whether a block has a scope of its own (§2). The runtime decides what other names a binding cannot use (see [`runtime.md`](runtime.md#scopes)).
- A reference to an identifier with no binding raises an [`UnboundIdentifier`](errors.md#unboundidentifier) error.

```flt
let n: Number = 1         # n is 1.0
n = 2                     # ok: n is 2.0
n = "two"                 # VariableTypeMismatch
let i: Integer = 1.0      # VariableTypeMismatch
let n = "two"             # ok: declares n again, without a type
let x: Number | None = None
let y: Foo = 1            # UnknownType
const DEFAULT = "default"
const LIMIT: Integer = 10
const RATE: Number = 2    # RATE is 2.0
```

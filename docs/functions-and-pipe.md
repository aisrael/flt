# Function Calls and Pipe

## Function Call Forms

`flt` supports two call styles.

### Parenthesized form

```flt
foo()
bar(1)
add(1, 2)
```

### Keyword arguments

In the parenthesized form, arguments may include trailing **keyword arguments**: key-value pairs with the form `key: value`. All positional arguments must come first; after the first keyword argument, no further positional arguments are allowed.

```flt
foo(1, bar: true)
baz(a: 1, b: 2)
qux(1, 2, option: "value")
```

Keys follow the same rules as [map keys](./literals.md#maps): each is a symbol, written bare or quoted, so `bar: true` and `"bar": true` both pass the keyword `:bar`.

Keyword arguments are passed by name, separately from the positional ones ([ADR 010](./adrs/010-keyword-arguments.md)). A function declares which keywords it accepts, each with a default, so a call can leave any of them out. With a host that registers the [native function](./native-functions-and-values.md#keyword-arguments) `greet`, which declares the keyword `greeting` with the default `"Hello"`:

```flt
greet("Ann")                   # "Hello, Ann"
greet("Ann", greeting: "Hi")   # "Hi, Ann"
"Ann" |> greet(greeting: "Hi") # "Hi, Ann"
typeof(greet)                  # fn(String, greeting: String) -> String
```

- **A keyword the function doesn't declare is an error**, even for a function that declares none: `greet("Ann", tone: "warm")` raises `Function greet got an unexpected keyword argument: tone`, and so does `typeof(1, k: 2)`, with `k`. A function can declare that it accepts any other keywords instead.
- **A keyword can't be repeated.** `greet("Ann", greeting: "Hi", greeting: "Yo")` is a parse error, `Duplicate keyword argument greeting in the call to greet`, and so is `greet("Ann", greeting: "Hi", "greeting": "Yo")`, since both keys are `:greeting`.
- **A keyword is never filled by position.** `greet("Ann", "Hi")` passes two positional arguments, and raises `Function greet expected 1 argument(s), found 2`.
- **A map is a positional argument.** `f(k: 1)` passes the keyword `k`, and `f({k: 1})` passes one map.
- **Keyword values are type-checked**, like positional arguments ([Calls](#calls)): `greet("Ann", greeting: 1)` raises `Keyword argument greeting of greet expects String, found Integer`.

Positional arguments are evaluated first, left to right, then keyword values, left to right.

### Whitespace form

In this form, at least one argument is required.

```flt
add 1
add 1, 2
```

The arguments must be on the same line as the name, so a name alone on a line, followed by a statement on the next line, is two statements. A `-` followed by a space is a subtraction, so `n - 1` is not the call `n(-1)`; write `n -1` for that call. The keyword `end` is never an argument. A `{` starts an argument only if it starts a map: `f { a: 1 }` passes a map, but `f { 1 }` is the name `f` followed by a block.

This form parses exactly like the parenthesized one: `add 1, 2` and `add(1, 2)` give the same AST. It can't be used next to a [pipe](#calls-next-to-a-pipe).

## Declaring Functions

flt source can declare a function with `def` ([ADR 019](./adrs/019-def-function-declarations.md)). The examples use `end` to close the body.

```flt
def mod(n: Integer, d: Integer) -> Integer
  n - (n // d) * d
end

mod(7, 3)             # 1
typeof(mod)           # fn(Integer, Integer) -> Integer
```

### Parameters

The parameters come in this order:

1. positional parameters, `name: Type`;
2. at most one variadic parameter, `*name: Type`;
3. keyword parameters, `name: Type = default`;
4. at most one rest keywords parameter, `**name: Type`.

```flt
def log(label: String, *values: Any, level: Symbol = :info, **extra: Any)
  None
end
```

A declaration in another order is a parse error, and so is a name used twice. Every parameter has a type, or is a [literal pattern](#pattern-matching). Use `Any` to accept every value. Inside the body, a variadic parameter is an `Array` and a rest keywords parameter is a `Map`.

A keyword parameter must have a default. A call that leaves out the keyword evaluates the default for that call. A default can use global constants and calls to [`const` functions](#const-functions), but not other parameters or global variables ([Scope](#scope)).

### Return Type and Body

The body is a list of statements that ends with `end`. The function gives the value of its last statement. An empty body gives `None`.

Without `-> Type`, the return type is `None`. Then the body must give `None`, or the call raises `ReturnTypeMismatch`. The result of every call is checked against the return type.

```flt
def two() -> Integer
  1
  2
end

two()                 # 2

def one()             # no return type, so -> None
  1
end

one()                 # error: the body gives an Integer, not None
```

### Returning Early

`return` stops the call and gives a value ([ADR 023](./adrs/023-return-statement.md)). A `return` as the last statement is optional, so these two functions are the same:

```flt
def foo() -> Boolean
  true
end

def foo() -> Boolean
  return true
end
```

A `return` earlier in the body leaves the function at once, also from inside an `if` or a block:

```flt
def check(x: Integer) -> String
  if x // 2 * 2 == x
    return "even"
  end

  check(x * 3 + 1)
end

check(8)              # "even"
```

`return` alone gives `None`. The value of a `return` is checked against the return type, so `return "even"` in a function declared `-> Integer` raises `ReturnTypeMismatch`. A `return` outside a function body is a parse error. Statements after a `return` in the same block never run. The reference runtime warns about them when it declares the function ([ADR 024](./adrs/024-unreachable-code-warning.md)).

### Scope

Each call has its own scope:

- Parameters and variables declared with `let` in the body exist only during the call.
- A name that the scope does not bind is looked up in the global scope. The body sees global constants, functions and type names, but not global variables ([ADR 027](./adrs/027-function-visibility.md)). A reference to a global variable raises `UnboundIdentifier`.
- `x = expr` assigns to the nearest binding of `x` that the body sees. If there is none, it declares `x` in the call. After `let x = 1` at the top level, `x = 2` in a body declares a local `x`, and the global `x` keeps its value.
- A [brace block](./syntax.md#brace-blocks) in the body has its own scope inside the call's scope. See [block scope](./runtime.md#block-scope).

### Names and Overloads

A `def` is allowed only at the top level, not in a block or in the body of a function: that raises `DefNotAtTopLevel`. A `def` binds its name globally and gives `None`. The name is bound before the body runs, so a function can call itself.

A second `def` with the same name adds an overload ([Overloads](#overloads)). Two overloads can't have the same parameter types: that raises `DuplicateOverload`. A `def` can't use the name of a built-in type, a native type, a variable, or a function that a host registered: that raises `NameAlreadyBound`. A function declared in flt follows the same rules for [calls](#calls) as a native function. All overloads of a function are `const`, or none is: a `def` that adds an overload to a `const` function, or the other way round, raises `ConstOverloadMismatch` ([`const` Functions](#const-functions)).

Known limitation: `SimpleRuntime` does not yet share overloads with a name that is bound to the function. After `let f = show`, a later `def show(…)` adds an overload to `show` but not to `f`. Declare every overload before you bind the function to another name.

### Pattern Matching

A positional parameter can be a literal instead of a typed name. The parameter then accepts only that value ([ADR 030](./adrs/030-pattern-matching.md)). The literal is an integer, a number, a string, a boolean, `None` or a symbol. An integer or a number can have a `-` or `+` sign in front, with no space, as in `-1`, `-2.5` and `+1`.

```flt
def foo(i: Integer) -> Integer
  i * 2
end

def foo(0) -> Integer
  1
end

foo(0)         # 1, from foo(0)
foo(3)         # 6, from foo(i: Integer)
```

A call matches a pattern if its argument has the same type as the literal and is equal to it. A literal keeps its own type, so `2` is an `Integer` and `2.0` is a `Number`:

| Overload       | `foo(0)` | `foo(0.0)` | `foo(1)` |
| -------------- | -------- | ---------- | -------- |
| `def foo(0)`   | matches  | no         | no       |
| `def foo(0.0)` | no       | matches    | no       |

`def foo(2)` does not accept `2.0`, although `2 == 2.0` is `true`. To accept both, declare both overloads, or declare a `Number`.

```flt
def fib(0) -> Integer
  0
end

def fib(1) -> Integer
  1
end

def fib(n: Integer) -> Integer
  fib(n - 1) + fib(n - 2)
end

fib(10)        # 55
```

- A pattern has no name. The body can't read it.
- A pattern can be in a positional parameter only. A pattern in a variadic parameter, a keyword parameter or the rest keywords is a parse error. So is a string pattern with interpolation, such as `"a{x}"`.
- A pattern can come with other parameters: `def scale(0, factor: Integer = 1)` and `def add(0, 0)`.
- A name without a type, such as `def foo(x)`, is still a parse error. Only a literal is a pattern. A sign is allowed before a number only: `-x`, `--1`, `-(1)` and `- 1` are parse errors. `+1` is the same pattern as `1`, so `def foo(+1)` after `def foo(1)` raises `DuplicateOverload`.
- A `const def` can have patterns. All overloads are `const`, or none is ([`const` Functions](#const-functions)).
- A pattern is part of the signature of its overload. `foo(0)` and `foo(i: Integer)` are different overloads, and so are `foo(0)` and `foo(0.0)`. A second `def foo(0)` raises `DuplicateOverload`.
- The order of the `def`s does not matter. `SimpleRuntime` runs the [narrowest](#the-narrowest-overload) overload, and a pattern is the narrowest overload for its value.

### `const` Functions

A function declared with `const def` is meant to be pure and deterministic: a call with the same arguments always gives the same result ([ADR 028](./adrs/028-const-functions.md)). In the reference runtime, a `const` function can _only_ call other `const` functions ([ADR 029](./adrs/029-const-functions-in-the-reference-runtime.md)).

```flt
const def area(w: Integer, h: Integer) -> Integer
  w * h
end
const def double_area(w: Integer, h: Integer) -> Integer
  area(w, h) * 2            # a call to a const function: allowed
end
def now_area(w: Integer, h: Integer) -> Integer
  area(w, h)                # a def function can call a const function
end

typeof(area)                # const fn(Integer, Integer) -> Integer
```

A call from a `const` function to a function that is not `const` raises `NonConstFunctionCall`. The runtime checks this when the call runs, so a branch that does not run gives no error:

```flt
def stamp() -> Integer
  now()
end
const def bad() -> Integer
  stamp()                   # NonConstFunctionCall
end
bad()
```

- A default of a keyword parameter follows the same rule, for `def` and for `const def`. It can call a `const` function. A call to another function raises `NonConstFunctionCall`.
- All overloads of a function are `const`, or none is. A `def` that adds an overload to a `const` function raises `ConstOverloadMismatch`, and so does a `const def` that adds an overload to a function declared with `def`.
- A host can register native `const` functions ([Native `const` Functions](./native-functions-and-values.md#native-const-functions)). The built-in `typeof` is `const`. A native function that the host does not mark is assumed to have side effects.
- A `const` function is pure for a fixed set of global constants.

## Pipe Operator

The pipe operator `|>` passes the value on its left as the **first** argument of the function on its right, as in [Elixir](https://elixirschool.com/en/lessons/basics/pipe_operator).

```flt
1 |> typeof              # typeof(1)         → Integer
1 |> typeof()            # typeof(1)         → Integer
typeof(1) |> typeof      # typeof(typeof(1)) → Type
10 |> sub(3)             # sub(10, 3)        → 7
x |> f(a, k: v)          # f(x, a, k: v)
a |> f |> g(1)           # g(f(a), 1)
1 |> typeof == Integer   # (1 |> typeof) == Integer → true
READ("input") |> SELECT(:id) |> WRITE("output")
```

Pipes are left-associative, so `a |> f |> g` is `(a |> f) |> g`. The left side is evaluated first, then the arguments on the right, left to right. The call on the right is never evaluated on its own, so `1 |> typeof()` works although `typeof()` alone raises an arity error.

### Grouping With Other Operators

`|>` binds looser than `+`, `-` and `<>`, and tighter than everything else ([precedence](./operators.md#precedence-low-to-high)), as in Elixir:

```flt
1 + 1 |> typeof          # (1 + 1) |> typeof   → Integer
x |> f == 3              # (x |> f) == 3
x |> f & 1               # (x |> f) & 1
a || b |> f              # a || (b |> f)
```

### The Right Side

The right side is a call, in either [call form](#function-call-forms), or a bare function name. A bare name `f` is the same as `f()`. The name is resolved like any other call ([name resolution](#name-resolution)), so a function bound to another name can be piped into:

```flt
let g = identity
42 |> g                  # identity(42) → 42
```

The piped value is passed positionally, before the call's other positional arguments: `x |> f(a, k: v)` calls `f` with `x` and `a` positionally and `k` by keyword.

### Equivalent Forms

A call and a pipe into a function are interchangeable. These all evaluate to `identity(1)`, though the last three parse to different trees:

```flt
identity 1               # parses the same as identity(1)
identity(1)
1 |> identity
1 |> identity()
```

### Calls Next to a Pipe

A call without parentheses takes everything after its name as its arguments, so it isn't allowed on either side of `|>`:

- On the right, a call needs parentheses. The right side is parsed like an `if` condition, where a name followed by whitespace isn't a call, so `1 |> f 2` is a parse error. Write `1 |> f(2)`. A bare name, as in `1 |> f`, is fine.
- On the left, a call without parentheses can't take a pipe as an argument. `f 1 |> g()` would mean `f(1 |> g())`, so it's a parse error. Write `f(1) |> g()`, or `f(1 |> g())`.

Inside the parentheses of a call on the right, calls need parentheses too, as in an `if` condition: `x |> f(g(1))`, not `x |> f(g 1)`. And `if x |> f "Ok" else "No"` reads as the condition `x |> f` followed by the `"Ok"` branch.

### Errors

A pipe calls the function exactly as the equivalent call would, so it raises the same [call errors](#calls). The argument count includes the piped value:

- `1 |> typeof(2)` calls `typeof(1, 2)`, and raises `Function typeof expected 1 argument(s), found 2`.
- `1 |> nope()` raises `Undefined function: nope`.
- `1 |> Number` raises `Number is not a function`.

Piping into anything other than a call or a name raises `Cannot pipe into <expr>`, e.g. `1 |> 2` raises `Cannot pipe into 2`.

## Built-in Functions

The runtime currently registers two built-in functions. `typeof` is [`const`](#const-functions), and `println` is not, because it has a side effect:

- `typeof(value)` - returns the [`Type`](./types.md#types-as-values) of `value` as a first-class value, e.g. `typeof(42)` evaluates to `Integer`, `typeof(4.2)` to `Number`, `typeof({a: 1})` to `Map`, and `typeof(42) == Integer` is `true`. `typeof(None)` evaluates to `None`. Calling it with the wrong number of arguments raises a `Function typeof expected 1 argument(s), found <n>` error.
- `println(s)` - prints the `String` `s` and a new line to standard output, and evaluates to `None`. It has the type `fn(String) -> None`. It accepts only a `String`, so `println(1)` raises `ArgumentTypeMismatch`. Use [string interpolation](./literals.md) to print another value, e.g. `println("x is {x}")`.

```flt
println("Hello, world!")  # prints Hello, world! and evaluates to None
typeof(println)           # fn(String) -> None
"Hello" |> println        # prints Hello
```

Built-ins are [native functions](./native-functions-and-values.md#native-functions) that the runtime registers in the global scope, so they're function values like any other. `typeof` has the type `const fn(Any) -> Type`:

```flt
typeof                # <fn typeof>
typeof(typeof)        # const fn(Any) -> Type
let t = typeof
t("hi")               # String
1 |> t                # Integer
```

A built-in's name can't be bound, like any function's name (see [Name Resolution](#name-resolution)):

```flt
let typeof = 1        # error: Cannot bind typeof: it's a function
```

## Native Functions

A host embedding `flt` can register its own functions, written in Rust, and its own types for the values they pass around. See [Native Functions, Types and Values](./native-functions-and-values.md).

## Name Resolution

An identifier, and the name in a call, can name one of:

1. a variable
2. a global function, built-in or [native](./native-functions-and-values.md#native-functions)
3. a [built-in type](./types.md#types-as-values)
4. a [native type](./native-functions-and-values.md#native-types-and-values) registered by the host

A name is never more than one of these, because a binding can't shadow a function, a built-in type or a native type ([ADR 006](./adrs/006-no-shadowing.md), [ADR 007](./adrs/007-native-types-and-values.md)). A function value can still be bound to a new name. With a host that registers the [native function](./native-functions-and-values.md#native-functions) `identity`:

```flt
let identity = 1      # error: Cannot bind identity: it's a function
let f = identity      # binds the function value to a new name
f(42)                 # 42
```

## Calls

Every call is type-checked before the function runs ([ADR 013](./adrs/013-type-checked-function-calls.md)). A function's signature declares the type of each positional argument, each [variadic](./native-functions-and-values.md#variadic-arguments) value, each [keyword argument](#keyword-arguments) and its result, and the call has to match:

- **The number of positional arguments.** A signature whose last positional argument is variadic takes any number of positional arguments after the ones before it ([ADR 011](./adrs/011-variadic-arguments.md)).
- **Each argument's type.** `Any` accepts every value. `Number` accepts an `Integer` too, and converts it to a `Number`, so a function declared with `Number` always gets a `Number` ([`Number` Accepts an `Integer`](./types.md#number-accepts-an-integer)). A union accepts a value of any of its members, and a [native type](./native-functions-and-values.md#native-types-and-values) only values of that same type.
- **Each keyword.** It has to be one the signature declares, with a value of its type, or one its rest keywords accept.
- **The result.** It has to be of the signature's return type.

With `pad(s: String, width: Integer) -> String`, `sprintf(format: String, *args: Any) -> String` and `identity(v: Any) -> Any`:

- `pad(1, 2)` raises `Argument s of pad expects String, found Integer`.
- `pad("x", 2.0)` raises `Argument width of pad expects Integer, found Number`.
- `identity(1, 2)` raises `Function identity expected 1 argument(s), found 2`.
- `sprintf()` raises `Function sprintf expected at least 1 argument(s), found 0`.
- `identity(1, k: 2)` raises `Function identity got an unexpected keyword argument: k`.
- `let x = 1; x(2)` raises `x is not a function`.
- `nope(1)`, where `nope` isn't bound, raises `Undefined function: nope`.

### Overloads

A function can have several signatures, or overloads. A call runs the one that matches it:

- **Overloads are tried by arity, longest first.** An overload's arity is the number of its positional arguments before any variadic one, so a call reaches the overload that matches the most of its arguments one for one, whatever order the overloads were declared in.
- **Types and keywords tell overloads of the same arity apart.** With `count(plan: Plan) -> Integer` and `count(column: Symbol | String) -> Aggregate`, `count(:id)` runs the second. With `foo(s: String, n: Number = 0)` and `foo(s: String, t: String = "")`, `foo("abc", n: 1)` runs the first.
- **A tie isn't specified.** If several overloads of the same arity match, flt doesn't say which one runs, so flt source shouldn't depend on it. `SimpleRuntime` runs the [narrowest](#the-narrowest-overload), and if none is narrowest, the one declared first.

When no overload matches:

- **No overload takes that many positional arguments:** the error lists the counts they take, e.g. `Function g expected 1 or 2 argument(s), found 3`, or `Function f expected 1 or at least 3 argument(s), found 2`.
- **One takes that many, but doesn't match:** the error names the first thing that fails, checking the positional arguments in order, then the keywords in call order, e.g. `Argument s of pad expects String, found Integer`. For a [literal pattern](#pattern-matching) the error shows the position of the argument, the literal and the value that was passed, e.g. `Argument 1 of foo expects 0, found 1`.
- **Several take that many, but none matches:** the error is the closest overload's, as if it were the only one ([ADR 015](./adrs/015-closest-overload-errors.md)). The closest is the one that passes the most checks before one fails. A literal pattern is one more check, after the type check of its argument. With `head(plan: Plan, n: Number)` and `head(path: String, n: Number)`, `read("data.parquet") |> head("x")` raises `Argument n of head expects Number, found String`, because `head(plan, n)` takes the plan. If several first fail at the same keyword, one that declares it is closer than one that doesn't, so with the two `foo`s above, `foo("abc", n: "x")` raises `Keyword argument n of foo expects Number, found String`.
- **Several are equally close:** `No overload of count accepts (Number)`, listing the call's argument types, then its keywords, e.g. `No overload of foo accepts (String, z: Number)`.

A function's type shows every overload: `typeof(count)` is `fn(Plan) -> Integer | fn(Symbol | String) -> Aggregate`. A literal pattern shows as the literal, so `typeof(foo)` for the `foo` above is `fn(Integer) -> Integer | fn(0) -> Integer`.

### The Narrowest Overload

Since `Number` accepts an `Integer`, overloads that take an `Integer` and a `Number` both match an integer. flt doesn't say which one runs, but `SimpleRuntime` runs the **narrowest** ([ADR 017](./adrs/017-integers.md)):

- One type is at least as narrow as another if they're the same, if the other is `Any`, if it's `Integer` and the other is `Number`, or if it's a member of the other, a union. A union is at least as narrow as a type if each of its members is.
- A literal pattern has a type that has one value. That type is at least as narrow as itself, as its base type (`0` is at least as narrow as `Integer`), as `Number` if it is an integer, as `Any`, and as a union that has one of these. Two different literals are not at least as narrow as each other.
- One overload is narrower than another if each of its types for the arguments the call passes is at least as narrow as the other's, and at least one is narrower.
- If no overload is narrower than every other, `SimpleRuntime` runs the one declared first.

With these overloads:

```
f(n: Number) -> String
f(i: Integer) -> String
show(x: Any) -> String
show(x: Number) -> String
g(a: Integer, b: Number) -> String
g(a: Number, b: Integer) -> String
h(n: Number) -> String
h(i: Integer) -> String
h(0) -> String
```

```flt
f(1)           # runs f(i: Integer)
f(1.0)         # runs f(n: Number), the only one that matches
show(1.0)      # runs show(x: Number), though show(x: Any) was declared first
g(1, 2)        # neither is narrower, so runs the first declared, g(a: Integer, b: Number)
h(0)           # runs h(0), the narrowest
h(1)           # runs h(i: Integer)
h(1.0)         # runs h(n: Number), the only one that matches
```

The overload is chosen before an `Integer` is converted to a `Number`, so `f(1)` passes `f(i: Integer)` the `Integer` `1`.

## Current Semantics Note

- Parsing for function calls (including keyword arguments) and pipe expressions is implemented.
- Calling a [native function](./native-functions-and-values.md#native-functions), whether a built-in (`typeof` or `println`) or one a host registered, evaluates it.
- [Native values](./native-functions-and-values.md#native-types-and-values) pass through flt between native functions; flt source can't create them.
- Pipes (`|>`) are evaluated. Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not supported yet.

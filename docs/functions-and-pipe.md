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

This form parses exactly like the parenthesized one: `add 1, 2` and `add(1, 2)` give the same AST. It can't be used next to a [pipe](#calls-next-to-a-pipe).

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

The runtime currently registers one built-in function:

- `typeof(value)` - returns the [`Type`](./types.md#types-as-values) of `value` as a first-class value, e.g. `typeof(42)` evaluates to `Integer`, `typeof(4.2)` to `Number`, `typeof({a: 1})` to `Map`, and `typeof(42) == Integer` is `true`. `typeof(None)` evaluates to `None`. Calling it with the wrong number of arguments raises a `Function typeof expected 1 argument(s), found <n>` error.

Built-ins are [native functions](./native-functions-and-values.md#native-functions) that the runtime registers in the global scope, so they're function values like any other. `typeof` has the type `fn(Any) -> Type`:

```flt
typeof                # <fn typeof>
typeof(typeof)        # fn(Any) -> Type
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
- **One takes that many, but doesn't match:** the error names the first thing that fails, checking the positional arguments in order, then the keywords in call order, e.g. `Argument s of pad expects String, found Integer`.
- **Several take that many, but none matches:** the error is the closest overload's, as if it were the only one ([ADR 015](./adrs/015-closest-overload-errors.md)). The closest is the one that passes the most checks before one fails. With `head(plan: Plan, n: Number)` and `head(path: String, n: Number)`, `read("data.parquet") |> head("x")` raises `Argument n of head expects Number, found String`, because `head(plan, n)` takes the plan. If several first fail at the same keyword, one that declares it is closer than one that doesn't, so with the two `foo`s above, `foo("abc", n: "x")` raises `Keyword argument n of foo expects Number, found String`.
- **Several are equally close:** `No overload of count accepts (Number)`, listing the call's argument types, then its keywords, e.g. `No overload of foo accepts (String, z: Number)`.

A function's type shows every overload: `typeof(count)` is `fn(Plan) -> Integer | fn(Symbol | String) -> Aggregate`.

### The Narrowest Overload

Since `Number` accepts an `Integer`, overloads that take an `Integer` and a `Number` both match an integer. flt doesn't say which one runs, but `SimpleRuntime` runs the **narrowest** ([ADR 017](./adrs/017-integers.md)):

- One type is at least as narrow as another if they're the same, if the other is `Any`, if it's `Integer` and the other is `Number`, or if it's a member of the other, a union. A union is at least as narrow as a type if each of its members is.
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
```

```flt
f(1)           # runs f(i: Integer)
f(1.0)         # runs f(n: Number), the only one that matches
show(1.0)      # runs show(x: Number), though show(x: Any) was declared first
g(1, 2)        # neither is narrower, so runs the first declared, g(a: Integer, b: Number)
```

The overload is chosen before an `Integer` is converted to a `Number`, so `f(1)` passes `f(i: Integer)` the `Integer` `1`.

## Current Semantics Note

- Parsing for function calls (including keyword arguments) and pipe expressions is implemented.
- Calling a [native function](./native-functions-and-values.md#native-functions), whether a built-in (currently just `typeof`) or one a host registered, evaluates it.
- [Native values](./native-functions-and-values.md#native-types-and-values) pass through flt between native functions; flt source can't create them.
- Pipes (`|>`) are evaluated. Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not supported yet.

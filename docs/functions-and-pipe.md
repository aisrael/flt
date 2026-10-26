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

Keys follow the same rules as [map keys](./literals.md#maps): each is a symbol, written bare or quoted, so `bar: true` passes the key `:bar`. The parser collects keyword arguments into a single map and passes them as the final argument to the call.

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
1 |> typeof              # typeof(1)         → Number
1 |> typeof()            # typeof(1)         → Number
typeof(1) |> typeof      # typeof(typeof(1)) → Type
10 |> sub(3)             # sub(10, 3)        → 7
x |> f(a, k: v)          # f(x, a, {k: v})
a |> f |> g(1)           # g(f(a), 1)
1 |> typeof == Number    # (1 |> typeof) == Number → true
READ("input") |> SELECT(:id) |> WRITE("output")
```

Pipes are left-associative, so `a |> f |> g` is `(a |> f) |> g`. The left side is evaluated first, then the arguments on the right, left to right. The call on the right is never evaluated on its own, so `1 |> typeof()` works although `typeof()` alone raises an arity error.

### Grouping With Other Operators

`|>` binds looser than `+`, `-` and `<>`, and tighter than everything else ([precedence](./operators.md#precedence-low-to-high)), as in Elixir:

```flt
1 + 1 |> typeof          # (1 + 1) |> typeof   → Number
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

Keyword arguments are collected into a trailing map, so the piped value goes before them and they stay last: `x |> f(a, k: v)` calls `f` with `x`, `a` and `{k: v}`.

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

- `typeof(value)` - returns the [`Type`](./types.md#types-as-values) of `value` as a first-class value, e.g. `typeof(42)` evaluates to `Number`, `typeof({a: 1})` evaluates to `Map`, and `typeof(42) == Number` is `true`. `typeof(None)` evaluates to `None`. Calling it with the wrong number of arguments raises a `Function typeof expected 1 argument(s), found <n>` error.

Built-ins are [native functions](./native-functions-and-values.md#native-functions) that the runtime registers in the global scope, so they're function values like any other. `typeof` has the type `fn(Any) -> Type`:

```flt
typeof                # <fn typeof>
typeof(typeof)        # fn(Any) -> Type
let t = typeof
t("hi")               # String
1 |> t                # Number
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

Before a function runs, the runtime checks the number of arguments against the function's signature. Argument types aren't checked yet; that is left to the function itself (see [native types](./native-functions-and-values.md#native-values-in-flt)).

- `identity(1, 2)` raises `Function identity expected 1 argument(s), found 2`.
- `let x = 1; x(2)` raises `x is not a function`.
- `nope(1)`, where `nope` isn't bound, raises `Undefined function: nope`.

## Current Semantics Note

- Parsing for function calls (including keyword arguments) and pipe expressions is implemented.
- Calling a [native function](./native-functions-and-values.md#native-functions), whether a built-in (currently just `typeof`) or one a host registered, evaluates it.
- [Native values](./native-functions-and-values.md#native-types-and-values) pass through flt between native functions; flt source can't create them.
- Pipes (`|>`) are evaluated. Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not supported yet.

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

Keys follow the same rules as map keys (bare identifier or quoted string). The parser collects keyword arguments into a single map and passes them as the final argument to the call.

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

Built-ins are [native functions](#native-functions) that the runtime registers in the global scope, so they're function values like any other. `typeof` has the type `fn(Any) -> Type`:

```flt
typeof                # <fn typeof>
typeof(typeof)        # fn(Any) -> Type
let t = typeof
t("hi")               # String
1 |> t                # Number
```

A variable shadows a built-in, and removing the variable (e.g. with `/unset typeof` in the REPL) makes the built-in visible again:

```flt
let typeof = 1
typeof                # 1
typeof(2)             # error: typeof is not a function
```

## Native Functions

A host application embedding `flt` can register native functions, written in Rust, in the global scope. A function's name then evaluates to a [function value](./types.md#function-types), which can be bound to another name and called through it.

```rust
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use flt::runtime::functions::Argument;
use flt::runtime::functions::FunctionDefinition;
use flt::runtime::functions::NativeFunction;
use flt::runtime::types::Type;
use flt::runtime::SimpleRuntime;
use flt::runtime::Value;

let mut runtime = SimpleRuntime::default();

// identity(v) returns its argument, and has type fn(Any) -> Any
runtime.global_scope.register_function(NativeFunction::new(
    FunctionDefinition::new("identity", Type::any(), vec![Argument::new("v", Type::any())]),
    |args| Ok(args[0].clone()),
));

// A handler is a closure, so it may capture state. Handlers are Send + Sync, so
// mutable state needs Arc<Mutex<_>> or atomics rather than Rc<RefCell<_>>.
let calls = Arc::new(AtomicU64::new(0));
let counter = Arc::clone(&calls);
runtime.global_scope.register_function(NativeFunction::new(
    FunctionDefinition::new("count", Type::number(), vec![]),
    move |_args| Ok(Value::Number((counter.fetch_add(1, Ordering::SeqCst) + 1).into())),
));
```

Registering a function under a name that's already registered replaces it. That includes the [built-ins](#built-in-functions): a host that registers its own `typeof` replaces the default one.

```flt
identity              # <fn identity>
typeof(identity)      # fn(Any) -> Any
identity(42)          # 42
let f = identity
f(42)                 # 42
```

### Name Resolution

An identifier, and the name in a call, are looked up in the same order:

1. variables
2. global functions
3. [built-in type names](./types.md#types-as-values)

So a variable shadows a function. There are no scopes yet, so the binding applies everywhere after it:

```flt
let identity = 1
identity              # 1
identity(2)           # error: identity is not a function
```

### Calls

Before a native function runs, the runtime checks the number of arguments against the function's signature. Argument types aren't checked yet; that is left to the function itself.

- `identity(1, 2)` raises `Function identity expected 1 argument(s), found 2`.
- `let x = 1; x(2)` raises `x is not a function`.
- `nope(1)`, where `nope` isn't bound, raises `Undefined function: nope`.

## Current Semantics Note

- Parsing for function calls (including keyword arguments) and pipe expressions is implemented.
- Calling a [native function](#native-functions), whether a built-in (currently just `typeof`) or one a host registered, evaluates it.
- Pipes (`|>`) are evaluated. Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not supported yet.

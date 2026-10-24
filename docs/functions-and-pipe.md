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

## Pipe Operator

The pipe operator is written as `|>`.

```flt
a |> b |> c
READ("input") |> SELECT(:id) |> WRITE("output")
```

The parser treats this as a left-associative binary operator chain.

## Built-in Functions

The runtime currently registers one built-in function:

- `typeof(value)` - returns the [`Type`](./types.md#types-as-values) of `value` as a first-class value, e.g. `typeof(42)` evaluates to `Number`, `typeof({a: 1})` evaluates to `Map`, and `typeof(42) == Number` is `true`. `typeof(None)` evaluates to `None`. Calling it with the wrong number of arguments raises a `Function typeof expected 1 argument(s), found <n>` error.

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

Registering a function under a name that's already registered replaces it.

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
- Calling a registered built-in (currently just `typeof`) or a host-registered [native function](#native-functions) evaluates it.
- `typeof` is not yet a function value, so `typeof` on its own raises `Unbound identifier: typeof`.
- Runtime evaluation for the pipe operator (`|>`) is not implemented yet; evaluating a pipe expression raises `Pipe operator not yet supported`.

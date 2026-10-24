# ADR 003: Functions as Values

- **Date:** 2026-10-23
- **Status:** Accepted and implemented.
- **PR:** TBD

## Context

A function existed only as an entry in the runtime's private table of built-ins, which only call
syntax could reach. A function's name wasn't a value, so it had no type, couldn't be bound to
another name, and a host embedding flt couldn't add functions of its own.

The main drivers are:

- moving `typeof` into the global scope, so that `typeof(typeof)` is `fn(Any) -> Type`;
- host applications such as datu, which need to register functions like `count()` that keep state.

## Decisions

### 1. Functions are values in the global scope

A function is a first-class value. Hosts register native functions, written in Rust, in the global
scope. A function's name evaluates to a function value, which:

- displays as `<fn name>`;
- can be bound to another name with `let`, and called through any name it's bound to;
- is equal only to itself: two function values are equal only if they're the same function.

A native function's handler is a Rust closure, so it may capture state. Handlers are `Send + Sync`,
so mutable state needs `Arc<Mutex<_>>` rather than `Rc<RefCell<_>>`.

### 2. A function's type is `fn(...) -> R`

A function's type uses the syntax from [ADR 002](002-union-types.md), decision 2: `fn(P1, P2) -> R`,
with a bracketed union return type, e.g. `fn(T) -> (T | None)`.

An overloaded function's type is the union of its signatures' function types (ADR 002, decision 4),
e.g. `fn(String) -> String | fn(String, Number) -> String`.

### 3. One way to resolve names

Identifiers and calls look up a name in the same order:

1. variables;
2. global functions;
3. built-in type names.

So a variable shadows a function, and a function shadows a built-in type name. A side effect is that
after `let typeof = 1`, `typeof(2)` raises `typeof is not a function`.

### 4. The runtime checks arity

Before a handler runs, the runtime checks the number of arguments against the function's signature.
A call passes if any overload has that many arguments. Handlers don't repeat the check.

Argument types aren't checked yet, and are left to the handler.

### 5. Errors

- `Undefined function: <name>`: calling a name with no binding. This replaces
  `Function calls not yet supported`.
- `<name> is not a function`: calling a name bound to a value that isn't a function.
- `Function <name> expected <expected> argument(s), found <found>`: the arity check, which now
  applies to every function.

## Alternatives Considered

- **Plain `fn` pointers as handlers.** Rejected. Hosts need callbacks that capture state.
- **Leave all argument checking to handlers.** Rejected. Checking arity in the runtime is the start
  of checking calls against function signatures, and saves every handler from repeating it.
- **Resolve calls separately from identifiers.** Rejected. One lookup order keeps shadowing
  consistent between `f` and `f(x)`.

## Consequences

- `Value::Function` and `Type::Function` are added.
- `BuiltinFunction` is renamed `NativeFunction`. Its handler is a boxed closure.
- `GlobalScope` holds native functions, and `register_function` adds one.
- `typeof` stays a private built-in, so `typeof` on its own is still an unbound identifier.

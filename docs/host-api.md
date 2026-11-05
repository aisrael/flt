# The `Runtime` Trait

This page describes how a host, a Rust program that embeds flt and works with a runtime. For what the reference runtime does when it runs flt source, see [`runtime.md`](runtime.md). For native functions and types, see [`native-functions-and-values.md`](native-functions-and-values.md).

A host works with a runtime only through the `flt::runtime::Runtime` trait ([ADR 008](./adrs/008-the-runtime-trait.md)). Every runtime implements it. `SimpleRuntime` is the reference implementation, and a host names it only to create one: `SimpleRuntime::default()` returns a runtime with the [built-in functions](./functions-and-pipe.md#built-in-functions) registered.

```rust
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;

let mut runtime = SimpleRuntime::default();
runtime.eval("let x = 1 + 2")?;
runtime.eval("x * 2")?;          // 6
```

| Method | Does |
| --- | --- |
| `eval(source)` | parses and evaluates flt source, returning the last statement's value |
| `eval_statement(statement)` | evaluates one parsed statement |
| `register_function(function)` | registers a [native function](./native-functions-and-values.md#native-functions), replacing one of the same name, or fails |
| `register_native_type::<T>()` | registers the [native type](./native-functions-and-values.md#native-types-and-values) `T`, or fails |
| `register_native_type_info(info)` | the same, given its `NativeTypeInfo` |
| `get_native_type(name)` | a registered native type |
| `set_variable(name, value)` | assigns a global variable, as `name = value` does, or fails. It fails on a constant. A function declared in flt does not see the variable |
| `declare_constant(name, value)` | binds a global constant, replacing a variable of that name, or fails. A function declared in flt sees the constant |
| `get_variable(name)` | a global variable's value |
| `remove_binding(name)` | unbinds a global variable or constant and its declared type, returning its value |
| `check_bindable(name)` | whether `name` can be bound, and if not, why |
| `get_function(name)` | a global function |

## `eval` and `eval_statement`

`eval(source)` parses all of `source` before evaluating any of it, so a syntax error raises `Parser Error: …` and no statement runs. It then evaluates the statements in order and returns the last one's value. Statements are separated by newlines or by `;`. Empty source, or source with only comments, evaluates to `None`.

```rust
runtime.eval("let a = 1; let b = a + 1; b")?;   // 2
runtime.eval("")?;                              // None
runtime.eval("let c = 1; let = 2");             // Err: Parser Error: …, and c stays unbound
```

`eval(source)` is the same as parsing `source` with `flt::parser::parse_statements`, then calling `eval_statement` on each statement.

`eval_statement(&statement)` evaluates a statement the host has already parsed, e.g. with `flt::parser::parse_statements` or `parse_statement`. It's for hosts that parse source themselves, such as a REPL that builds statements from what the user types.

## Calling the Trait Through `dyn Runtime`

A host can hold a `&mut dyn Runtime` or a `Box<dyn Runtime>`. `register_native_type::<T>()` is generic, so it can't be called through `dyn Runtime`. Call `register_native_type_info` instead:

```rust
runtime.register_native_type_info(NativeTypeInfo::of::<Date>())?;
```

## Rules Belong to the Runtime

The trait doesn't say which names a runtime refuses. `set_variable` and `check_bindable` return an error for a name the runtime won't bind, and the `register_*` methods return an error for a name it won't register.

`SimpleRuntime` refuses to bind a built-in type name, a function's name or a native type's name, with `Cannot bind <name>: …`, as `let` does. Its `register_function` and `register_native_type` refuse a name that's already in use, with `Cannot register <function|native type> <name>: …`. Registering a function under an existing function's name replaces it ([Native Functions](./native-functions-and-values.md#native-functions), [Native Types](./native-functions-and-values.md#defining-and-registering-a-native-type)).

## Warnings

`SimpleRuntime` collects the [warnings](errors.md#warnings) that `eval` and `eval_statement` give ([ADR 024](./adrs/024-unreachable-code-warning.md)). A host reads and clears them with `SimpleRuntime::take_warnings`. The `Runtime` trait has no method for warnings, so a host that holds a `dyn Runtime` does not see them. `FltRepl::new()` prints the warnings of its `SimpleRuntime` to standard error. A host that builds the REPL with `FltRepl::with_runtime` gets the same with `.with_warnings(SimpleRuntime::take_warnings)`. Without it, the REPL leaves the warnings to the host. See [Warnings](runtime.md#warnings).

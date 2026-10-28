# ADR 014: Native Function Arguments

- **Date:** 2026-10-28
- **Status:** Proposed.

## Context

A native function's handler gets its call's values as a slice, and its keywords as a `Map` if it
was built with `NativeFunction::with_keywords` ([ADR 010](010-keyword-arguments.md)):

```rust
NativeFunction::new(definition, |args: &[Value]| ...)
NativeFunction::with_keywords(definition, |args: &[Value], keywords: &Map| ...)
```

Handlers read their arguments by index and convert them by hand. datu's are typical:

```rust
fn read(args: &[Value]) -> crate::Result<Value> {
    let path = string_arg("read", &args[0])?;
    ...
}

fn filter(args: &[Value]) -> crate::Result<Value> {
    let plan = args[0].downcast_native::<DatuPlan>()?;
    let sql = string_arg("filter", &args[1])?;
    ...
}
```

where `string_arg` is datu's own helper, which matches on `Value::String` and builds an error
naming the function. This has several problems:

- **The checks are already done.** Every call is type-checked before its handler runs
  ([ADR 013](013-type-checked-function-calls.md)), so `read`'s `args[0]` is known to be a string.
  What's left is unwrapping it, but each host writes its own helpers to do that, and their error
  branches can't be reached.
- **The handler repeats what the signature says.** The definition names the arguments (`path`,
  `plan`, `sql`), but the handler finds them by position, and passes the function's name in again
  for its errors.
- **Positions vary between overloads.** One handler serves every overload of a function (ADR 013,
  decision 9), so the same argument can be in different places. datu's `head` takes `n` either
  positionally or as a keyword, depending on the overload, and finds it with
  `args.get(1).or_else(|| keywords.get(&MapKey::symbol("n")))`.
- **Two constructors, and a panic.** A handler that needs keywords has to use `with_keywords`, and
  `NativeFunction::new` panics if its definition declares keywords, because its handler couldn't
  see them.

The runtime knows which overload it called, and that overload names every argument. So it can pass
the handler its arguments by name.

## Decisions

### 1. Handlers take `&Arguments`

A handler is a `Fn(&Arguments) -> Result<Value, Error>`. `Arguments` (in `flt::runtime::functions`)
holds the call's positional values, its keywords bound as before (declared ones with their
defaults filled in, then any rest keywords), the signature of the overload that was called, and the
function's name.

There's one constructor, `NativeFunction::new(definition, handler)`, for functions with and
without keywords. `NativeFunction::with_keywords` goes, and so does `new`'s panic.

### 2. Arguments are looked up by name

A handler asks for an argument by the name its signature declares, whether the call passed it
positionally or as a keyword:

```rust
fn read(args: &Arguments) -> crate::Result<Value> {
    let path: String = args.get("path")?;
    ...
}

fn filter(args: &Arguments) -> crate::Result<Value> {
    let plan = args.native::<DatuPlan>("plan")?;
    let sql: &str = args.get("sql")?;
    ...
}

/// head(plan, n), or head(plan) with n: 10 as a keyword
fn head(args: &Arguments) -> crate::Result<Value> {
    let n: usize = args.get("n")?;
    ...
}
```

Names are looked up in the overload that was called, so overloads that share a handler should
use the same name for the same thing.

### 3. Typed access

- `get::<T>(name)` converts the argument to `T`, which implements the new `FromValue` trait.
  flt implements it for `String`, `&str`, `bool`, `&Value`, the integer types and `f64` (as their
  `TryFrom<&Value>` does now), and `Option<T>`, which gives `None` for `None`.
- `native::<T>(name)` borrows a native value, as `Value::downcast_native` does
  ([ADR 007](007-native-types-and-values.md)).
- `value(name)` gives the `&Value`, or `None` if the called overload has no argument or keyword of
  that name. A handler serving overloads with different argument names uses it to tell them apart:

  ```rust
  /// head(plan, ...) or head(path, ...)
  match args.value("plan") {
      Some(plan) => ...,
      None => {
          let path: String = args.get("path")?;
          ...
      }
  }
  ```

A value that doesn't convert raises the errors that converting it does now:
`RuntimeError::UnexpectedType` for the wrong type, and `RuntimeError::CannotConvert` for a number
that doesn't fit, like `1.5` or `-1` as a `usize`. Since calls are type-checked, the wrong type
means the declared type is wider than the handler accepts, for example `Any` read as a `String`.

### 4. A name the overload doesn't declare is a bug

`get` and `native` panic if the called overload declares no argument or keyword of that name, as
`args[3]` panics today when there are three arguments. Only the host can make that mistake, and
the panic names the function and the argument.

### 5. Variadic values and the rest

- `rest()` gives the values of a variadic last argument
  ([ADR 011](011-variadic-arguments.md)) as a slice, empty if there are none. `get` and `native`
  take one value, so they panic on the variadic argument's name.
- `keywords()` gives the bound keywords as a `Map`, for handlers that take rest keywords, like
  datu's `select(plan, *columns, **aliases)`.
- `positional()` gives every positional value as a slice, as handlers get now, and
  `function_name()` and `signature()` give the function's name and the called overload.

### 6. The language doesn't change

This is `SimpleRuntime`'s Rust API, not part of the language. What a call passes, how it's checked
and which overload it reaches are as in ADR 013, and the spec is unchanged.

## Alternatives Considered

- **Index-based helpers, like `args.string_arg("read", 0)`.** Rejected: it removes the hand-written
  match, but keeps positions, which differ between overloads, and passes the function's name in
  when `Arguments` already knows it.
- **Keeping `new` and `with_keywords`, and adding a third constructor for `Arguments`.** Rejected:
  three ways to build a native function, two of them superseded. flt is still at 0.x, and its own
  built-ins and datu are the handlers to migrate.
- **Errors instead of panics for undeclared names.** Rejected: a misspelt name is a bug in the
  host, not in the flt source, and an error would reach the user as if their call were wrong.
  `value` covers the handlers that need to ask.
- **Using `TryFrom<&Value>` instead of a new trait.** Rejected: `Option<T>` can't implement it
  without overlapping the standard library's blanket `TryFrom`, and a trait of flt's own can be
  implemented for borrowed values like `&str`.
- **Handlers as typed Rust functions, like `fn read(path: String)`, with the signature taken from
  their parameters.** Not yet: that's a bigger change, and it can be built on `Arguments`, which
  would hold the values such an adapter unpacks.

## Consequences

- Breaking Rust API changes: handlers take `&Arguments` instead of `&[Value]` and `&Map`, and
  `NativeFunction::with_keywords` is gone. flt's built-ins, its tests and datu's handlers move to
  the new API.
- Hosts can drop helpers like datu's `string_arg`, and handlers stop indexing into their arguments.
- A handler's argument names have to match its signature's, and a mismatch panics on the first
  call that reaches it.
- `docs/native-functions-and-values.md`'s examples are rewritten for `Arguments`.

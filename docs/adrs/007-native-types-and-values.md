# ADR 007: Native Types and Values

- **Date:** 2026-10-25
- **Status:** Accepted.

## Context

Native functions ([ADR 003](003-functions-as-values.md)) take and return only flt's own values. A
host often needs to pass its own data between calls. In datu,

```flt
read("x.parquet") |> filter("a > 1") |> count()
```

`read` returns a lazy query plan, `filter` adds to it and `count` runs it. The plan is a Rust value
that only datu understands.

Translating such data into maps and arrays needs conversion code both ways, lets flt source forge
values, gives them no type of their own, and can't represent live resources such as an open file.

## Decisions

### 1. A native type is a Rust type that implements `NativeType`

```rust
impl NativeType for DatuPlan {
    const NAME: &'static str = "DatuPlan";
}
```

The trait requires `Display + PartialEq + Send + Sync + 'static`. The Rust type is the identity:
one Rust type, one native type, one name.

### 2. The host registers native types with the runtime

```rust
runtime.register_native_type::<DatuPlan>();
```

Registration comes before the functions that use the type and before any flt source is evaluated.
Native types belong to the runtime, not the global scope, which still holds only variables and
functions. The global scope records their names so it can protect them (decision 7).

A value of an unregistered native type still works, but its type's name doesn't resolve. That's a
host bug, like those in [ADR 006](006-no-shadowing.md) (decision 3); creating a value doesn't
involve the runtime, so nothing catches it.

### 3. Only the host creates and reads native values

Host code creates a native value with `Value::native(plan)` and reads it with
`value.downcast_native::<DatuPlan>()`. Handlers need no access to the runtime.

flt source can bind, pass, pipe, compare and print native values and get their type, but can't
create them, look inside them or change them:

```flt
let p = read("x.parquet")
p |> filter("a > 1")     # passed and piped like any value
DatuPlan(1)              # DatuPlan is not a function
p.stages                 # Invalid Operand Type
```

`<>` treats a native value as the empty string, as it does a function.

### 4. Native values print with the host's `Display`

Nothing is added, e.g. no `<DatuPlan …>` wrapper:

```flt
read("x.parquet") |> filter("a > 1")   # read("x.parquet") |> filter("a > 1")
```

### 5. Native values compare with the host's `PartialEq`

```flt
read("x") == read("x")    # true, if the host's PartialEq says so
read("x") == read("y")    # false
```

Values of different native types are never equal, nor is a native value equal to any other value.
Native values can't be ordered: `<`, `>`, `<=` and `>=` raise `Invalid Operand Type`.

### 6. A native type's name evaluates to that type

```flt
typeof(p)               # DatuPlan
typeof(p) == DatuPlan   # true
typeof(DatuPlan)        # Type
```

Lookup checks native types after variables, functions and built-in types. Since a global name means
only one thing ([ADR 006](006-no-shadowing.md), decision 4), the order doesn't change the result.

### 7. Native type names are protected

Following [ADR 006](006-no-shadowing.md) (decision 4), in `SimpleRuntime`:

- `let DatuPlan = 1`, or binding the name through the host API, raises
  `Cannot bind DatuPlan: it's a native type`;
- in the REPL, `/unset DatuPlan` is refused for the same reason, and `/inspect DatuPlan` reports
  `(native type) DatuPlan`;
- registering a function under a native type's name, or a native type under any name already in
  use, panics.

### 8. Function types can name native types

```flt
typeof(filter)          # fn(DatuPlan, String) -> DatuPlan
```

### 9. The runtime still checks only arity

A native type in a signature isn't enforced ([ADR 003](003-functions-as-values.md), decision 4).
Handlers check their arguments as they read them: `filter(1, "a > 1")` raises
`Expected DatuPlan, found Number` from `downcast_native`.

### 10. Host errors are shown unchanged

A native function can fail with the host's own error. It displays exactly as the host's error
does, with no `Runtime Error:` prefix, and the host can recover the original.

### 11. The concept is the language's; the Rust API is the runtime's

The spec says a runtime may let its host define native types, with decisions 3 to 6, 8 and 10.
The `NativeType` trait, registration and protected names belong to the reference runtime, as in
[ADR 006](006-no-shadowing.md) (decision 5).

## Alternatives Considered

- **A type name per value**, e.g. `NativeValue::new("DatuPlan", plan)`, printed as `<DatuPlan>`
  and compared by identity. Rejected: the name wasn't bound, so `typeof(p) == DatuPlan` failed,
  one name could cover different Rust types, and the host couldn't control printing or equality.
- **Translating host data into maps and arrays.** Rejected for the reasons in the Context.
- **Structs.** Rejected for this need: a much larger feature that still needs conversion, can still
  be forged from flt source, and can't hold live resources. Structs remain possible for flt data.
- **A typed handle from registration**, e.g. `NativeTypeHandle<DatuPlan>`. It would catch
  unregistered types at compile time, but every handler would capture a handle and the name would
  live apart from its type.
- **Registering native types in the global scope.** Rejected: types belong to the runtime.
- **Identity equality.** Rejected: equal data would compare unequal.
- **Wrapped display, e.g. `<DatuPlan …>`.** Rejected: printing is the host's choice.
- **Checking native arguments in the runtime.** Deferred: the runtime checks no argument types yet.

## Consequences

- New Rust API:
  - the `NativeType` trait;
  - `SimpleRuntime::register_native_type` and `get_native_type`;
  - `Type::Native` and `Type::native::<T>()`;
  - `Value::Native`, `Value::native`, `get_native` and `downcast_native`;
  - `Error::Native` and `Error::native`;
  - `From<u64 | i64 | usize>` for `Value`, and `TryFrom<&Value>` for `i32`, `i64`, `u32`, `u64`,
    `usize` and `f64`, so hosts needn't depend on `bigdecimal`.
- New errors:
  - `Expected <type>, found <type>`, also raised by `TryFrom<&Value>` for a non-number;
  - `Cannot convert <number> to <type>`, e.g. `2.5` or `-1` as a `u32`;
  - `Cannot bind <name>: it's a native type`.
- Breaking changes:
  - `Type`, `Value`, `Error`, `RuntimeError` and `ProtectedName` gain variants;
  - `SimpleRuntime` gains a private field, so build it with `SimpleRuntime::default()`.
- A macro, either `macro_rules!` or a derive, could later write the one-constant `NativeType` impl.
- `Type::Custom` stays unused.

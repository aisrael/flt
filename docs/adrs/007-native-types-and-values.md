# ADR 007: Native Types and Values

- **Date:** 2026-10-25
- **Status:** Accepted.

## Context

[ADR 003](003-functions-as-values.md) lets a host register native functions, but those functions
can only take and return flt's own values: numbers, strings, symbols, arrays, maps, types and
functions. A host often needs to pass its own data between calls. For example, datu evaluates

```flt
read("x.parquet") |> filter("a > 1") |> count()
```

where `read` returns a lazy query plan, `filter` adds a stage to it, and `count` runs it. The plan is
a Rust value that only datu understands, and it must travel through flt between the calls.

A host could translate its data into maps and arrays, but then it needs conversion code in both
directions, flt source can build a fake value by writing the same map by hand, the value has no type
of its own, and live resources such as an open file or a query engine can't be translated at all.

## Decisions

### 1. A native type is a host's Rust type that implements `NativeType`

A host makes one of its Rust types usable in flt by implementing the `NativeType` trait, which gives
the type's name in flt:

```rust
impl NativeType for DatuPlan {
    const NAME: &'static str = "DatuPlan";
}
```

The trait requires `Display + PartialEq + Send + Sync + 'static`, so every native type can be
printed and compared (decisions 4 and 5), and shared between threads like the rest of the runtime.
The Rust type is the native type's identity: one Rust type is one native type, with one name.

### 2. The host registers native types with the runtime

A host registers each native type with the runtime before it registers the functions that use it,
and before it evaluates any flt source:

```rust
runtime.register_native_type::<DatuPlan>();
```

Native types are bound in the runtime, not in the global scope. The global scope still holds only
variables and functions ([ADR 006](006-no-shadowing.md), decision 4), but it learns the names of
native types, so that it can protect them (decision 7).

Registering a native type is what makes its name resolve in flt source (decision 6). A value of a
type that was never registered still works, prints and compares, and `typeof` names its type, but
that name is unbound. That's a host bug, like the registration mistakes in
[ADR 006](006-no-shadowing.md) (decision 3). The runtime can't catch it when the value is created,
since creating a value doesn't involve the runtime.

### 3. Only the host creates and reads native values

A native value is a value of a native type. The host's Rust code creates it, e.g. with
`Value::native(plan)`, and reads it back as the Rust type, e.g. with
`value.downcast_native::<DatuPlan>()`. Native functions do this in their handlers; they capture
nothing and need no access to the runtime.

To flt source, a native value is opaque. It can be bound, passed to functions, piped, compared,
printed and given to `typeof`. It can't be created, looked inside or changed:

```flt
let p = read("x.parquet")
p |> filter("a > 1")     # passed and piped like any value
DatuPlan(1)              # DatuPlan is not a function
p.stages                 # Invalid Operand Type
```

As with function values, `<>` treats a native value as the empty string.

### 4. Native values print as the host displays them

A native value prints exactly as its `Display` implementation writes it, with nothing added. That's
how the REPL shows it:

```flt
read("x.parquet") |> filter("a > 1")   # read("x.parquet") |> filter("a > 1")
```

How a value prints is the host's choice, so flt doesn't wrap it, e.g. as `<DatuPlan …>`.

### 5. Native values compare with the host's equality

`==` and `!=` compare native values with the native type's `PartialEq`:

```flt
read("x") == read("x")    # true, if the host's PartialEq says the two plans are equal
read("x") == read("y")    # false
```

Values of different native types are never equal, and a native value never equals any other kind of
value. Native values can't be ordered: `<`, `>`, `<=` and `>=` raise `Invalid Operand Type`, as
they do for maps.

### 6. A native type's name evaluates to that type

Once registered, a native type's name evaluates to the type, which is a value like any other type
([ADR 001](001-none-types.md)):

```flt
typeof(p)               # DatuPlan
typeof(p) == DatuPlan   # true
typeof(DatuPlan)        # Type
```

Name lookup checks native types after variables, functions and built-in types. As
[ADR 006](006-no-shadowing.md) (decision 4) explains, the order doesn't change the result, since a
name in the global scope means only one thing.

### 7. Native type names are protected

[ADR 006](006-no-shadowing.md) expects every new kind of global name to be protected the same way
(decision 4). In the reference runtime, `SimpleRuntime`:

- a native type's name can't be bound: `let DatuPlan = 1` raises
  `Cannot bind DatuPlan: it's a native type`, and so does binding it through the host API;
- in the REPL, `/unset DatuPlan` reports `Cannot unset DatuPlan: it's a native type`, and
  `/inspect DatuPlan` reports `(native type) DatuPlan`;
- registering a function under a native type's name, or a native type under a built-in type name,
  a function's name, a variable's name or another native type's name, is a host bug and panics.

### 8. Function types can name native types

A native function's type is still an flt function type ([ADR 003](003-functions-as-values.md)).
Its parameters and its return type can be native types:

```flt
typeof(filter)          # fn(DatuPlan, String) -> DatuPlan
```

### 9. The runtime still checks only arity

The runtime checks the number of arguments to a native function, not their types
([ADR 003](003-functions-as-values.md), decision 4), and a native type in a signature doesn't
change that. A handler checks a native argument when it reads it: given any other value,
`downcast_native` raises `Expected DatuPlan, found Number`, so `filter(1, "a > 1")` fails with that
error.

### 10. Host errors are reported unchanged

A native function can fail with the host's own error. The runtime reports it exactly as the host's
error displays, with no `Runtime Error:` prefix, and the host can recover the original error from
it.

### 11. Native types are part of the language; the Rust API is the runtime's

The language allows a runtime to let its host define native types, with the behaviour in decisions
3 to 6, 8 and 10: opaque values that only the host creates and reads, that print and compare as
the host defines, and whose type names evaluate to types. The language spec describes that, without
the reference runtime's Rust API.

The `NativeType` trait, registration on `SimpleRuntime`, and protected names (decision 7) belong to
the reference runtime, as [ADR 006](006-no-shadowing.md) (decision 5) treats its own rules.

## Alternatives Considered

- **Opaque values with a type name given per value**, e.g. `NativeValue::new("DatuPlan", plan)`,
  printed as `<DatuPlan>` and compared by identity. Rejected. The type name wasn't bound, so
  `typeof(p) == DatuPlan` raised `Unbound identifier: DatuPlan`, nothing stopped two values from
  using one name for different Rust types, and the host had no say in how values print or compare.
- **Translating host data into maps and arrays.** Rejected. It needs conversion code in both
  directions, flt source can forge a value by writing the map by hand, the value has no type of its
  own, and live resources can't be translated.
- **Structs.** A struct, a value with a named type and fixed fields, would give host data a type of
  its own. Rejected for this need: it's a much larger language feature, the host would still convert
  its data into fields, flt source could still build a struct by hand, and live resources still
  couldn't be held. Structs remain a possible feature for data written in flt.
- **A typed handle returned by registration**, e.g. `NativeTypeHandle<DatuPlan>`, through which the
  host creates and reads values. It would guarantee at compile time that every value's type is
  registered. Rejected: every handler would capture a handle, and the type's name would live apart
  from the type it names. The trait keeps the name with the type, and an unregistered type is a host
  bug (decision 2).
- **Registering native types in the global scope.** Rejected. The global scope holds variables and
  functions; types belong to the runtime.
- **Comparing native values by identity.** Rejected. Two values holding the same data would be
  unequal, so the REPL couldn't usefully compare them.
- **Wrapping printed values, e.g. `<DatuPlan …>`.** Rejected. The host decides how its values print.
- **Checking native arguments in the runtime.** Not now. The runtime checks no argument types yet,
  and checking only native ones would be a special case. Handlers check them with one call.

## Consequences

- The Rust API gains:
  - the `NativeType` trait;
  - `SimpleRuntime::register_native_type` and `get_native_type`;
  - `Type::native::<T>()` and `Type::Native`;
  - `Value::Native`, `Value::native`, `get_native` and `downcast_native`;
  - `Error::Native` and `Error::native`, for host errors;
  - `From<u64>`, `From<i64>` and `From<usize>` for `Value`, and `TryFrom<&Value>` for `i32`, `i64`,
    `u32`, `u64`, `usize` and `f64`, so hosts can return and read numbers without depending on
    `bigdecimal`. `TryFrom` borrows the value, since handlers get `&[Value]`.
- **Breaking changes:**
  - `Type`, `Value`, `Error`, `RuntimeError` and `ProtectedName` gain variants, so hosts that match
    on them exhaustively need a new arm;
  - `SimpleRuntime` gains a private field, so it can't be built with a struct literal; use
    `SimpleRuntime::default()`.
- New errors: `Expected <type>, found <type>` (also raised by `TryFrom<&Value>` for a value that
  isn't a number), `Cannot convert <number> to <type>` (a number that doesn't fit the Rust type,
  e.g. `2.5` or `-1` as a `u32`), and `Cannot bind <name>: it's a native type`.
- The trait impl a host writes is one constant. A macro, either a `macro_rules!` macro in flt or a
  derive in a separate proc-macro crate, could write it later without changing any behaviour.
- `Type::Custom` stays unused.

# ADR 012: Variadic Notation

- **Date:** 2026-10-27
- **Status:** Proposed.

## Context

[ADR 011](011-variadic-arguments.md) (decision 3) shows a variadic parameter as `...T` in function
types, as in TypeScript, e.g. `fn(String, ...Any) -> String`.
[ADR 010](010-keyword-arguments.md) shows rest keywords as `**T`, as in Python, e.g.
`fn(Any, **Any) -> Any`, and leaves the `**` open to change. So one function type can mix the two
styles:

```flt
typeof(select)   # fn(Any, ...Any, **Any) -> Any
```

Signatures that name their arguments make the mismatch worse. The Cucumber features describe
each native function they use by its signature, which in ADR 011's notation would be
`sprintf(format: String, ...args: Any) -> String`, and the docs will do the same.

## Decisions

### 1. A variadic parameter shows as `*T`

```flt
typeof(count_args)   # fn(*Any) -> Number
typeof(sprintf)      # fn(String, *Any) -> String
typeof(select)       # fn(Any, *Any, **Any) -> Any
```

This replaces ADR 011's `...T`. `T` is still the type of each value the parameter takes, not of a
collection.

### 2. `*` and `**` are one convention

Rest keywords keep `**T` (ADR 010), which this ADR settles rather than leaving open. A variadic
parameter takes the rest of the positional arguments and rest keywords take the rest of the
keywords, so `*` and `**` mark them, as in Python and Ruby.

### 3. Signatures with argument names

Where a signature names its arguments, each positional argument is `name: T`, a variadic one is
`*name: T`, a keyword is `name: T = default`, and rest keywords are `**kwargs: T`:

```
sprintf(format: String, *args: Any) -> String
greet(name: String, greeting: String = "Hello") -> String
select(plan: Any, *columns: Any, **kwargs: Any) -> Any
```

Rest keywords have no name of their own, so they're always `kwargs`, as in Python's `**kwargs`.
A variadic argument keeps the name its host gives it.

Leaving out the positional names, `kwargs` and the defaults gives the function's type: `sprintf`'s
is `fn(String, *Any) -> String`, and `select`'s is `fn(Any, *Any, **Any) -> Any`.

### 4. It's notation, not syntax

flt source can't write signatures or types yet, so this decides only how runtimes print function
types, and how docs and tests write signatures. A later syntax for signatures in flt source starts
from it.

## Alternatives Considered

- **Keeping `...T`.** Rejected: it's familiar from TypeScript, but it mixes with `**T`, and
  changing `**T` to something TypeScript-like would leave rest keywords without a familiar form.
- **`args: ...T`.** Rejected: it keeps every argument as `name: type`, but no other language
  writes it.
- **`*` meaning multiplication.** Not a conflict: signatures and types aren't expressions. If flt
  later adds call-site splats (`f(*list)`), they'd take `*` and `**` in calls, as in Python.

## Consequences

- Breaking change: function types print `*T` instead of `...T`, so `typeof(sprintf)` prints
  `fn(String, *Any) -> String`. Hosts or tests that compare those strings need updating.
- The Rust API doesn't change: `Argument::variadic` and `FunctionType`'s `variadic` field stay.
- The docs and Cucumber features write variadic parameters as `*T`, and named ones as `*name: T`.

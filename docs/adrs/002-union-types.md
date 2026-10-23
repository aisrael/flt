# ADR 002: Union Types

- **Date:** 2026-10-22
- **Status:** Accepted and implemented.
- **PR:** [#16](https://github.com/aisrael/flt/pull/16)

## Context

flt needs a type that means "either of these". The main drivers, none implemented yet, are:

- function overloading (decision 4);
- type inference for `if` (decision 5);
- optional parameters, where `T?` is shorthand for `T | None`.

## Decisions

### 1. Union types: `A | B`

A value of type `A | B` is a value of either type.

Unions are always simplified:

- nested unions are flattened;
- duplicates are removed, keeping the first occurrence;
- a union with a single member is that type, so `Number | Number` is `Number`.

### 2. Function type syntax

A function type is written `fn(P1, P2) -> R`. `fn() -> T` takes no parameters.

`->` groups before `|`, so a union return type needs brackets:

- `fn(T) -> (T | None)` is a function that returns `T` or `None`;
- `fn(T) -> T | None` is either a function or `None`.

### 3. Types are checked at run time

flt is dynamically typed: `typeof(x)` returns the type of the value `x` holds when it runs.

Inferred types are left to future static analysis, for example an flt language server. After
`let result = if flip_a_coin() 0 else "zero"`:

- at run time, `typeof(result)` is `Number` or `String`, depending on the branch taken;
- a static analyzer would annotate `result` as `Number | String`.

### 4. Function overloading is deferred

When overloading is added, the type of an overloaded function will be the union of its signatures,
e.g. `fn(String) -> String | fn(String, Number) -> String`. It is out of scope for now.

### 5. Type inference for `if` is deferred

When static type inference is added, an `if` whose branches have types `T` and `F` will have the
inferred type `T | F`. An `if` with no `else` evaluates to `None` when its condition is `false` (see
[ADR 001](001-none-types.md)), so its inferred type is `T | None`. Until then, types are only
checked at run time (decision 3).

### 6. Staging

1. **This stage:** `None` and union types, then types as values and `Any`.
2. **Next:** function definitions and function bodies.
3. **Later:** static type inference and parametric polymorphism (generics), so that
   `random_choice(a: A, b: B)` returns `A | B`.

## Alternatives Considered

- **Make `->` bind more loosely than `|`**, or **use a different symbol for unions of function
  types.** Both were superseded by requiring brackets around a union return type, which keeps `|`
  as the only union operator.
- **Infer `typeof` statically.** Rejected. flt is dynamically typed, and static inference belongs to
  a later stage.

## Consequences

- `Type::Union` is added. It is built with `Type::union`, which keeps unions simplified.
- The spec (`docs/spec.md`) and the type docs (`docs/types/union_types.md`,
  `docs/types/functions.md`) describe these rules.

# ADR 001: `None` Types

- **Date:** 2026-10-22
- **Status:** Accepted. `None` is implemented; types as values are documented but not yet
  implemented.
- **PR:** [#16](https://github.com/aisrael/flt/pull/16)

## Context

flt had a `Unit` type, written `()`, for the result of an `if` with no `else` whose condition is
`false`. It also had a separate `None` value for the empty `Option`. Two "empty" values were one
too many, and the type of an `if` without `else` (see [ADR 002](002-union-types.md)) needs a single
type for the empty case.

## Decisions

### 1. `None` replaces `()` / `Unit`

An `if` with no `else` evaluates to `None` when its condition is `false`, and `typeof(None)` is
`None`. `Unit` and `()` are removed.

`Option<T>` stays a separate concept, a wrapper type, even though its empty case is also `None`.

Tuples, including the empty tuple `()`, are not part of this design.

### 2. Types are values

`typeof` has type `fn(Any) -> Type` and returns a type, not a string. Types print without quotes.

- A built-in type name evaluates to that type, so `typeof(1) == Number` is `true`.
- `typeof(Number)` is `Type`.
- The `None` value equals the `None` type, so `None == typeof(None)` is `true`.

### 3. `Any` is a placeholder, not a type of values

`Any` is the implicit type of any variable, parameter or return type declared without one, so
`def foo(a, b)` has type `fn(Any, Any) -> Any`.

`Any` appears only in signatures. No expression evaluates to `Any`, and `Any` used as a value is an
unbound identifier.

### 4. Bindings are global

A variable is a binding of an identifier: to a value at run time, or to an expression for static
analysis.

- There are no scopes yet, so every binding is global.
- Re-binding replaces the binding and its type, both at run time and for inference.
- A binding shadows a built-in type name: after `let Number = 5`, `typeof(Number)` is `Number`, not
  `Type`.

### 5. Docs first

New language features start with documentation and examples under `docs/`. Cucumber features and
unit tests that match those examples come next, and the implementation comes last. This rule is
recorded in `AGENTS.md`.

### 6. Version 0.2.0

Replacing `()` with `None` is a breaking change, so the next release is 0.2.0.

## Alternatives Considered

- **Keep `()` as the unit type, or bring it back as the empty tuple.** Rejected. One empty value
  (`None`) is simpler, and tuples are out of scope.
- **Have `typeof` return a string.** Rejected. Returning a `Type` value lets it be compared with a
  type name, as in `typeof(1) == Number`.

## Consequences

- Code and tests that expected `()` now get `None`.
- `Value::Unit` and `BuiltinType::Unit` are gone; the type is `BuiltinType::None`.
- Identifier lookup must fall back to built-in type names after variables. Global functions will
  take a step between the two.
- The spec (`docs/spec.md`) and the type docs (`docs/types.md`, `docs/types/none.md`) describe these
  rules.

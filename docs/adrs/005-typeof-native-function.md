# ADR 005: `typeof` as a Native Function

- **Date:** 2026-10-24
- **Status:** Accepted and implemented.
- **PR:** [#21](https://github.com/aisrael/flt/pull/21)

## Context

[ADR 003](003-functions-as-values.md) made functions values in the global scope, but left `typeof`
in the runtime's private table of built-ins. Calls fell back to that table when a name wasn't
bound, so `typeof(1)` and `1 |> typeof` worked, but `typeof` on its own was an unbound identifier:

- `typeof(typeof)` raised `Unbound identifier: typeof`, though `typeof` has the type
  `fn(Any) -> Type`;
- `let t = typeof` failed, so `typeof` couldn't be bound to another name or piped into through one.

## Decisions

### 1. `typeof` is a native function in the global scope

The runtime registers `typeof` in the global scope by default, as a native function of type
`fn(Any) -> Type`. It's a function value like any other:

```flt
typeof               # <fn typeof>
typeof(typeof)       # fn(Any) -> Type
let t = typeof
t("hi")              # String
1 |> t               # Number
```

### 2. Built-ins are ordinary native functions

There's no separate table of built-ins, and no fallback when a call's name isn't bound. A built-in
resolves like any other name ([ADR 003](003-functions-as-values.md), decision 3):

- a variable shadows it: after `let typeof = 1`, `typeof` is `1`, and `typeof(2)` raises
  `typeof is not a function`;
- removing the variable, e.g. with `/unset typeof` in the REPL, makes the built-in visible again;
- a host that registers its own function named `typeof` replaces the built-in.

### 3. The runtime's arity check covers built-ins

`typeof` relies on the runtime's arity check ([ADR 003](003-functions-as-values.md), decision 4),
like other native functions, and doesn't check its arguments itself.

## Alternatives Considered

- **Keep the private table, and look it up in identifier lookup too.** Rejected. Built-ins would
  still be a second kind of function, with their own lookup step and their own rules for shadowing
  and replacement.
- **Make built-ins read-only, so a host can't replace them.** Rejected. A variable can already
  shadow `typeof`, and letting a host replace it keeps one rule for every function in the global
  scope.

## Consequences

- This supersedes ADR 003's consequence that `typeof` stays a private built-in.
- `SimpleRuntime.built_in_functions` is removed. It's a public field, so this changes the Rust API
  for hosts that read it. `typeof` can be read from the global scope instead.
- Calling an unbound name raises `Undefined function: <name>` directly.
- The REPL's `/inspect` reports a function's type, e.g. `/inspect typeof` prints
  `(function) fn(Any) -> Type`.

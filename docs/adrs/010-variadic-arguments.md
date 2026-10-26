# ADR 010: Variadic Arguments

- **Date:** 2026-10-26
- **Status:** Proposed.

## Context

Since [ADR 003](003-functions-as-values.md) (decision 4), a call passes the arity check only if
some overload of the function has exactly that many arguments. A host can't register a function
that takes any number of arguments. In datu,

```flt
read("x.parquet") |> select(:id, :name, :email)
read("x.parquet") |> group_by(:region, :year)
```

`select` and `group_by` take as many columns as the user names. Overloads for one, two, three
columns and so on would be guesswork, and each would show up in the function's type.

## Decisions

### 1. The last argument of a signature can be variadic

```rust
FunctionDefinition::new(
    "select",
    Type::any(),
    vec![Argument::new("plan", Type::any()), Argument::variadic("columns", Type::any())],
)
```

A variadic argument takes zero or more values: `select(plan)`, `select(plan, :a)` and
`select(plan, :a, :b)` all reach the handler, which receives every value as one flat `&[Value]`.
Whether no columns is an error is up to the handler.

Only the last argument can be variadic. A variadic argument anywhere else is a host bug, so
`FunctionDefinition::new` and `add_overload` panic on it.

### 2. Arity with a variadic argument

A signature with `k` arguments before its variadic one accepts any call with at least `k`
arguments. As before, a call passes if any overload accepts it. Otherwise, if the first overload
is variadic, the error says how many arguments it needs at least:

```flt
count_args()     # Function count_args expected at least 1 argument(s), found 0
```

A function without a variadic overload raises `Function <name> expected <expected> argument(s),
found <found>`, as before. A piped value counts as an argument, as in
[ADR 004](004-pipe-operator.md), so `1 |> count_args` passes.

### 3. Variadic parameters show as `...T` in function types

```flt
typeof(count_args)   # fn(Any, ...Any) -> Number
```

The type of a variadic argument is the type of each value it takes, not of a collection. As
elsewhere, it isn't checked ([ADR 003](003-functions-as-values.md), decision 4).

### 4. Keyword arguments are left to a later decision

This ADR doesn't change keyword arguments. The parser still passes them as one trailing map
([ADR 004](004-pipe-operator.md)) with symbol keys
([ADR 009](009-symbol-keys-and-ordered-maps.md)), which a variadic argument would take as its last
value. That isn't part of this decision and isn't documented or tested. A separate decision is expected to give signatures their own keyword arguments, after
which a variadic argument would take only positional values.

### 5. The concept is the language's; the Rust API is the runtime's

The spec says a function may take a variable number of trailing arguments (decisions 1 to 3).
`Argument::variadic` and the panic on a misplaced variadic argument belong to the reference
runtime, as in [ADR 006](006-no-shadowing.md) (decision 5).

## Alternatives Considered

- **Overloads for each count.** Rejected: the maximum is guesswork, and every overload clutters
  the function's type.
- **Passing variadic values as one array**, e.g. `select(plan, [:a, :b])`. Rejected: it changes
  how users call the function, and `1 |> f(2, 3)` would need the array built by hand.
- **Variadic arguments in any position.** Rejected: matching arguments to parameters becomes
  ambiguous, and no function needs it yet.
- **Returning an error from `FunctionDefinition::new` instead of panicking.** Rejected for now:
  the definition is built in host code, and a misplaced variadic argument is a mistake in that
  code, not in flt source.

## Consequences

- New Rust API:
  - `Argument::variadic` and the `Argument::variadic` field;
  - `FunctionType`'s `variadic` field and a constructor for variadic function types.
    `FunctionType::new` keeps its signature.
- New error: `Function <name> expected at least <minimum> argument(s), found <found>`.
- Breaking changes:
  - `Argument` and `FunctionType` gain public fields, so hosts that build them with struct
    literals break;
  - `RuntimeError` gains a variant.

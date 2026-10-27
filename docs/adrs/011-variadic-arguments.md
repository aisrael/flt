# ADR 011: Variadic Arguments

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

### 1. The last positional argument of a signature can be variadic

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

Only the last positional argument can be variadic. A variadic argument anywhere else is a host
bug, so `FunctionDefinition::new` and `add_overload` panic on it. Keyword arguments are declared
separately ([ADR 010](010-keyword-arguments.md)), so they don't count: a function can have both.

### 2. Arity with a variadic argument

A variadic argument takes zero or more values, so only the arguments before it are required. A
signature with `k` arguments before its variadic one accepts any call with at least `k`
arguments:

```rust
// k = 0: count_args takes only a variadic argument
FunctionDefinition::new("count_args", Type::number(), vec![Argument::variadic("args", Type::any())])
// k = 1: sprintf takes a format before its variadic argument
FunctionDefinition::new(
    "sprintf",
    Type::string(),
    vec![Argument::string("format"), Argument::variadic("args", Type::any())],
)
```

```flt
count_args()         # passes: args is empty
count_args(1, 2)     # passes
sprintf("x")         # passes: args is empty
sprintf("%d", 1)     # passes
sprintf()            # Function sprintf expected at least 1 argument(s), found 0
```

As before, a call passes if any overload accepts it. Otherwise, if the first overload is variadic,
the error says how many arguments it needs at least. A function without a variadic overload raises
`Function <name> expected <expected> argument(s), found <found>`, as before. A piped value counts
as an argument, as in [ADR 004](004-pipe-operator.md), so `"%d" |> sprintf(1)` passes.

### 3. Variadic parameters show as `...T` in function types

```flt
typeof(count_args)   # fn(...Any) -> Number
typeof(sprintf)      # fn(String, ...Any) -> String
```

The type of a variadic argument is the type of each value it takes, not of a collection. As
elsewhere, it isn't checked ([ADR 003](003-functions-as-values.md), decision 4).

### 4. A variadic argument takes only positional values

Keyword arguments are passed separately from positional ones
([ADR 010](010-keyword-arguments.md)), so a variadic argument never receives them. A call's
keywords are matched against the keywords its signature declares, as for any function, and an
undeclared one is an error:

```flt
count_args(1, 2, k: 3)   # Function count_args got an unexpected keyword argument: k
```

A signature can declare keywords alongside a variadic argument, and a call can pass keywords with
no variadic values, as in datu's `select(plan, total: sum(:b))`. That's one reason a variadic
argument takes zero or more values rather than one or more.

### 5. The concept is the language's; the Rust API is the runtime's

The spec says a function may take a variable number of trailing arguments (decisions 1 to 4).
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

# ADR 013: Type-Checked Function Calls

- **Date:** 2026-10-27
- **Status:** Proposed.

## Context

A function's signatures declare the types of its arguments, keywords and result, and those types
show in the function's type, but a call checks only the number of arguments:

- **Types aren't checked.** Nothing enforces the declared types
  ([ADR 003](003-functions-as-values.md), decision 4; [ADR 010](010-keyword-arguments.md),
  decision 6). Each function checks its own arguments, and nothing checks what it returns, so
  `typeof` can show a return type the function doesn't keep to.
- **The overload is picked by argument count alone.** A call passes if some overload takes that
  many positional arguments (ADR 003, decision 4), the first such overload in declaration order is
  the one whose keywords are checked (ADR 010, decision 5), and a variadic overload takes any count
  from its minimum up ([ADR 011](011-variadic-arguments.md)). How a variadic overload combines
  with fixed ones was left open.

Checking types can't keep the second rule. datu's `count` has two forms with one argument each:

```
count(plan: DatuPlan) -> Number
count(column: Any) -> DatuSelectItem
```

`count(plan)` runs a query and returns its row count, and `count(:id)` builds an aggregate for
`select`. They're two functions that share a name. Since both take one argument, the first is
always the one picked, and one handler serves both by looking at its argument.

Checking types against the overload picked by count would break `count`: `count(:id)` would be
checked against `count(plan: DatuPlan)` and fail, though the second overload accepts it. So a
type-checked call has to let the types pick the overload.

## Decisions

### 1. Every call is type-checked

Before a function runs, the runtime checks each positional argument, each variadic value and each
keyword value against the type its signature declares:

```
pad(s: String, width: Number) -> String
total(*values: Number) -> Number
greet(name: String, greeting: String = "Hello") -> String
```

```flt
pad("x", 3)                  # passes
pad(1, 3)                    # Argument s of pad expects String, found Number
total(1, "a")                # Argument values of total expects Number, found String
greet("Ann", greeting: 1)    # Keyword argument greeting of greet expects String, found Number
```

`Any` accepts every value, so a function declared with `Any`, like `typeof`, behaves as before.

### 2. What a type accepts

- `Any`: every value.
- A built-in type (`Number`, `String`, `Boolean`, `Symbol`, `Array`, `Map`, `Type`, `None`):
  values of that type. The `None` type accepts the `None` value ([ADR 001](001-none-types.md)).
- A union `A | B`: the values any of its members accepts ([ADR 002](002-union-types.md)).
- `Option<T>`: `None`, and the values `T` accepts.
- A native type: native values of that type ([ADR 007](007-native-types-and-values.md)). Two
  native types with the same name are still different types.
- A function type: a function whose type equals it, or one of whose overloads' types does. There's
  no subtyping, so a `fn(Any) -> Any` parameter doesn't accept a `fn(Number) -> Number`.

### 3. Overloads are tried in declaration order, and the first match wins

An overload matches a call if it takes that many positional arguments and accepts each of them.
The overloads are tried in the order they were declared, like Elixir's function clauses, and the
first that matches is the one called:

```
count(plan: DatuPlan) -> Number
count(column: Symbol | String) -> DatuSelectItem
```

```flt
read("x.csv") |> count()     # the first overload
count(:id)                   # the second
count(42)                    # No overload of count accepts (Number)
```

This settles how fixed and variadic overloads combine (ADR 011): the more specific one is
declared first. A piped value counts as the first argument, as before
([ADR 004](004-pipe-operator.md)), so `1 |> describe` picks by the type of `1`.

### 4. Keywords don't pick the overload

The overload is picked by its positional arguments, then the call's keywords are checked against
that overload's keywords, as in ADR 010. A keyword value of the wrong type is an error, not a
reason to try the next overload. Rest keywords are checked against their type, so
`**kwargs: Number` accepts only numbers ([ADR 012](012-variadic-notation.md)).

### 5. The return value is checked

The function's result must match the return type of the overload that was called:

```flt
liar()     # where liar() -> Number returns a string:
           # Function liar returned String, expected Number
```

A function's type is what `typeof` shows, so the return type has to be true. A mismatch is a bug in
the function, not in the call, but it's reported where it happens rather than passed on.

### 6. Errors

- No overload takes that many positional arguments: unchanged,
  `Function <name> expected <expected> argument(s), found <found>`, or
  `Function <name> expected at least <minimum> argument(s), found <found>` when the first overload
  is variadic.
- One overload takes that many, but an argument doesn't match:
  `Argument <argument> of <name> expects <type>, found <type>`, naming the first argument that
  fails. A variadic value names the variadic argument.
- Several overloads take that many, and none matches:
  `No overload of <name> accepts (<type>, <type>, …)`, listing the call's positional argument
  types. The overloads aren't listed; `typeof` shows them.
- A keyword value doesn't match: `Keyword argument <key> of <name> expects <type>, found <type>`.
- The return value doesn't match: `Function <name> returned <type>, expected <type>`.

### 7. A function's type doesn't change

It's still the union of its overloads' function types (ADR 003, decision 2):

```flt
typeof(count)    # fn(DatuPlan) -> Number | fn(Symbol | String) -> DatuSelectItem
```

### 8. The rules are the language's; the Rust API is the runtime's

The spec says calls are checked and overloads picked as in decisions 1 to 6. In `SimpleRuntime`:

- overloads keep sharing one handler, which gets the arguments of whichever overload matched;
- `FunctionDefinition::resolve` takes the call's positional values instead of their count, and
  returns the matching signature. `check_arity` still checks the count alone;
- `FunctionDefinition::keyword` panics if a keyword's default doesn't match its type, as
  `FunctionDefinition::new` panics on a misplaced variadic argument (ADR 011). It's a bug in host
  code, found when the function is defined.

A handler for each overload, and signatures taken from typed Rust functions, are expected to
follow.

## Alternatives Considered

- **Checking types against the overload picked by count.** Rejected: it breaks functions whose
  overloads share a count, like `count`.
- **Picking the most specific overload.** Rejected for now: "most specific" needs rules for unions,
  `Any` and variadic arguments, and declaration order is simpler to predict. A warning for an
  overload that an earlier one hides could come later.
- **Checking types only when a function has one overload.** Rejected: it leaves overloaded
  functions unchecked, which are the ones where checks help most.
- **Letting keywords pick the overload.** Rejected: an overload would be chosen by arguments that
  may be left out, and ADR 010 already picks by positional arguments.
- **Not checking return values, or only in debug builds.** Rejected: a wrong return type is
  otherwise silent, and `typeof` would show something untrue.

## Consequences

- Calls whose argument types don't match their declared types now fail before the function runs,
  and so do functions whose results don't match their return types. Hosts need declared types
  that match what their handlers accept and return; `Any` keeps the old behaviour.
- New errors, as in decision 6, and new `RuntimeError` variants for them.
- Breaking Rust API changes: `FunctionDefinition::resolve` takes `&[Value]`, and
  `FunctionDefinition::keyword` panics on a default of the wrong type.
- Handlers can stop checking their own argument types, though checks they keep aren't wrong.

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

### 3. Overloads are tried by arity, longest first

An overload matches a call if it takes that many positional arguments and accepts each of them.
An overload's arity is the number of its positional arguments before any variadic one. The
overloads are tried in order of arity, longest first, and the first that matches is the one
called. So a call picks the overload that matches the most of its arguments one for one, whatever
order the overloads were declared in:

```
foo(*args: Any) -> String            # arity 0, declared first
foo(a: Number, *args: Any) -> String # arity 1
```

```flt
foo(1, :b)       # the second: arity 1 is tried before arity 0
foo(:b)          # the first: the second doesn't accept :b as a Number
foo()            # the first
```

This settles how fixed and variadic overloads combine (ADR 011). Overloads with different types
and the same arity pick by type:

```
count(plan: DatuPlan) -> Number
count(column: Symbol | String) -> DatuSelectItem
```

```flt
read("x.csv") |> count()     # count(plan)
count(:id)                   # count(column)
count(42)                    # No overload of count accepts (Number)
```

A piped value counts as the first argument, as before ([ADR 004](004-pipe-operator.md)), so
`1 |> describe` picks by the type of `1`.

### 4. Ties between overloads of the same arity aren't specified

If several overloads of the same arity all accept a call's arguments, the spec doesn't say which
one is called, and flt source shouldn't depend on it:

```
show(x: Number) -> String
show(x: Any) -> String
```

```flt
show(1)          # both accept 1, so either may be called
```

`SimpleRuntime` calls the one declared first, for now (decision 9). Nothing else about which
overload is called depends on the order overloads were declared in.

### 5. Keywords don't pick the overload

The overload is picked by its positional arguments, then the call's keywords are checked against
that overload's keywords, as in ADR 010. A keyword value of the wrong type is an error, not a
reason to try the next overload. Rest keywords are checked against their type, so
`**kwargs: Number` accepts only numbers ([ADR 012](012-variadic-notation.md)).

### 6. The return value is checked

The function's result must match the return type of the overload that was called:

```flt
liar()     # where liar() -> Number returns a string:
           # Function liar returned String, expected Number
```

A function's type is what `typeof` shows, so the return type has to be true. A mismatch is a bug in
the function, not in the call, but it's reported where it happens rather than passed on.

### 7. Errors

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

### 8. A function's type doesn't change

It's still the union of its overloads' function types (ADR 003, decision 2):

```flt
typeof(count)    # fn(DatuPlan) -> Number | fn(Symbol | String) -> DatuSelectItem
```

### 9. The rules are the language's; the Rust API is the runtime's

The spec says calls are checked and overloads picked as in decisions 1 to 7. In `SimpleRuntime`:

- among overloads of the same arity that all accept a call (decision 4), the one declared first is
  called. This is a tie-breaker for now, and the only part of picking an overload that depends on
  declaration order;
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
- **Trying overloads in declaration order**, like Elixir's function clauses. Rejected: which
  overload a call reaches would depend on how the host happened to order its definitions, and a
  more general overload declared first would hide a more specific one.
- **Picking the most specific overload by type.** Rejected for now: "most specific" needs rules for
  unions and `Any`. Arity is a simpler measure that covers fixed and variadic overloads. Ties of
  the same arity may get such a rule later, replacing `SimpleRuntime`'s tie-breaker.
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
- New errors, as in decision 7, and new `RuntimeError` variants for them.
- Breaking Rust API changes: `FunctionDefinition::resolve` takes `&[Value]`, and
  `FunctionDefinition::keyword` panics on a default of the wrong type.
- Handlers can stop checking their own argument types, though checks they keep aren't wrong.

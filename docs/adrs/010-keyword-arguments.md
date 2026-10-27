# ADR 010: Keyword Arguments

- **Date:** 2026-10-26
- **Status:** Proposed.

## Context

Keyword arguments are only syntax. The parser collects the `key: value` pairs of a call into one
map and passes it as the call's last argument ([ADR 004](004-pipe-operator.md)), with symbol keys
([ADR 009](009-symbol-keys-and-ordered-maps.md)). So:

- `f(1, k: 2)` and `f(1, {k: 2})` are the same call. A function can't tell keywords from a map
  passed positionally, such as a map variable.
- The map counts as an argument in the arity check ([ADR 003](003-functions-as-values.md),
  decision 4), so a function that takes one argument and an optional keyword needs an overload
  for each.
- A function's signature says nothing about keywords. A function that doesn't expect any, such
  as `typeof`, still gets the map, and a misspelled keyword passes unnoticed.

Hosts want keywords for options and names the user picks. In datu,

```flt
read("x.parquet") |> select(:id, total: sum(:amount))
```

`total` names an output column.

## Decisions

### 1. Keyword arguments are separate from positional ones

A call's keyword arguments are kept apart from its positional arguments, all the way to the
function. `f(k: 1)` passes the keyword `k`, and `f({k: 1})` a map as its first positional argument.

The call syntax doesn't change. Positional arguments come first, and after the first keyword
argument no positional argument may follow. A keyword follows map-key rules, so it's a symbol,
written bare or quoted: `f(total: 1)` and `f("total": 1)` both pass the keyword `:total`.

### 2. A function declares the keywords it accepts

Each keyword argument a function declares has a name, a type and a default:

```flt
greet("Ann")                  # "Hello, Ann"
greet("Ann", greeting: "Hi")  # "Hi, Ann"
```

- **Every keyword argument is optional.** A call that leaves one out gets its default.
- **Keyword arguments are keyword-only.** A positional argument never fills one, so
  `greet("Ann", "Hi")` fails the arity check.

A function can also accept any other keywords, as **rest keywords**, like Python's `**kwargs`.
That suits keywords that name things, such as datu's output columns.

### 3. An undeclared keyword is an error

```flt
greet("Ann", tone: "warm")    # Function greet got an unexpected keyword argument: tone
typeof(1, k: 2)               # Function typeof got an unexpected keyword argument: k
```

This applies to a function that declares no keywords, too. A function that takes any keywords
declares rest keywords.

### 4. A duplicate keyword is a syntax error

`f(n: 1, n: 2)` doesn't parse, and neither does `f(n: 1, "n": 2)`, since both name the keyword
`:n`. The error names both the keyword and the call: `Duplicate keyword argument n in the call to
f`. Map literals don't change: `{n: 1, n: 2}` is `{n: 2}` (ADR 009), so `f({n: 1, n: 2})` is
fine.

### 5. Calls and pipes

- **Arity counts positional arguments only.** As before, a call passes if some overload of the
  function takes that many. The keywords are then checked against that overload's keywords. Each
  overload declares its own.
- **A pipe passes its value positionally.** `x |> f(a, k: v)` calls `f` with `x` and `a`
  positionally and `k` by keyword. The keywords no longer have to stay last.
- **Evaluation order:** the positional arguments left to right, then the keyword values left to
  right. With a pipe, the piped value comes first, as before.

### 6. Keyword values aren't type-checked yet

A keyword declares a type, but nothing checks it, as for positional arguments (ADR 003). flt
source has no way to write a keyword's type yet. A syntax for typed keyword arguments is expected
to come with type checking.

### 7. Function types show keywords by name

```flt
typeof(greet)                 # fn(String, greeting: String) -> String
```

Keywords follow the positional parameters. Defaults aren't part of the type. Rest keywords show as
`**T`, e.g. `fn(Any, **Any) -> Any`; the `**` may change with the syntax for typed keywords
(decision 6). Two function types are equal only if their keywords are too.

### 8. In `SimpleRuntime`, a handler gets every keyword in a map

A native function that takes keywords gets them as a `Map` with symbol keys, separately from its
positional arguments. The map holds every declared keyword, with the call's value or else its
default, in declaration order. Rest keywords follow, in call order.

### 9. The concept is the language's; the Rust API is the runtime's

The spec says how a call passes keyword arguments and how they're matched to the keywords a
function declares (decisions 1 to 7). How a host declares them, and how its handler receives them
(decision 8), belong to each runtime, like the rules of [ADR 006](006-no-shadowing.md) (decision 5).

## Alternatives Considered

- **Keeping the trailing map.** Rejected: a function can't tell keywords from a map passed
  positionally, and every function must expect a map it may not want.
- **Keywords that can also be passed by position**, as in Python. Rejected for now: matching
  becomes harder to explain, and no function needs it yet.
- **Required keyword arguments.** Deferred: optional ones cover what hosts need so far.
- **Ignoring undeclared keywords.** Rejected: a misspelled keyword would pass unnoticed.
- **Keeping the last of duplicate keywords**, as a map literal does. Rejected: a repeated keyword
  in a call is almost always a mistake, and nothing is lost by refusing it.
- **Passing the map as the handler's last argument**, keeping one handler signature. Rejected: the
  handler couldn't tell a keyword map from a positional one, which is the problem decision 1
  solves.

## Consequences

- New Rust API:
  - `FunctionDefinition::keyword(name, type, default)` and `FunctionDefinition::rest_keywords(type)`,
    which add to the most recently added overload;
  - `NativeFunction::with_keywords(definition, handler)`, whose handler takes `(&[Value], &Map)`,
    and `NativeFunction::call_with_keywords`. `NativeFunction::new` and `call` work as before for
    functions without keywords;
  - `KeywordArgument`, and `FunctionSignature`'s `keywords` and `rest_keywords` fields;
  - `FunctionType`'s `keywords` and `rest_keywords` fields.
- New error: `Function <name> got an unexpected keyword argument: <key>`.
- Breaking changes:
  - keyword arguments are no longer passed as a trailing map, and a function that doesn't declare
    a keyword raises an error when it's given one;
  - a duplicate keyword in a call is a syntax error;
  - `Expr::FunctionCall` holds a `FunctionCall`, with keyword arguments in `keyword_args` instead of
    a trailing `MapLiteral`;
  - `FunctionSignature` and `FunctionType` gain public fields, so hosts that build them with struct
    literals break.

# ADR 006: Bindings Can't Shadow Types, Functions or Keywords

- **Date:** 2026-10-24
- **Status:** Accepted.

## Context

Until now, a binding could shadow a built-in type name ([ADR 001](001-none-types.md), decision 4)
or any function in the global scope ([ADR 003](003-functions-as-values.md), decision 3), including
the built-in `typeof` ([ADR 005](005-typeof-native-function.md), decision 2). The parser also
accepted reserved keywords as binding names:

```flt
let Number = 5
typeof(Number)    # Number
let typeof = 1
typeof(2)         # typeof is not a function
let None = 1      # binds a variable that can't be read: `None` parses as the literal
let if = 1        # binds a variable that can't be read: `if` parses as the keyword
```

This caused problems:

- `None` and keywords could be bound, but those variables could never be read.
- Shadowing could break the REPL until the variable was removed. After `let Number = 5`,
  `typeof(1) == Number` is `false`. In a host such as datu, `let count = read("x")` silently hides
  the host's `count()` function.
- The lookup order (variables, then functions, then built-in type names) existed only to settle
  these clashes, and every new kind of global name would need a place in it.

## Decisions

### 1. A reserved keyword can't be a binding name

The parser doesn't accept a reserved keyword as the name in `let x = …` or `x = …`, so `let if = 1`,
`let let = 1` and `fn = 1` are parse errors. Keywords match whole words, so `iffy = 1` still binds.

The language already reserves these keywords. This decision only enforces that in bindings, and
any keyword added later is covered the same way. There's no runtime check for keywords: flt source
can no longer produce such a binding.

### 2. The reference runtime doesn't let a binding shadow a type or a function

In the reference runtime, `SimpleRuntime`:

- **A built-in type name can't be bound.** These are `None`, `Boolean`, `Number`, `String`,
  `Symbol`, `Array`, `Map` and `Type`, and also `Any`, which appears only in signatures.
  `let None = 1` and `Number = false` raise `Cannot bind Number: it's a built-in type`.
- **A function's name can't be bound.** This applies to every function in the global scope: the
  built-in `typeof`, and any native function the host registers. `let typeof = 1` and
  `let identity = 1` raise `Cannot bind identity: it's a function`.
- **A function value can still be bound to a new name.** `let f = identity` works, because `f`
  isn't a function's name.
- **A host can still replace a function** by registering another one under the same name, as
  [ADR 005](005-typeof-native-function.md) allows. The name stays protected.

The name is checked before the right side is evaluated, so `let Number = f()` raises the error
without calling `f`.

### 3. The global scope enforces the rule

The check is made by the global scope, not only by `let`:

- Binding a variable through the host API refuses the same names as `let` does.
- Registering a function under a built-in type name, or under a name that's already bound as a
  variable, is a host bug and panics. A host registers its functions before it evaluates flt source,
  so the second case only happens if it registers more functions later.

### 4. Every global name means one thing

After decisions 1 to 3, a name in the global scope is exactly one of: a variable, a function, or a
built-in type. Lookup keeps the order from [ADR 003](003-functions-as-values.md), but the order no
longer changes the result. Any new kind of global name can follow the same rule: it can't be
shadowed, and it can't reuse a name already in use.

### 5. This is a runtime decision, not a language rule

Decision 1 is part of the language, because the language reserves keywords. Decisions 2 to 4 are
design decisions for the reference runtime. Another runtime could let bindings shadow its
built-ins.

So the language spec gets no rule about shadowing. It already described shadowing in
runtime-level detail, in its "Types" section ("a `let` binding shadows it") and its "Bindings"
section ("A binding shadows a built-in type name"). Both sentences are deleted, and nothing replaces
them, so the spec says nothing either way. This follows the rule in `AGENTS.md` that the spec stays
broad and not runtime-specific.

## Alternatives Considered

- **Protect only built-ins, and let variables shadow host functions.** Rejected. It would leave one
  clash, and lookup order would still decide what a host function's name means. A host's functions
  are as much a part of the user's environment as the built-ins.
- **Reject keywords at run time instead of in the parser.** Rejected. The parser can reject them
  before anything is evaluated, and flt source then can't produce such a binding at all.
- **Allow shadowing, but warn.** Rejected. flt has no warnings, and a warned-about shadow still
  breaks the REPL.
- **State the rule in the spec.** Rejected. Shadowing is up to each runtime, as decision 5 explains.

## Consequences

- This supersedes:
  - ADR 001's decision that a binding shadows a built-in type name (decision 4, last bullet);
  - ADR 003's statements that a variable shadows a function and a function shadows a built-in type
    name (decision 3), though the lookup order itself stays;
  - ADR 005's statements that a variable shadows `typeof` and that removing it makes the built-in
    visible again (decision 2, first two bullets).
- **Breaking changes:**
  - `let` and assignment raise an error for a built-in type name or a function's name, and a reserved
    keyword as a binding name is a parse error.
  - In the Rust API, binding a variable returns a `Result`, and registering a function panics on a
    built-in type name or an existing variable's name. The global scope's maps of functions and
    variables are private, so a host can't get round the checks.
- In the REPL, `/unset` can only remove variables. `/unset Number` reports
  `Cannot unset Number: it's a built-in type`, and `/unset typeof` reports
  `Cannot unset typeof: it's a function`.
- Future kinds of global names, such as struct types, are protected the same way, and don't need a
  place in the lookup order.

# ADR 030: Pattern Matching in Function Parameters

- **Date:** 2026-11-05
- **Scope:** Language specification
- **Status:** Proposed

## Context

A function declared with `def` can have overloads ([ADR 013](013-type-checked-function-calls.md), [ADR 019](019-def-function-declarations.md)). A call runs an overload that accepts its arguments by number, type and keyword. A type is the narrowest way to tell overloads apart. A function that must treat one value in a special way, such as `0` in a recursive function, can only test the value in its body:

```flt
def fib(n: Integer) -> Integer
  if n == 0 { 0 } elsif n == 1 { 1 } else { fib(n - 1) + fib(n - 2) }
end
```

Some languages, such as Elixir, let a parameter be a value. The overload then accepts only that value. This ADR adds **pattern matching** to positional parameters, starting with **literal patterns**.

```flt
def fib(0) -> Integer
  0
end

def fib(1) -> Integer
  1
end

def fib(n: Integer) -> Integer
  fib(n - 1) + fib(n - 2)
end

fib(10)   # 55
```

## Decisions

### 1. Syntax

A positional parameter is a typed name or a literal:

```ebnf
param   = ident ":" type              (* positional, typed name *)
        | pattern                     (* positional, literal pattern *)
        | "*" ident ":" type          (* variadic *)
        | ident ":" type "=" expr     (* keyword, with default *)
        | "**" ident ":" type ;       (* rest keywords *)
pattern = [ "-" | "+" ] ( integer | number ) | string | boolean | "None" | symbol ;
```

- A number can have a `-` or `+` sign in front, with no space between: `-1`, `-2.5` and `+1`. The sign is part of the number. `+1` is the same pattern as `1`. The sign is allowed only before a number. `-x`, `--1`, `-(1)` and `- 1` are not patterns.
- A pattern is a literal and nothing else. A name, a call or any other expression is not a pattern. A name without a type is still a parse error ([ADR 019](019-def-function-declarations.md), decision 3).
- A string pattern is a plain string. It has no `{expr}` interpolation. An escape such as `\{` is allowed, because it gives a plain value.
- A pattern is allowed only in a positional parameter. A variadic parameter, a keyword parameter or a rest keywords parameter can't be a pattern.
- A pattern comes in the same place as a positional parameter. It follows the order rules of [ADR 019](019-def-function-declarations.md), decision 2.
- A pattern has no name, so the body can't read it. A function that needs the value declares a typed name.

### 2. A pattern accepts one value

An overload with a pattern accepts a call only if the argument for that parameter **matches** the pattern. An argument matches if it has the same type as the literal and is equal to it.

A literal has the type that it has everywhere else ([ADR 017](017-integers.md)): `0` is an `Integer`, `0.0` is a `Number`. So `def foo(0)` accepts the `Integer` `0` and does not accept the `Number` `0.0`, although `0 == 0.0` is `true`. A program that needs both writes two overloads, or declares a `Number`.

| Overload       | `foo(0)` | `foo(0.0)` | `foo(1)` |
| -------------- | -------- | ---------- | -------- |
| `def foo(0)`   | accepts  | no         | no       |
| `def foo(0.0)` | no       | accepts    | no       |

The check does not convert the argument. In particular, an `Integer` argument does not become a `Number` to match a `Number` pattern.

### 3. A pattern is part of the parameter signature

The literal belongs to the parameter signature of its overload, as a parameter type does ([ADR 018](018-distinct-overload-signatures.md), decision 1). So `foo(0)` and `foo(i: Integer)` are different signatures, and so are `foo(0)` and `foo(0.0)`. A second `foo(0)` has an equal signature, so a runtime that enforces distinct signatures treats it as a duplicate.

### 4. The pattern is narrower than its type

An overload with a pattern accepts a subset of the calls that an overload with the type of the literal accepts. With `f(n: Number)`, `f(i: Integer)` and `f(0)`, the call `f(0)` matches all three. The pattern is the narrowest of them. The language does not say which overload runs when several accept a call ([ADR 017](017-integers.md)). A runtime should run the narrowest, so that `f(0)` runs the pattern, and the order of the declarations does not matter. This differs from Elixir, where the first match runs.

### 5. A call that no overload accepts raises an error

A call with an argument that matches no pattern, and that no other overload accepts, raises an error as any other call does. The language does not require a runtime to give a message, or to say which overload the error is about.

### 6. Patterns and `const` functions

A `const def` can have patterns. All overloads of a function are `const`, or none is ([ADR 028](028-const-functions.md)).

### 7. Only literals

A pattern is a literal. A `const` name (`foo(MAX)`) and a call to a `const` function (`foo(twice(DEFAULT))`) are not patterns. A later ADR can allow them.

## Alternatives Considered

- **Matching by value with `==`.** Rejected: `def foo(2)` would accept `2.0`. Then `typeof` could not tell the two apart, and two overloads could be hard to tell apart in their type. A user who wants both values can declare two overloads. Guards, if flt gets them, are a better place for a comparison with `==`.
- **The first matching `def` runs, as in Elixir.** Rejected: the order of the declarations would decide the result. [ADR 013](013-type-checked-function-calls.md) rejects this for overloads, and [ADR 018](018-distinct-overload-signatures.md) does the same for duplicates.
- **A pattern with a type, such as `n: 0`.** Rejected: the literal already has a type, so the user would write it twice.
- **Names for patterns, such as `n @ 0`.** Deferred: it needs a rule for the type of `n` and a new syntax.
- **`const` names as patterns.** Deferred: a bare name in a parameter list needs a rule. A later ADR can choose one.

## Consequences

- A `def` can have a literal where it had a typed name. A program that used a typed name is unchanged.
- A new kind of parameter is needed in the AST.
- The type of an overload has a literal in place of a type. The language does not say how a runtime shows it. [`types.md`](../types.md) and [`functions-and-pipe.md`](../functions-and-pipe.md) describe the reference runtime.
- A later ADR can add guards, `const` patterns and patterns for arrays and maps.

# ADR 023: The `return` Statement

- **Date:** 2026-11-04
- **Scope:** Language specification
- **Status:** Accepted

## Context

A function declared with `def` gives the value of the last statement of its body ([ADR 019](019-def-function-declarations.md), decision 5). A function cannot stop earlier. To skip the rest of a body, the code must nest the rest inside an `else`.

The `return` keyword is reserved since [ADR 019](019-def-function-declarations.md), which left the `return` statement out of scope.

This ADR adds the `return` statement. It makes the `return` of the last statement optional, and allows an early return. Blocks that hold statements ([ADR 020](020-brace-blocks.md), [ADR 022](022-expanded-if.md)) give an early return a place to be written.

```flt
def check(x: Integer) -> String
  if x // 2 * 2 == x
    return "even"
  end

  "odd"
end
```

## Decisions

### 1. Syntax

`return` is a statement, with an optional expression:

```ebnf
statement = … | "return" [ expr ] ;
```

The expression is on the same line as `return`. A `return` that a line break, a `;` or `end` follows has no expression. The expression can be any expression, including a pipe: `return x |> f`.

### 2. Meaning

`return expr` stops the call of the nearest enclosing function. The value of the call is the value of `expr`. A `return` with no expression gives `None`.

The runtime checks this value against the return type, as it does for the value of the last statement. Thus `return "even"` in a function declared `-> Integer` raises a `ReturnTypeMismatch` error ([ADR 013](013-type-checked-function-calls.md), decision 6).

A `return` stops the call at once. The runtime does not evaluate the rest of the statement or expression that holds the `return`. The runtime closes every scope that the call opened ([ADR 021](021-block-scopes.md), decision 2).

### 3. The last `return` is optional

A function whose last statement is `e` and a function whose last statement is `return e` give the same value:

```flt
def foo() -> Boolean
  true
end

def foo() -> Boolean
  return true
end
```

A body that ends with `return` alone has the value `None`, as an empty body does.

### 4. Where `return` is allowed

A `return` can occur only in the body of a `def`, and in any block of statements inside it: a brace block or an arm of an `if` ([ADR 020](020-brace-blocks.md), [ADR 022](022-expanded-if.md)). A `return` at the top level, also inside a top-level block, is a syntax error. The same is true in a keyword default of a parameter: a default is an expression, and cannot hold a statement.

A runtime that is given a `return` statement by its host with no call active raises a `ReturnOutsideFunction` error. This protects a host that builds statements without the parser.

### 5. Calls and types do not change

A `return` does not change how the runtime picks an overload ([ADR 018](018-distinct-overload-signatures.md)), and does not change the type of the function, as `typeof` shows it ([ADR 019](019-def-function-declarations.md), decision 7).

## Alternatives Considered

- **`return` as an expression.** Rejected: an expression has a value, and `return` has none, because it never gives control back. Making it a statement keeps `1 + return 2` out of the grammar.
- **A `return` that is required as the last statement.** Rejected: it would break every function from [ADR 019](019-def-function-declarations.md), and `n * 2` is shorter than `return n * 2`.
- **A runtime error for a `return` at the top level.** Rejected for source code: the grammar can say it, and a syntax error shows it before anything runs. The runtime error in decision 4 is only for hosts.
- **Returning several values.** Rejected: the language has one value per expression. An `Array` or a `Map` returns several values.
- **Changing ADR 019 so that a missing `-> type` means `Any`.** Rejected here: it would change an accepted decision. See the open questions.

## Consequences

- The spec grammar gets `return` in `statement`, and the spec gets a section on returning early. It names the `ReturnTypeMismatch` and `ReturnOutsideFunction` errors and does not quote their messages.
- `Statement` gets a `Return` variant with an optional expression. Hosts that match on `Statement` exhaustively need a new arm.
- `ReturnOutsideFunction` is a new `RuntimeError` variant.
- A runtime needs a way to stop several nested evaluations at once. How it does this is left to the runtime.
- The `return` of [ADR 019](019-def-function-declarations.md)'s out-of-scope list is now in scope. Anonymous functions and closures are still out of scope. A later ADR that adds them has to say what `return` leaves.
- The `check` example above needs `-> String`. Without a return type, a function returns `None` ([ADR 019](019-def-function-declarations.md), decision 5), and `return "even"` raises `ReturnTypeMismatch`.
- A `return` in a block leaves the call, not only the block. A function body cannot hold a `def` ([ADR 021](021-block-scopes.md), decision 3), so no `return` can leave a nested function.
- Open questions for review:
  - Is it too strict that a function without `->` cannot return a value? The alternative is `Any`. It needs a change to ADR 019.

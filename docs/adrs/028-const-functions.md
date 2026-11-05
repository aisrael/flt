# ADR 028: `const` Functions

- **Date:** 2026-11-04
- **Scope:** Language specification
- **Status:** Proposed

## Context

A function declared with `def` ([ADR 019](019-def-function-declarations.md)) can give a different result for the same arguments. It can call a function that reads the clock, or one that reads a file. A program has no way to say that a function is meant to be **pure**: the same arguments always give the same result.

[ADR 025](025-const-declarations.md) added `const` for a binding that is meant to keep one value. This ADR uses the same keyword for a function that is meant to keep one result for one list of arguments.

```flt
const def area(w: Integer, h: Integer) -> Integer
  w * h
end

area(3, 4)      # 12, each time
```

## Decisions

### 1. Syntax

A `const` before `def` makes a function declaration a `const` declaration:

```ebnf
statement = … | [ "const" ] "def" ident "(" [ params ] ")" [ "->" type ] body ;
```

The rest of the declaration is the same as for `def`: the name, the parameters, the defaults, the return type, the body and `return` (ADR 019, [ADR 023](023-return-statement.md)).

`const` followed by `def` is a function declaration. `const` followed by a name is a constant declaration (ADR 025).

### 2. Meaning

A `const def` declares a function that is meant to be pure and deterministic. A call `f(a, b)` is meant to give the same value each time it has the same arguments `a` and `b`. The value must not depend on anything else, and the call must not change anything outside it.

The word "meant" is exact. The language states an intention. It is a contract that the author of the function and the runtime keep.

### 3. A runtime enforces the contract

The language does not say how a runtime keeps the contract, or whether it checks it. It does not name an error for a `const def` that is not pure. A runtime decides these:

- which functions a `const def` can call;
- what a default of a keyword parameter can call;
- how a `const def` and `def` overloads of one name work together;
- how the type of a function shows that it is `const`.

A runtime that does not check anything still follows the language. The `const` is then a statement of intent.

### 4. Reserved keyword

`const` is already a reserved keyword (ADR 025). This ADR does not add a keyword.

## Alternatives Considered

- **An annotation, such as `@pure def`.** Rejected: the language has no annotations, and `const` already means "meant to keep one value".
- **A new keyword, such as `pure`.** Rejected: it adds a keyword for an idea that `const` already covers.
- **The language checks the body.** Rejected: which functions are pure depends on the functions that a host provides. Only a runtime knows this.
- **`const def` only for native functions.** Rejected: a program should be able to build pure functions from pure functions.

## Consequences

- A `const def` is a new kind of statement. The AST node of a function declaration needs a flag for it.
- A `def` function is the same as before. Nothing changes for a program that does not use `const def`.
- A runtime can use the intent to check calls, or later to cache results. The language does not require this.

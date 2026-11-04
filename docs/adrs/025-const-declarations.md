# ADR 025: `const` Declarations

- **Date:** 2026-11-04
- **Scope:** Language specification
- **Status:** Accepted

## Context

`let` declares a variable ([ADR 016](016-typed-let-bindings.md)). A later `let` or assignment can replace its value. A program has no way to say that a name is meant to keep one value, such as a default or a limit.

This ADR adds the `const` keyword. It declares a **constant**: a binding that the program does not intend to change.

```flt
const DEFAULT = "default"
const LIMIT: Integer = 10
const RATE: Number = 2        # RATE is 2.0
```

## Decisions

### 1. Syntax

`const` is a statement. It has the same form as a declaration with `let`, and the type is optional:

```ebnf
statement = … | "const" ident [ ":" type ] "=" expr ;
```

### 2. Meaning

`const NAME = expr` binds `NAME` to the value of `expr`, as `let` does. A `const` with a type follows the rules of a `let` with a type ([ADR 016](016-typed-let-bindings.md)). The value must be of the type, and the runtime converts it. Thus `const RATE: Number = 2` binds `2.0`. A `const` without a type has no declared type. Its value keeps the type that it has.

A `const` gives the bound value, as `let` does.

The language does not say when the runtime evaluates the expression, or if the expression must be constant. The runtime decides.

### 3. Reserved keyword

`const` is a reserved keyword. It cannot be the name of a binding or of a function.

### 4. Rebinding is up to the runtime

The language does not say what happens when a program rebinds a constant. A runtime decides if it refuses, and how. A runtime can raise an error, or can accept the new value.

### 5. Naming convention

By convention, the name of a constant is in `ALL_CAPS`, such as `DEFAULT` and `MAX_RETRIES`. The language does not enforce this. A constant can have any name that a variable can have.

## Alternatives Considered

- **A required type.** Rejected: `const DEFAULT = "default"` is clear without one, and `let` does not need one.
- **A naming rule enforced by the parser.** Rejected: it adds a rule to the grammar for little value. A convention is enough.
- **The language forbids rebinding.** Rejected: the scope rules decide what counts as a rebind (for example, a name in a block that hides an outer name), and the language leaves scopes to the runtime ([ADR 021](021-block-scopes.md)).

## Consequences

- `const` is no longer a valid identifier. A program that used it as a name must rename it.
- A new AST node is needed for the declaration.
- A later ADR can let a constant stand in for a literal in other constructs.

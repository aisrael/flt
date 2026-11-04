# ADR 021: Block Scopes

- **Date:** 2026-11-03
- **Scope:** Runtime
- **Status:** Accepted

## Context

`SimpleRuntime` has two kinds of scope: the global scope, and the scope of a call to a function declared with `def` ([ADR 019](019-def-function-declarations.md)). A `let` binds a name in the innermost of these ([ADR 016](016-typed-let-bindings.md)).

A brace block ([ADR 020](020-brace-blocks.md)) holds statements, so it can hold a `let`. The language does not say what a block can see or declare. `SimpleRuntime` must decide.

## Decisions

### 1. A block is a scope

`SimpleRuntime` has three kinds of scope: global, function and **block**. Each brace block, including each brace arm of an `if`, has a block scope of its own.

### 2. Rules for names

A block scope follows the rules of a function scope:

- `let` declares a variable in the block only. It can shadow a name from an enclosing scope. The outer binding returns when the block ends.
- A lookup goes from the innermost block outward, then to the global scope.
- `x = expr` assigns to the nearest binding of `x`. If there is none, it declares `x` in the innermost block.
- A function call sees only its own scopes and the global scope. It does not see the blocks of its caller.
- The condition of an `if` is evaluated in the enclosing scope.
- `SimpleRuntime` closes every block it opened when an error stops the block.

### 3. No `def` in a block or a function body

For now, `SimpleRuntime` does not allow a `def` inside a block or inside the body of a function. Such a `def` raises a `DefNotAtTopLevel` error. A `def` is allowed only at the top level, where its name is bound in the global scope ([ADR 019](019-def-function-declarations.md)). A later ADR can change this, for example to give a `def` in a block a name in the scope of that block.

### 4. The host sees only the global scope

The `Runtime` trait ([ADR 008](008-the-runtime-trait.md)) does not change. A host sees only the global scope.

## Alternatives Considered

- **No scope for a block.** Rejected: a `let` in an arm would leak into the code after the `if`.
- **Binding the name of a `def` in a block globally.** Rejected for now: the function would stay after the block ends, which breaks the idea that a block keeps its names to itself.
- **Binding the name of a `def` in the scope of the block.** Deferred: it needs a rule for a function value that outlives its block.
- **Blocks that cannot shadow.** Rejected: a function scope can shadow, and one rule is simpler.

## Consequences

- A `let` in a block is not visible after the block.
- `SimpleRuntime` keeps a stack of scopes for each call, and one for the top level.
- Another runtime can choose differently, for example one scope for each function and none for a block.
- A function body cannot declare a function. This is a change from the behavior of ADR 019 in `SimpleRuntime`, where a `def` in a body bound its name globally.
- A new `RuntimeError` variant is needed: `DefNotAtTopLevel`.

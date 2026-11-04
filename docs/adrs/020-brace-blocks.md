# ADR 020: Brace Blocks

- **Date:** 2026-11-03
- **Scope:** Language specification
- **Status:** Accepted

## Context

An arm of an `if` is one expression, or one expression in braces: `if c 1 else 2` and `if c { 1 } else { 2 }`. The braces only group. An arm cannot hold a statement, so it cannot declare a name with `let`.

We want an expression that holds several statements.

## Decisions

### 1. Brace blocks

A **brace block** starts with `{` and ends with `}`. It holds statements, and it is an expression. It evaluates to the value of its last statement. A block with no statements evaluates to `None`.

Statements in a block are separated as at the top level: by a line break, or by `;`. Thus `{ let a = 1; a * 2 }` is a block with two statements, and it gives `2`.

`{ 1 }` is a block with one statement. It means: start the block, evaluate `1`, end the block. It gives `1`.

### 2. Block or map

A `{` also starts a map literal. A `{` starts a map if the next token is `}`, or a key followed by `:`. Otherwise it starts a block. Thus `{ a: 1 }` is a map, and `{ a }` is a block.

The context decides for `{}`. As an arm of an `if` (see decision 3), `{}` is an empty block, and it evaluates to `None`. In every other place, `{}` is an empty map. Parentheses make an arm a plain expression, so `if true ({})` evaluates to an empty map.

### 3. Every brace arm of an `if` is a block

In `if c { 1 } else { 2 }`, each brace arm is a brace block. The parser does not remove the braces. Thus `if true {}` has an empty block as its arm. An arm without braces (`if c 1 else 2`) is a plain expression.

### 4. The language does not restrict scope

The language does not say which names a block can see or declare. That is a decision of the runtime.

## Alternatives Considered

- **Collapsing `{ expr }` to `expr`.** Rejected: the shape of the AST would depend on the content of the block.
- **A different delimiter for blocks.** Rejected: `{` ... `}` is already the delimiter of the brace arm.

## Consequences

- This is a breaking change for `if c {}`: its arm was an empty map, and is now an empty block. To get an empty map as an arm, write `({})`.
- The AST of a brace arm changes. `{ 1 }` was `Literal(Integer(1))`. It is now a block that holds one statement.
- In a call without parentheses, a `{` starts an argument only if it starts a map. Thus in `if x { 1 }` the braces are the arm of the `if`, not an argument of `x`.
- The spec grammar gets a `block` primary, and its `if` grammar changes for brace arms.
- A later ADR can add a block form of `if` (`if` ... `else` ... `end`). `elsif` is also deferred.

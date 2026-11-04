# ADR 022: The Expanded Form of `if`, and `elsif`

- **Date:** 2026-11-03
- **Scope:** Language specification
- **Status:** Accepted

## Context

An `if` has a compact form: `if c a else b`, and `if c { a } else { b }`. A brace arm is a block that can hold statements ([ADR 020](020-brace-blocks.md)). A long arm is easier to read as lines closed by `end`, as the body of a `def` is ([ADR 019](019-def-function-declarations.md)).

We also want to test more than one condition without nesting `if` in `else`.

## Decisions

### 1. Two forms of `if`

The **compact form** does not change. The **expanded form** has statements in each arm, and `end` closes it:

```flt
if ready
  let n = 2
  n * 3
else
  0
end
```

It is an expression. It evaluates to the value of the last statement of the arm that ran. An `if` without `else` evaluates to `None` when no condition is true.

`if c { 1 } else { 2 }` and `if c`, `1`, `else`, `2`, `end` (on separate lines) are equivalent, and give the same AST: each arm is a block ([ADR 020](020-brace-blocks.md)).

### 2. Telling the forms apart

After the condition:

- A `{` on the same line means the compact form with a brace block.
- A line break, a comment, or a `;` before the first token of the arm means the expanded form. It must close with `end`.
- Anything else means the compact form with a one-expression arm.

A `;` is the same as a line break here. No expression has a `;` right after the condition of an `if`, so this is not ambiguous. Thus `if x; a; else b; end` is valid.

### 3. Statements in an arm

Statements in an arm are separated by a line break or a `;`, as in a `def` body. A list of statements ends at `else`, `elsif` or `end`. The first statement after `else` can follow it on the same line.

An arm with no statements is an empty block, and it evaluates to `None`.

The forms do not mix. An expanded `if` always ends with `end`, and a compact `if` never does.

### 4. `elsif`

`elsif` is a keyword. It is sugar for an `else` arm that holds one nested `if`:

```flt
if a { 1 } elsif b { 2 } else { 3 }
```

means

```flt
if a { 1 } else { if b { 2 } else { 3 } }
```

An `elsif` has a condition, and then an arm in the same form as the `if` it belongs to. A chain can have any number of `elsif` arms. `elsif` must come before `else`. The conditions are tested in order, and the first true one runs its arm. The later conditions are not evaluated.

In the expanded form, one `end` closes the whole chain:

```flt
if a
  "first"
elsif b
  "second"
else
  "third"
end
```

A written-out `else` followed by `if` needs its own `end` for the inner `if`. `elsif` does not.

There is no new AST node. `Display` prints the nested form.

## Alternatives Considered

- **Only the expanded form.** Rejected: the compact form is short for simple cases.
- **A `then` keyword.** Rejected: it adds a keyword, and a line break already shows the form.
- **Not allowing `;` after the condition.** Rejected: `;` separates statements everywhere else, so one-line forms such as `if x; a; else b; end` would be the only exception.
- **A new AST node for `elsif`.** Rejected: the nested `if` has the same meaning, so a runtime needs no change.

## Consequences

- This is a **breaking change**. `if c`, a line break, and an arm without `end` was valid, and is now an error because the expanded form needs `end`.
- `elsif` becomes a keyword, so it is no longer a valid name.
- The AST does not change. Both forms give an `IfExpr` with block arms, so `Display` can keep printing the compact form.
- The keywords `else`, `elsif` and `end` must never start an argument of a call without parentheses.
- A runtime needs no change. How an arm is scoped is a decision of the runtime ([ADR 021](021-block-scopes.md)).

# ADR 024: A Warning for Statements After a `return`

- **Date:** 2026-11-04
- **Scope:** Runtime
- **Status:** Proposed

## Context

A statement after a `return` in the same block is allowed, and it never runs ([ADR 023](023-return-statement.md)). The language does not call this an error. But such a statement is almost always a mistake: the author expects it to run.

flt has no warnings yet. This ADR adds the first one, in `SimpleRuntime`. It does not change [ADR 004](004-pipe-operator.md) or [ADR 006](006-no-shadowing.md). Each of them rejected a warning, and noted that flt had none, but each gave another reason as well.

The language specification does not change: a warning is a choice of the runtime, not a rule of the language.

## Decisions

### 1. What is unreachable

A statement is **unreachable** when an earlier statement in the same list of statements is a `return`. A list of statements is the body of a `def`, a brace block, or an arm of an `if` ([ADR 020](020-brace-blocks.md), [ADR 022](022-expanded-if.md)).

```flt
def f() -> Integer
  return 1
  2              # unreachable
end

def g(c: Boolean) -> Integer
  if c
    return 1
    2            # unreachable
  end
  3              # reachable: the `if` may not run its arm
end
```

This is a check of the syntax. It does not follow the flow of the program. It does not know that `if true { return 1 }` always returns, so the statements after that `if` are not reported.

### 2. When the runtime warns

`SimpleRuntime` checks the body of a function when it runs the `def` that declares the function. It does not check at each call. Thus a function gives its warnings once, and a function that is never called still gives them.

The check finds the first unreachable statement of each list of statements, and gives one warning for that list. It does not look inside the statements that it reports.

### 3. A warning is not an error

A warning does not stop the `def`, and does not change what the function does. The function is declared, and calls to it run as [ADR 023](023-return-statement.md) says.

### 4. The warning

The warning is `UnreachableCode`. It has the name of the function. Its message is `Unreachable code in <name>: statements after return never run`. [`docs/errors.md`](../errors.md#warnings) documents it.

### 5. How a host gets warnings

`SimpleRuntime` collects the warnings that `eval` and `eval_statement` give. A host reads and clears them with `SimpleRuntime::take_warnings`. A host that does not call it does not see them.

The `Runtime` trait does not change ([ADR 008](008-the-runtime-trait.md)): a warning is not part of the host API that every runtime must have. The `flt` command and its REPL print each warning to standard error, as `Warning: <message>`, after the statement that gave it.

## Alternatives Considered

- **A language rule that makes it an error.** Rejected: [ADR 023](023-return-statement.md) allows these statements, and code under development often has a `return` early on purpose.
- **Warning at each call.** Rejected: it repeats the same warning, and a function that is never called gives none.
- **Warning when the `return` runs.** Rejected: it needs a run that reaches the `return`, and it warns about a statement that the author may not know is dead.
- **Following the flow of the program** (for example, `if true { return 1 }`). Rejected for now: it needs rules for conditions and for the types of `if`. A later ADR can add them.
- **A method on the `Runtime` trait.** Deferred: the trait is the host API of every runtime, and one use does not justify a change to it.
- **Warnings as `RuntimeError` variants.** Rejected: an error stops a statement, and a warning does not.

## Consequences

- A new `Warning` type has one variant now, `UnreachableCode`. Other warnings can use it later.
- `SimpleRuntime` gets a list of warnings, and `take_warnings`.
- The `flt` command and the REPL print warnings to standard error. A script that reads standard output does not change.
- A runtime other than `SimpleRuntime` can choose not to warn.
- Open questions for review:
  - Should the REPL print the warning before or after the value of the statement?
  - Should a later ADR let a host choose to treat a warning as an error?

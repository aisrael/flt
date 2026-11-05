# ADR 027: Function Visibility

- **Date:** 2026-11-04
- **Scope:** Runtime
- **Status:** Accepted

## Context

A function declared with `def` ([ADR 019](019-def-function-declarations.md)) runs in a scope of its own. A name that this scope does not bind is looked up in the global scope ([ADR 021](021-block-scopes.md)). An assignment `x = expr` changes the nearest binding of `x`, and that can be a global variable. Thus a function can read and change global state:

```flt
let counter = 0
def bump()
  counter = counter + 1
end
bump()          # counter is now 1
```

The same call can give a different value each time, because it reads a global variable that another call changed. A default of a keyword parameter has the same problem.

This ADR limits what the body of a `def` function sees in `SimpleRuntime`. The result of a call then depends only on the arguments, on global constants ([ADR 025](025-const-declarations.md), [ADR 026](026-const-in-the-reference-runtime.md)) and on the functions that the body calls.

The language specification does not change its rules: a runtime decides what the body of a function sees.

## Decisions

### 1. What a body sees

The body of a `def` function, with the blocks in it, sees only these names:

- its parameters and its local bindings;
- global constants;
- global functions, the ones declared with `def` and the native ones;
- type names.

It does not see global variables. A global variable does not exist for the body.

### 2. Reading a global variable

A reference to a global variable in a body raises an `UnboundIdentifier` error, as for any name that has no binding.

### 3. Assigning to the name of a global variable

An assignment `x = expr` in a body assigns the nearest binding of `x` that the body sees. A global variable is not one of them. If there is none, the assignment declares `x` in the scope of the call ([ADR 021](021-block-scopes.md)). The global variable keeps its value. A `let x = expr` does the same.

```flt
let counter = 0
def write()
  counter = 1       # declares a local counter
  counter
end
write()             # 1
counter             # 0
```

### 4. Global constants

A body sees a global constant and can read it. A constant cannot change, so a read gives the same value in each call. The rules of ADR 026 do not change:

- An assignment to a global constant in a body raises `ConstantRebound`.
- A `let` or `const` in a body with the name of a global constant hides it, and gives a `ConstantHidden` warning. A parameter hides it with no warning.

```flt
const STEP = 2
def next(n: Integer) -> Integer
  n + STEP          # STEP is a constant
end
next(1)             # 3
```

### 5. Defaults of keyword parameters

A default is stricter than a body. A default sees global constants and type names. It does not see global variables, and it does not see global functions. A reference to a global variable raises `UnboundIdentifier`. A call to a global function raises `UndefinedFunction`, because the function does not exist for the default.

```flt
def foo(now = now())    # a call of foo without now raises UndefinedFunction
  now
end
```

A function in a default could have side effects, as `now()` has, and then the function that has the default is not pure. A default is evaluated only when the call leaves out the keyword, and it raises its error then. The declaration does not check the default.

A default still cannot see the other parameters or the variables of the caller.

### 6. The check comes at call time

The runtime applies these rules when a name is looked up and when an assignment runs. It does not analyse the body when the function is declared. A body can use a global constant or a function that is not declared yet, so an analysis at declaration time is not complete.

### 7. Native functions

The rules are for functions declared in flt. A native function runs in the host, and the runtime does not limit it. The runtime assumes that a native function can have side effects, such as `now()`. A body can call a native function, and so a function declared with `def` is pure only if it does not call a native function, directly or through other functions.

### 8. The result is relative to the constants

The result of a call depends on the arguments and on the global constants. A host can change the constants between two calls, with `remove_binding` or `declare_constant`, and a program can declare a constant after the function. A function is pure for a fixed set of global constants.

### 9. The host does not change

The `Runtime` trait ([ADR 008](008-the-runtime-trait.md)) does not change. `get_variable` and `set_variable` still see all of the global scope. A host that gives a value to the functions of a program declares it as a constant with `declare_constant`.

## Alternatives Considered

- **A new error for a global variable in a body.** Rejected: `UnboundIdentifier` already describes a name with no binding. A body cannot use the variable, so it does not exist for the body.
- **An error for an assignment to the name of a global variable.** Rejected: it needs a new error, and the name then means two things. `=` declares a local when no binding is found, and this rule keeps that.
- **Forbid only the assignment of a global variable.** Rejected: a read of a global variable also makes the result depend on state that the call does not control.
- **Allow global functions in a default.** Rejected: a default such as `now()` would make a function impure.
- **Analyse the body when the function is declared.** Rejected: see decision 6.
- **Let a host mark a variable as visible.** Rejected: a constant already does this, and a variable that a function sees is not pure.

## Consequences

- This is a breaking change. A program that reads or assigns a global variable in a `def` stops working. It must pass the value as an argument, return the new value, or use a `const`.
- A host that uses `set_variable` to give context to functions must use `declare_constant`.
- An assignment in a body to the name of a global variable declares a local, and gives no warning. A later ADR can add a warning.
- A later ADR can allow calls to pure functions in a default.

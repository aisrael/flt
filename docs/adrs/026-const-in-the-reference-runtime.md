# ADR 026: Constants in the Reference Runtime

- **Date:** 2026-11-04
- **Scope:** Runtime
- **Status:** Accepted

## Context

[ADR 025](025-const-declarations.md) adds `const` and leaves rebinding to the runtime. This ADR says what `SimpleRuntime` does. It also changes the `Runtime` trait ([ADR 008](008-the-runtime-trait.md)), so a host can declare and remove constants.

## Decisions

### 1. Rebinding raises `ConstantRebound`

`SimpleRuntime` marks a constant in the scope that declares it. It raises a `ConstantRebound` error when a program or a host binds a name that is a constant **in the same scope**, in any of these ways:

- `let X = …` or `let X: T = …`
- `X = …`, when `X` is the nearest binding of the name
- `const X = …`
- `Runtime::set_variable("X", …)` and `Runtime::declare_constant("X", …)`

The statement binds nothing, and the constant keeps its value.

### 2. The check comes first

As for `CannotBind` and `UnknownType` ([ADR 016](016-typed-let-bindings.md)), the runtime checks the name before it evaluates the expression. Thus `const X = f()` does not call `f` when `X` is already a constant.

### 3. The expression is evaluated once

The expression of a `const` does not need to be constant. `SimpleRuntime` evaluates it once, when the statement runs. Thus `const NOW = now()` calls `now()` once, and the value stays fixed.

### 4. Scopes, and hiding a constant

A constant belongs to the scope that declares it ([ADR 021](021-block-scopes.md)). It is gone when that scope ends.

A `let` or a `const` in an inner scope, a block or a function call, can use the name of a constant in an outer scope. This declares a new binding that **hides** the constant. It is not a rebind, so it is not an error. But it is probably a mistake, so the runtime gives a `ConstantHidden` warning ([ADR 024](024-unreachable-code-warning.md)). The runtime gives it after the new binding is made. If the statement fails, there is no warning.

- `X = 2` in the inner scope assigns the nearest binding of `X`. If this is the outer constant, it is a rebind: `ConstantRebound`.
- A parameter that has the name of a constant hides it with no warning. The name is part of the signature, and a warning at each call would not help.

### 5. Name rules

A `const` follows the name rules of `let`. A built-in type name, the name of a function and the name of a native type raise `CannotBind` ([ADR 006](006-no-shadowing.md)).

### 6. `remove_variable` becomes `remove_binding`

The `Runtime` method `remove_variable` is renamed `remove_binding`. It removes a variable or a constant from the global scope, with its declared type and its constant mark, and returns its value. A host acts outside the program, and the REPL command `/unset` needs this to start again. The old name is removed, with no alias.

### 7. Hosts declare constants

A new `Runtime` method, `declare_constant(name, value) -> Result<(), Error>`, binds `name` to `value` as a constant in the global scope. It raises `CannotBind` for a name that the runtime refuses, and `ConstantRebound` if `name` is already a constant. It replaces a variable of that name. The host converts the value itself if it needs a type.

## Alternatives Considered

- **A warning for a rebind.** Rejected: the program would run with a value that it meant to keep fixed.
- **An error for hiding.** Rejected: a block and a function call have scopes of their own so that they can use their own names ([ADR 021](021-block-scopes.md)). The outer constant is not changed.
- **No message for hiding.** Rejected: a constant that a later `let` hides is easy to miss.
- **A warning for a parameter that hides a constant.** Rejected: see decision 4.
- **Let a host's `set_variable` change a constant.** Rejected: the host would bypass the rule that the program must keep.
- **Let `remove_binding` fail on a constant.** Rejected: a host and the REPL would have no way to reset a name.
- **A separate `remove_constant` method.** Rejected: a host that removes a name should not need to know what kind of binding it is.

## Consequences

- A new `RuntimeError` variant is needed: `ConstantRebound`. A new `Warning` variant is needed: `ConstantHidden`.
- `Runtime::remove_variable` is a breaking change for hosts. They must call `remove_binding`.
- `Runtime` has a new method, `declare_constant`. A host that implements `Runtime` itself must add it.
- Another runtime can choose differently, for example to accept the new value.

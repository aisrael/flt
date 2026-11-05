# ADR 029: `const` Functions in the Reference Runtime

- **Date:** 2026-11-04
- **Scope:** Runtime
- **Status:** Accepted

## Context

[ADR 028](028-const-functions.md) adds `const def` for a function that is meant to be pure, and leaves the enforcement to the runtime. This ADR says what `SimpleRuntime` does.

[ADR 027](027-function-visibility.md) already limits what the body of a `def` sees. A body does not see global variables. One source of impurity is left: a call to a function that has side effects, such as the native function `now()`. A `const` function must not call such a function.

## Decisions

### 1. A `const` function can only call other `const` functions

While the body of a `const def` runs, a call to a function that is not `const` raises a `NonConstFunctionCall` error. This applies to the blocks in the body, and to the functions that the body calls. A call to a `const` function is always allowed.

```flt
const def area(w: Integer, h: Integer) -> Integer
  w * h
end

def stamp() -> Integer
  now()
end

const def bad() -> Integer
  stamp()          # NonConstFunctionCall
end
```

A `def` function can call a `const` function. Top-level code can call any function.

### 2. Defaults can call `const` functions

A default of a keyword parameter, of a `def` or of a `const def`, is evaluated with the same rule: it can only call `const` functions. A call to another function raises `NonConstFunctionCall`. This changes decision 5 of ADR 027, where a call to any global function raised `UndefinedFunction`.

```flt
const def seconds(m: Integer) -> Integer
  m * 60
end

def wait(timeout: Integer = seconds(2)) -> Integer      # allowed
  timeout
end

def stamp(at = now())                                    # stamp() raises NonConstFunctionCall
  at
end
```

The rest of decision 5 stays. A default sees global constants and type names, and not global variables or the other parameters. A default is evaluated only when the call leaves out the keyword, and it raises its error then. The declaration does not check it.

A default now sees the same global names as a body. So `SimpleRuntime` needs no separate visibility for a default.

### 3. The check comes at call time

The runtime checks a call when it resolves the name of the function. It does not analyse the body when the function is declared. A branch that does not run cannot break the contract, and a body can call a function that is declared later.

The runtime checks the function that the call names. It does not need the overload that the call picks, because all overloads have the same property (decision 4).

### 4. All overloads are `const`, or none is

`const` is a property of the function, and not of one overload. A `def` that adds an overload to a function declared with `const def` raises a `ConstOverloadMismatch` error. So does a `const def` that adds an overload to a function declared with `def`.

```flt
const def f(n: Integer) -> Integer
  n
end

def f(s: String) -> String          # ConstOverloadMismatch
  s
end
```

The runtime does this check before the check for a duplicate parameter signature ([ADR 018](018-distinct-overload-signatures.md)). Thus a `def f(Integer)` followed by a `const def f(Integer)` raises `ConstOverloadMismatch`. Two `const def` overloads with equal parameter signatures raise `DuplicateOverload`.

A `def` cannot add an overload to a function that a host registered ([ADR 019](019-def-function-declarations.md)), so a native function and a `def` never mix.

### 5. Native `const` functions

A host declares that a native function is `const` in the definition of the function, and registers it as before. The `Runtime` trait ([ADR 008](008-the-runtime-trait.md)) does not change. The mark applies to the whole function. A host cannot mark one overload of a native function and not another, so a marked function has only `const` overloads.

The runtime cannot check a native function. The host must keep the contract: a call with the same arguments always gives the same result. A native function that the host does not mark is assumed to be impure (ADR 027, decision 7).

The built-in `typeof` is `const`.

### 6. The type of a `const` function

The type of a function shows `const` on each of its overloads, for example `const fn(Integer) -> Integer`. The property is part of the type. Two types are equal only if both are `const` or both are not. All overloads of a function show it, so each overload still shows differently and maps back to its declaration (ADR 018).

### 7. The result is relative to the constants

As in ADR 027, decision 8, a `const` function is pure for a fixed set of global constants. A `const NOW = now()` is evaluated once, so a `const` function that reads it gives the same result for as long as the constant keeps its value.

## Alternatives Considered

- **A visibility that hides the functions that are not `const`.** Rejected: ADR 027 uses visibility for names that do not exist for a body. A function that exists but is not allowed is a different error, and a message that says "undefined" would mislead.
- **Check at declaration time.** Rejected: see decision 3.
- **A warning instead of an error.** Rejected: the function would run and break the contract.
- **`const` on each overload.** Rejected: a call would need the overload before the runtime can check it, and a function with some `const` overloads has no clear meaning.
- **A new `Runtime` method to register a `const` native function.** Rejected: the definition of the function already describes the function and its overloads.
- **Keep `UndefinedFunction` for a default.** Rejected: a `const` function in a default is allowed, so the function does exist.

## Consequences

- Two new `RuntimeError` variants are needed: `NonConstFunctionCall` and `ConstOverloadMismatch`.
- A default that calls a function that is not `const` raises `NonConstFunctionCall` and no longer `UndefinedFunction`. A default that calls a `const` function works.
- `FunctionDefinition` and `FunctionType` get a `const` property. Both have public fields, so a host that builds them with a struct literal must add it.
- A host that wants its native functions to be callable from `const` functions must mark them.
- Another runtime can choose differently, for example to check nothing.

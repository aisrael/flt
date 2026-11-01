# ADR 018: Overloads Have Distinct Parameter Signatures

- **Date:** 2026-10-31
- **Applies to:** Language specification
- **Status:** Proposed

## Context

A function can have several overloads ([ADR 003](003-functions-as-values.md), [ADR 013](013-type-checked-function-calls.md)). A call picks an overload from the types of its positional arguments and the names and types of its keywords (ADR 013, decisions 3 and 5).

Nothing yet forbids two overloads that a call cannot tell apart. These two take the same arguments:

```
foo(s: String) -> Number
foo(s: String) -> String
```

```flt
foo("a")      # which overload is called?
```

The return type cannot help. The runtime picks the overload before it calls the function, and it checks the result against the return type of the overload it picked (ADR 013, decision 6). A second case is two overloads that differ only in a default value or in the name of a positional parameter. A call cannot see these either.

The same problem affects the type of the function. `typeof` shows the type of each overload (ADR 013, decision 8). If two overloads show the same type, we cannot tell from the type which overload declaration it came from.

## Decisions

### 1. The overloads of a function have distinct parameter signatures

The **parameter signature** of an overload is:

- its positional parameter types, in order;
- its variadic parameter type, if it has one ([ADR 011](011-variadic-arguments.md));
- its keyword parameters, each by name and type, and its rest keyword type, if it has one ([ADR 010](010-keyword-arguments.md), [ADR 012](012-variadic-notation.md)).

Two overloads of one function must not have equal parameter signatures. The runtime compares types as types. The order in which keywords are declared does not matter, because a call passes them by name.

These parts of an overload are not in the parameter signature:

- the return type;
- the names of positional and variadic parameters;
- the default values of keyword parameters.

So these pairs are all duplicates:

```
foo(s: String) -> Number
foo(s: String) -> String                 # differs only in the return type

foo(s: String) -> Number
foo(text: String) -> Number              # differs only in a positional name

foo(s: String, n: Number = 0) -> Number
foo(s: String, n: Number = 1) -> Number  # differs only in a default
```

These pairs are not duplicates:

```
foo(s: String, i: Integer) -> Number         # 2 positional parameters
foo(s: String, i: Integer = 0) -> Number     # 1 positional parameter and the keyword i

foo(s: String) -> Number
foo(s: String, *rest: Any) -> Number         # a variadic parameter

foo(s: String, n: Number = 0) -> Number
foo(s: String, t: String = "") -> Number    # different keywords (ADR 013, decision 5)
```

### 2. A duplicate raises a `DuplicateOverload` error

When a function declares a second overload with an equal parameter signature, the declaration raises a `DuplicateOverload` error.

### 3. To vary the result, use a union in the return type

An overload cannot differ from another only by its return type. A function that can return several types declares a union ([ADR 002](002-union-types.md)):

```
foo(s: String) -> Number | String
```

The caller checks the type of the result, for example with `typeof`.

### 4. Overlapping signatures are still allowed

This rule applies to equal parameter signatures only. Two different signatures can still accept the same call:

```
show(x: Number) -> String
show(x: Any) -> String
```

`show(1)` is still a tie that the spec does not resolve (ADR 013, decision 4). This ADR does not change that.

### 5. Native functions follow the same rule

Every runtime must also enforce this rule for the native functions that a host registers. A host cannot register an overload with a parameter signature that is equal to the signature of another overload of the same function. A runtime rejects the registration. How a runtime reports the rejection is a matter for that runtime.

## Alternatives Considered

- **Allowing duplicates and picking the first declared.** Rejected: the overload that runs would depend on the order of declaration, and the later overload could never run. ADR 013 already rejects declaration order as a rule for picking overloads.
- **Picking an overload by return type.** Rejected: the runtime picks the overload before it calls the function. A return type would need type inference from the context of the call, which flt does not have.
- **Including default values in the parameter signature.** Rejected: a call that omits the keyword gets the default, so a call cannot tell two defaults apart. Two overloads that differ only in a default are still ambiguous.
- **Including the names of positional parameters in the parameter signature.** Rejected: a call passes positional arguments by position, so the names do not help it pick an overload.
- **Rejecting overlapping signatures, such as `Number` and `Any`.** Rejected for now: it needs rules for unions and `Any`. It would also remove useful pairs of a general overload and a specific one. ADR 013 leaves such ties unspecified, and a later ADR can add a rule.

## Consequences

- Each overload of a function has a different parameter signature. So the displayed type of each overload is different, and a type maps back to one overload declaration. The notation for function types ([ADR 003](003-functions-as-values.md), [ADR 012](012-variadic-notation.md)) does not change. Keyword names stay in the type, so `fn(String, Integer)` and `fn(String, i: Integer)` stay different.
- The spec names the `DuplicateOverload` error. A new `RuntimeError` variant is not needed until flt source can declare functions, because `SimpleRuntime` panics for now.
- Hosts that declare two overloads with equal parameter signatures must merge them. They can use a union in the return type, or a union in a parameter type.

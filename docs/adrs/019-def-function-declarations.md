# ADR 019: Declaring Functions in flt with `def`

- **Date:** 2026-10-31
- **Applies to:** Language specification
- **Status:** Accepted

## Context

Until now, only a host could define a function. A host registers native functions in the global scope ([ADR 003](003-functions-as-values.md), [ADR 014](014-native-function-arguments.md)). flt source can call them, but it cannot declare one.

`docs/spec.md` already reserves the `def` keyword and writes signatures such as `def mod(n: Integer, d: Integer) -> Integer`. It uses this notation to describe functions, but the grammar does not yet say what a `def` statement does.

The language already has all the parts of a signature:

- positional parameters ([ADR 003](003-functions-as-values.md));
- keyword parameters with defaults, and rest keywords ([ADR 010](010-keyword-arguments.md));
- a variadic parameter ([ADR 011](011-variadic-arguments.md), [ADR 012](012-variadic-notation.md));
- type-checked calls and overloads ([ADR 013](013-type-checked-function-calls.md), [ADR 018](018-distinct-overload-signatures.md));
- typed names ([ADR 016](016-typed-let-bindings.md)).

This ADR gives these parts a source syntax, so a user can declare a function in flt.

## Decisions

### 1. Syntax

A function declaration is a statement:

```ebnf
statement = "def" ident "(" [ params ] ")" [ "->" type ] body ;
params    = param { "," param } ;
param     = ident ":" type                 (* positional *)
          | "*" ident ":" type             (* variadic *)
          | ident ":" type "=" expr        (* keyword, with default *)
          | "**" ident ":" type ;          (* rest keywords *)
body      = { statement [ ";" ] } "end" ;
```

```flt
def mod(n: Integer, d: Integer) -> Integer
  n - (n // d) * d
end

def greet(name: String, greeting: String = "Hello") -> String
  greeting <> ", " <> name
end

def log(label: String, *values: Any, level: Symbol = :info)
  # ...
end
```

The `def` keyword is already reserved. A function name follows the rules for identifiers.

### 2. The order of parameters

Parameters come in this order:

1. positional parameters;
2. at most one variadic parameter (`*name: T`);
3. keyword parameters;
4. at most one rest-keywords parameter (`**name: T`).

This is the order in which a call passes its arguments ([ADR 010](010-keyword-arguments.md), [ADR 011](011-variadic-arguments.md)). A declaration in another order is a syntax error.

A parameter name cannot occur twice in one declaration. A repeated name is a syntax error.

### 3. Every parameter has a type

A parameter always declares its type. There is no untyped parameter. Use `Any` to accept every value. The runtime checks each argument against this type and converts an `Integer` for a `Number` parameter, as it does for a native function ([ADR 013](013-type-checked-function-calls.md), [ADR 017](017-integers.md)).

### 4. Defaults of keyword parameters

Every keyword parameter has a default expression. A call that omits the keyword evaluates the default expression at that call. A default can use the global bindings, but not other parameters. The runtime checks the value of the default against the type of the keyword, as it checks a value that the call passes.

### 5. The return type and the body

The `-> type` clause is optional. A declaration without it returns `None`, as if it had `-> None`.

The body is a sequence of statements that ends with `end`. The value of the function is the value of the last statement of the body. An empty body gives `None`. The runtime checks this value against the return type, and raises a `ReturnTypeMismatch` error if it does not match ([ADR 013](013-type-checked-function-calls.md), decision 6).

A `def` always has a body. The signature-only forms in §6 of the spec are notation, not statements. They do not declare a function.

### 6. A call has its own scope

A call to a flt function creates a scope for its body:

- Each parameter is bound in this scope, with its declared type ([ADR 016](016-typed-let-bindings.md)). A variadic parameter is bound to an `Array`. A rest-keywords parameter is bound to a `Map`.
- A `let` in the body declares a variable in this scope only. It is gone when the call ends.
- A reference to a name that the scope does not bind looks in the global scope.
- An assignment `x = e` assigns to the nearest binding of `x`. If there is none, it declares `x` in the scope of the call.

This is the first scope in the language. The spec says that "Bindings are global" until scopes exist. This ADR adds the scope of a call. It does not add any other scope.

### 7. What a declaration does

A `def` statement binds the name in the global scope to a function value, and gives `None`. The name is bound before the body runs, so a function can call itself.

Following [ADR 018](018-distinct-overload-signatures.md), a second `def` with the same name adds an overload. It raises a `DuplicateOverload` error if its parameter signature equals the signature of an existing overload. A second `def` never replaces an overload.

A `def` cannot use a name that is a reserved keyword, a built-in type, or a variable. It cannot add an overload to a function that a host registered. It raises a `NameAlreadyBound` error. This follows the intent of [ADR 006](006-no-shadowing.md).

The type of the function, as `typeof` shows it, is the same as for a native function: its signature without the name and the parameter names ([ADR 013](013-type-checked-function-calls.md), decision 8).

```flt
def double(n: Number) -> Number
  n * 2
end

typeof(double)    # fn(Number) -> Number
double(21)        # 42.0
```

### 8. Out of scope

This ADR does not add these:

- the `return` statement (the `return` keyword stays reserved);
- anonymous functions (`fn`) and closures;
- type parameters.

A later ADR can add them.

## Alternatives Considered

- **Untyped parameters.** Rejected: calls are checked by type, and overloads are chosen by type ([ADR 013](013-type-checked-function-calls.md)). A parameter without a type would need a rule for its type. `Any` says it clearly.
- **Required keyword parameters (no default).** Rejected: this would change the rule from [ADR 010](010-keyword-arguments.md) that a call can omit every keyword.
- **A signature-only `def` (no body) as a declaration.** Rejected: without `end`, the parser cannot tell if the next statement is a body or a new statement.
- **Keeping all bindings global.** Rejected: a parameter such as `n` would replace a global `n`, and a recursive call would overwrite its own arguments.
- **Replacing an overload on a second `def` with the same signature.** Rejected: it hides mistakes, and [ADR 018](018-distinct-overload-signatures.md) already chooses an error.
- **Letting a default use earlier parameters.** Rejected for now: it needs an order of evaluation between the defaults and the keyword arguments of the call. A later ADR can add it.

## Consequences

- The spec grammar changes from `[ body ]` to `body` for `def`, and §7 states that a call has a scope.
- Two new `RuntimeError` variants are needed: `NameAlreadyBound` and (with ADR 018) `DuplicateOverload`. `SimpleRuntime` needs a way to store function values with a flt body next to native ones.
- The `Runtime` trait ([ADR 008](008-the-runtime-trait.md)) does not change. A host can call a flt function like any other function.
- Because the body of a function can read global variables, a function can give different results for the same arguments. This is intended.
- Two parser rules came with the implementation. The arguments of a call in the whitespace form must be on the same line as its name, so `n` followed by a statement on the next line is two statements. A `-` followed by a space is a subtraction in `n - (n // d)`, not the start of an argument. The keyword `end` is never an argument.
- A function value should be a reference to the declared function, so `let f = show` and `show` share one list of overloads. `SimpleRuntime` does not do this yet: an overload added after the binding is seen through `show` only. A later change can fix it.
- Open questions for review:
  - A body value other than `None` in a function without `->` raises `ReturnTypeMismatch`. Is this too strict? The alternative is to discard the value.
  - Can `def` add an overload to a native function? Decision 7 says no.
  - Should `x = e` in a body assign to a global variable, or always declare a local one?

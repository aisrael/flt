# Runtime Behavior of the Reference Runtime

The language specification ([`spec.md`](spec.md)) describes the syntax of flt and the core semantics that every runtime must give. This page describes what the reference runtime (`SimpleRuntime`) does where the spec leaves a choice to the runtime. Another runtime can make other choices.

Related pages:

- [`errors.md`](errors.md) - the message text and the cause of each runtime error;
- [`native-functions-and-values.md`](native-functions-and-values.md) - native functions, types and values;
- [`runtime-and-limitations.md`](runtime-and-limitations.md) - the `Runtime` trait, evaluation and current limitations.

## Numbers

`Integer` and `Number` have arbitrary precision (see the spec). A `Number` with no digits after its decimal point prints with a trailing `.0`, for example `2.0`. Thus a printed `Number` never looks like an `Integer`.

## Native types

A runtime can let its host define **native types** (see [native types](native-functions-and-values.md#native-types-and-values)). The name of a native type gives that type. Its values come only from the functions of the host. flt source can bind, pass, compare and print native values. flt source cannot create them or look inside them. The host decides how they print. The host also decides how `==` and `!=` compare them. Values of different native types are never equal. Native values cannot be ordered.

A type in a `let` or a `def` declaration can be the name of a native type.

## Built-ins

A runtime can register any number of built-in, native functions. The reference runtime registers only `typeof(Any) -> Type`. It returns the type of its argument. Thus `typeof(1) == Integer` is `true`, and `typeof(x)` reports the type of the value of `x`, not the declared type of `x`.

`Any` is the type of every value. It occurs only in signatures. For example, `typeof` is `fn(Any) -> Type`.

## Function types

The type of a function is its signature without its name or parameter names. It displays like this:

```flt
typeof(flip_a_coin) # => fn() -> Boolean
typeof(say_hi)      # => fn(String) -> None
```

A function type lists its keywords after its positional parameters, for example `fn(String, greeting: String) -> String`.

A function type shows the variadic parameter as `*T`. It comes after the other positional parameters and before any keywords, for example `fn(String, *Any) -> String` or `fn(Any, *Any, k: Number) -> Any`. A rest keywords parameter shows as `**T`, for example `fn(**Any) -> Map`. A union after `*` or `**` is in parentheses, for example `fn(*(String | Symbol)) -> String`. A union return type is also in parentheses, for example `fn(T) -> (T | None)`.

The signature of a function that has a default for a keyword shows only the type of the keyword:

```text
greet: fn(String, greeting: String) -> String    # greeting defaults to "Hello"
```

## Scopes

The reference runtime has three kinds of scope: global, function and block ([ADR 021](adrs/021-block-scopes.md)). Outside a function body, a binding is global. It is visible to every statement after it.

### Function scope

A call to a function declared with `def` creates a scope for its body:

- Each parameter is bound in this scope, with its declared type. A variadic parameter is bound to an `Array`. A rest keywords parameter is bound to a `Map`.
- A `let` in the body declares a variable in this scope only. The variable is gone when the call ends.
- A reference to a name that this scope does not bind looks in the global scope.
- An assignment `x = expr` assigns to the nearest binding of `x`. If there is none, it declares `x` in the scope of the call.

A default of a keyword parameter can use global bindings, but not other parameters.

A called function does not see the blocks of its caller. It sees only its own scopes and the global scope.

### Block scope

Each [brace block](spec.md#blocks) has a scope of its own. This includes each brace arm of an `if` ([ADR 020](adrs/020-brace-blocks.md)). A block scope follows the rules of a function scope:

- A `let` declares a variable in the block only. It can hide a name from an outer scope. The outer binding returns when the block ends.
- A lookup goes from the innermost block outward through the enclosing blocks. Then it goes to the scope of the call, and last to the global scope.
- An assignment `x = expr` assigns to the nearest binding of `x`. If there is none, it declares `x` in the innermost block.
- The condition of an `if` is evaluated in the enclosing scope, not in a block of the `if`.
- The scope of a block is gone when the block ends. It is also gone when an error stops the block.

```flt
let x = 1
let r = {
  let x = 10         # hides the global x in this block
  x + 1
}
r                    # 11
x                    # 1
{ let y = 2 }
y                    # error: y is not bound
```

An arm without braces (`if c 1 else 2`) is a plain expression. It has no scope of its own.

A host sees only the global scope. The `Runtime` trait does not change.

## Declaring functions

A `def` statement binds its name in the global scope to a function value, and gives `None`. The name is bound before the body runs, so a function can call itself.

A `def` is allowed only at the top level. A `def` inside a block or inside the body of a function raises a `DefNotAtTopLevel` error ([ADR 021](adrs/021-block-scopes.md)). A later change can allow it.

A second `def` with the same name adds an overload. It raises a `DuplicateOverload` error if its parameters have the same types as an existing overload (see [ADR 018](adrs/018-distinct-overload-signatures.md)). A `def` raises a `NameAlreadyBound` error if its name is a built-in type or a variable. It also raises this error if the name is a function that a host registered.

## Type-checked calls

A function can have several signatures, or **overloads**. The runtime checks each call against the signatures by type. A signature **matches** a call if all of these are true:

- It takes that number of positional arguments.
- Each positional argument, and each value of a variadic parameter, is of the type of its parameter.
- Each keyword in the call is declared by the signature, with a value of the type of that keyword. Or the rest keywords of the signature accept it, with a value of their type. A keyword that the call omits takes its default.

A value is of a type in these cases:

- Every value is of type `Any`.
- A value is of a built-in type if it has that type.
- A value is of type `Number` if it is a `Number` or an `Integer`.
- A value is of type `A | B` if it is of either type.
- A value is of type `Option<T>` if it is `None` or of type `T`.
- A value is of a native type only if it is a value of that same native type, whatever its name.
- A value is of a function type if the type of the function, or the type of one of its signatures, is exactly that type.

Sometimes an `Integer` is of a type only because `Number` accepts it. Then the runtime converts the `Integer` to a `Number`. It converts an argument or a keyword value before it calls the function. It converts a result before it returns the result.

A union or `Option` converts the `Integer` only if none of its members accepts the `Integer` as it is. Thus `Integer | Number` and `Any` leave it an `Integer`. `Number | String` converts it. The runtime first chooses the signature to call. The types of that signature decide the conversion.

The **arity** of a signature is the number of its positional parameters before any variadic one. If several signatures match a call, which one the runtime calls is unspecified. The result of the function must be of the return type of the called signature.

```
foo(*args: Any) -> String
foo(a: Number, *args: Any) -> String
```

```flt
foo(1, :b)       # the second: arity 1 is tried before arity 0
foo(:b)          # the first
```

The keywords of a call help to select the signature to call. If no signature declares a keyword, the call raises an `UnexpectedKeywordArgument` error.

## Native functions and function values

A host can register native functions in the global scope (see [native functions](native-functions-and-values.md#native-functions)). The name of a function gives a function value of type `fn(P1, P2) -> R`. It displays as `<fn name>`. You can bind it with `let`. You can call it through any name that is bound to it.

Two function values are equal only if they are the same function. A function type can name native types, for example `fn(Date, Date) -> Number`. A native function can fail with the host's own error. The runtime reports this error without change.

## Name resolution

The runtime resolves identifiers and call names in this order:

1. variables
2. global functions
3. built-in type names
4. native type names

## Errors for calls

A call raises one of these errors. See [`errors.md`](errors.md) for the cause and the message of each.

- an `UndefinedFunction` error when the name is not bound;
- a `NotCallable` error when the name is bound to a value that is not a function;
- an `ArgumentCountMismatch` error when no signature of the function takes that number of positional arguments;
- when one signature takes that number of positional arguments but does not match, an error for the first check that fails. The runtime checks positional arguments first. Then it checks keywords in the order of the call:
  - an `ArgumentTypeMismatch` error for a positional argument of the wrong type;
  - an `UnexpectedKeywordArgument` error for a keyword that the signature does not accept;
  - a `KeywordTypeMismatch` error for a keyword value of the wrong type;
- when several signatures take that number of positional arguments and none matches, the error for the closest signature, as if it were the only one;
  - The closest signature is the one that passes the most checks, in the order above, before one fails.
  - If several signatures first fail at the same keyword, a signature that accepts the keyword but not its value is closer than one that does not accept it;
- a `NoMatchingOverload` error when several signatures are equally close;
- a `ReturnTypeMismatch` error when the result is not of the return type of the called signature.

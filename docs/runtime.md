# Runtime Behavior of the Reference Runtime

The language specification ([`spec.md`](spec.md)) describes the syntax of flt and the core semantics that every runtime must give. This page describes what the reference runtime (`SimpleRuntime`) does where the spec leaves a choice to the runtime. Another runtime can make other choices.

Related pages:

- [`errors.md`](errors.md) - the message text and the cause of each runtime error;
- [`native-functions-and-values.md`](native-functions-and-values.md) - native functions, types and values;
- [`host-api.md`](host-api.md) - the `Runtime` trait: how a host works with a runtime.

## Numbers

`Integer` and `Number` have arbitrary precision (see the spec). A `Number` with no digits after its decimal point prints with a trailing `.0`, for example `2.0`. Thus a printed `Number` never looks like an `Integer`.

## String concatenation

`<>` coerces its operands to strings. Numbers, booleans and symbols can be concatenated. A number becomes the text it prints as, so `"x" <> 2.0` is `"x2.0"`. Other values, including functions and native values, become the empty string.

## Native types

A runtime can let its host define **native types** (see [native types](native-functions-and-values.md#native-types-and-values)). The name of a native type gives that type. Its values come only from the functions of the host. flt source can bind, pass, compare and print native values. flt source cannot create them or look inside them. The host decides how they print. The host also decides how `==` and `!=` compare them. Values of different native types are never equal. Native values cannot be ordered.

A type in a `let` or a `def` declaration can be the name of a native type.

## Built-ins

A runtime can register any number of built-in, native functions. The reference runtime registers `typeof(Any) -> Type` and `println(String) -> None`. `typeof` is a [`const` function](#const-functions). It returns the type of its argument. `println` is not `const`: it prints its argument and a new line to standard output, and returns `None`. Thus `typeof(1) == Integer` is `true`, and `typeof(x)` reports the type of the value of `x`, not the declared type of `x`.

`Any` is the type of every value. It occurs only in signatures. For example, `typeof` is `const fn(Any) -> Type`.

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

The reference runtime has three kinds of scope: global, function and block ([ADR 021](adrs/021-block-scopes.md)). Outside a function body, a binding is global. It is visible to every statement after it, but a function body sees only some global bindings (see [Function scope](#function-scope)).

### Function scope

A call to a function declared with `def` creates a scope for its body:

- Each parameter is bound in this scope, with its declared type. A variadic parameter is bound to an `Array`. A rest keywords parameter is bound to a `Map`.
- A `let` in the body declares a variable in this scope only. The variable is gone when the call ends.
- A reference to a name that this scope does not bind looks in the global scope, but it finds only constants, functions and type names ([ADR 027](adrs/027-function-visibility.md)). A global variable does not exist for the body. A reference to it raises an [`UnboundIdentifier`](errors.md#unboundidentifier) error.
- An assignment `x = expr` assigns to the nearest binding of `x` that the body sees. A global variable is not one of them. If there is none, it declares `x` in the scope of the call, and the global variable keeps its value.

```flt
let counter = 0
const STEP = 2
def next(n: Integer) -> Integer
  n + STEP          # STEP is a constant: visible
end
def read()
  counter           # UnboundIdentifier: a global variable is not visible
end
def write()
  counter = 1       # declares a local counter
  counter
end
next(1)             # 3
write()             # 1
counter             # 0
```

Thus the result of a call depends only on the arguments, the global constants and the functions that the body calls. A host can change the constants between two calls, so a function is pure for a fixed set of global constants. The runtime assumes that a native function can have side effects, so a function that calls one is not pure.

A default of a keyword parameter sees global constants, type names and `const` functions ([`const` functions](#const-functions)). It does not see other parameters or global variables. A reference to a global variable raises `UnboundIdentifier`. A call to a function that is not `const` raises [`NonConstFunctionCall`](errors.md#nonconstfunctioncall). The runtime evaluates a default only when the call leaves out the keyword, so it raises the error then.

```flt
const WAIT = 5
def fetch(timeout: Integer = WAIT)       # a constant: allowed
  timeout
end
def stamp(at = now())                    # stamp() raises NonConstFunctionCall
  at
end
```

A called function does not see the blocks of its caller. It sees only its own scopes and the global constants, functions and type names.

### `const` functions

A function declared with `const def` is meant to be pure ([ADR 028](adrs/028-const-functions.md)). `SimpleRuntime` enforces this with a flag that says the running code must be `const` ([ADR 029](adrs/029-const-functions-in-the-reference-runtime.md)):

- The runtime sets the flag while the body of a `const def` runs, with its blocks and the functions that it calls. It sets the flag too while it evaluates a default of a keyword parameter, for a `def` and for a `const def`. It restores the flag when the call ends, also after an error.
- When the flag is set, a call to a function that is not `const` raises a [`NonConstFunctionCall`](errors.md#nonconstfunctioncall) error. A call to a `const` function is allowed.
- Top-level code and the body of a `def` do not set the flag. They can call any function, and a call to a `const` function from them is a normal call.
- The runtime checks the function that the call names, when the call runs. It does not analyse the body when the function is declared.
- A native function is `const` only if the host marks it so. The runtime cannot check it. The built-in `typeof` is `const`.

```flt
const def area(w: Integer, h: Integer) -> Integer
  w * h
end
def stamp() -> Integer
  now()
end
const def ok() -> Integer
  area(2, 3)         # 6
end
const def bad() -> Integer
  stamp()            # NonConstFunctionCall
end
def wait(t: Integer = area(1, 2))      # allowed: area is const
  t
end
```

The body of a `const def` sees the same names as the body of a `def`: its parameters, its locals, global constants, global functions and type names. A function that is not `const` is visible, but a call to it raises the error. So the result of a `const` call depends only on the arguments, the global constants and the `const` natives. As for any function, this holds for a fixed set of global constants.

Functions are called by name, and a function that a variable holds can not be called. A `const` body therefore cannot reach a function that is not `const` through an argument.

### Block scope

Each [brace block](spec.md#blocks) has a scope of its own. This includes each brace arm of an `if` ([ADR 020](adrs/020-brace-blocks.md)). A block scope follows the rules of a function scope:

- A `let` declares a variable in the block only. It can hide a name from an outer scope. The outer binding returns when the block ends.
- A lookup goes from the innermost block outward through the enclosing blocks. Then it goes to the scope of the call, and last to the global scope. In the body of a function, the lookup finds only the global names that [Function scope](#function-scope) lists.
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

### Constants

A binding that `const` declares is a constant ([ADR 025](adrs/025-const-declarations.md)). The runtime marks it in the scope that declares it ([ADR 026](adrs/026-const-in-the-reference-runtime.md)). In that same scope, a `let`, an assignment, a `const` or a host's `set_variable` with the name raises a [`ConstantRebound`](errors.md#constantrebound) error. The runtime checks the name before it evaluates the expression. It evaluates the expression once, when the statement runs, and it does not need to be constant: `const NOW = now()` calls `now()` once.

A `let` or a `const` in an inner scope can hide a constant of an outer scope. This is not a rebind, but it gives a [`ConstantHidden`](errors.md#constanthidden) warning. A parameter hides a constant with no warning. An assignment `X = expr` assigns the nearest binding of `X`. If that is a constant, it raises `ConstantRebound`.

```flt
const LIMIT = 10
LIMIT = 11            # ConstantRebound
let LIMIT = 11        # ConstantRebound
{
  let LIMIT = 1       # ConstantHidden: hides the constant in this block
  LIMIT + 1           # 2
}
{ LIMIT = 2 }         # ConstantRebound: the nearest LIMIT is the constant
LIMIT                 # 10
```

A host sees only the global scope. The `Runtime` trait does not change.

## Declaring functions

A `def` statement binds its name in the global scope to a function value, and gives `None`. The name is bound before the body runs, so a function can call itself.

A `def` is allowed only at the top level. A `def` inside a block or inside the body of a function raises a `DefNotAtTopLevel` error ([ADR 021](adrs/021-block-scopes.md)). A later change can allow it.

A second `def` with the same name adds an overload. All overloads of a function are `const`, or none is: a `def` or `const def` that differs from the function in this raises a `ConstOverloadMismatch` error. The runtime checks this before it checks for a duplicate. A `DuplicateOverload` error follows if its parameters have the same types as an existing overload (see [ADR 018](adrs/018-distinct-overload-signatures.md)). A `def` raises a `NameAlreadyBound` error if its name is a built-in type or a variable. It also raises this error if the name is a function that a host registered.

## Returning from a function

A `return` leaves the call of the nearest enclosing function ([ADR 023](adrs/023-return-statement.md)). The reference runtime does this with an internal signal that passes through the statements and expressions around the `return`. Each block closes its scope on the way out, as it does for an error. The call turns the signal into its value, and then checks the value against the return type, so a `return` and a last statement are checked the same way.

A `return` outside a function call raises a `ReturnOutsideFunction` error. The parser rejects a `return` outside a `def` in source code, so only a host that passes a `return` statement to `eval_statement` can cause it.

## Warnings

The reference runtime warns about code that is legal but probably a mistake ([ADR 024](adrs/024-unreachable-code-warning.md)). A warning does not stop a statement, and does not change its value. See [`errors.md`](errors.md#warnings) for the message of each warning.

- The runtime collects the warnings that `eval` and `eval_statement` give. A host reads and clears them with `SimpleRuntime::take_warnings`. The `Runtime` trait has no method for warnings (see [`host-api.md`](host-api.md#warnings)).
- The `flt` command and the REPL print each warning to standard error, as `Warning: <message>`, after the statement that gave it.
- The only warning now is `UnreachableCode`. The runtime gives it when it runs a `def` whose body has a statement after a `return` in the same list of statements. It checks the syntax and does not follow the flow of the program.

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

## Pattern matching

A positional parameter can be a literal pattern ([ADR 030](adrs/030-pattern-matching.md)). `SimpleRuntime` treats it in these ways:

- **A pattern matches by type and by value.** An argument matches if its type is the type of the literal and it is equal to it. `def foo(2)` does not accept `2.0`. The runtime does not convert an `Integer` to a `Number` to match a `Number` pattern.
- **A pattern is the narrowest type.** A literal type is at least as narrow as itself, as its base type, as `Number` if it is an integer, as `Any` and as a union that has one of these (see [The Narrowest Overload](functions-and-pipe.md#the-narrowest-overload)). So `SimpleRuntime` runs the pattern when it matches, whatever the order of the `def`s. This needs no new rule for selecting an overload.
- **A pattern is part of the signature.** The runtime compares a literal type as a type, so `foo(0)` and `foo(0.0)` differ, and a second `foo(0)` raises a `DuplicateOverload` error. A `const def` can have patterns, and the `ConstOverloadMismatch` check is unchanged.
- **Closeness.** A pattern counts as one more check after the type check of its argument, when the runtime looks for the closest overload.
- **Errors.** A failed pattern raises an `ArgumentTypeMismatch` error. `<argument>` is the position of the argument, `<type>` after `expects` is the literal, and the value after `found` is the value that was passed. Several overloads that are equally close raise `NoMatchingOverload`.
- **The type shows the literal.** `typeof` shows a pattern as its literal: `fn(0) -> Integer`.
- A host can't register a function with a pattern yet. The signature type can hold one.

## Native functions and function values

A host can register native functions in the global scope (see [native functions](native-functions-and-values.md#native-functions)). The name of a function gives a function value of type `fn(P1, P2) -> R`. It displays as `<fn name>`. You can bind it with `let`. You can call it through any name that is bound to it.

Two function values are equal only if they are the same function. A function type can name native types, for example `fn(Date, Date) -> Number`. A native function can fail with the host's own error. The runtime reports this error without change.

## Name resolution

The runtime resolves identifiers and call names in this order:

1. variables
2. global functions
3. built-in type names
4. native type names

## Errors

Every error that the runtime raises, with its message and its cause, is in [`errors.md`](errors.md). That page also lists the other kinds of error that `eval` returns: parse errors, host errors and handler errors.

### Errors for calls

A call raises one of these errors. See [`errors.md`](errors.md) for the cause and the message of each.

- an `UndefinedFunction` error when the name is not bound;
- a `NotCallable` error when the name is bound to a value that is not a function;
- an `ArgumentCountMismatch` error when no signature of the function takes that number of positional arguments;
- when one signature takes that number of positional arguments but does not match, an error for the first check that fails. The runtime checks positional arguments first. Then it checks keywords in the order of the call:
  - an `ArgumentTypeMismatch` error for a positional argument of the wrong type, or one that does not match a [literal pattern](#pattern-matching);
  - an `UnexpectedKeywordArgument` error for a keyword that the signature does not accept;
  - a `KeywordTypeMismatch` error for a keyword value of the wrong type;
- when several signatures take that number of positional arguments and none matches, the error for the closest signature, as if it were the only one;
  - The closest signature is the one that passes the most checks, in the order above, before one fails.
  - If several signatures first fail at the same keyword, a signature that accepts the keyword but not its value is closer than one that does not accept it;
- a `NoMatchingOverload` error when several signatures are equally close;
- a `ReturnTypeMismatch` error when the result is not of the return type of the called signature.

## Limitations

These constructs parse, but the reference runtime does not fully support them:

- Bitwise operators (`&`, `|`, `^`)
- Standalone keyword expressions (e.g. `fn`) - they parse as expressions but have no evaluation behavior yet

When you document the language:

- Treat parser support and evaluator support as separate capabilities.
- Use parser-focused tests when documenting syntax.
- Use REPL behavior when documenting what executes today.

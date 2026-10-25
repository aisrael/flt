# Types

Types in `flt` are checked at runtime. Every value has exactly one concrete type.

## Built-In Types

| Type      | Example values        | Notes                                                              |
| --------- | --------------------- | ------------------------------------------------------------------ |
| `None`    | `None`                | The `None` type (and value); see [None](./types/none.md)           |
| `Boolean` | `true`, `false`       |                                                                    |
| `Number`  | `42`, `3.14`          | Arbitrary-precision decimal                                        |
| `String`  | `"hi"`                |                                                                    |
| `Symbol`  | `:id`                 |                                                                    |
| `Array`   | `[1, 2, 3]`           |                                                                    |
| `Map`     | `{ foo: 1 }`          |                                                                    |
| `Type`    | `Number`, `String`    | The type of a type value; see [Types as Values](#types-as-values)  |

## More About Types

- [None](./types/none.md): the `None` value and type, and the result of an `if` without `else`.
- [Union Types](./types/union_types.md): `A | B`, a value of one of several types, and the type of an `if`.
- [Native Types](./native-functions-and-values.md#native-types-and-values): types a host defines for its own values, which only its native functions create and read.

## Types as Values

Types are first-class values. The name of a built-in type evaluates to that type, and types print without quotes:

```flt
Number                 # Number
Number == Number       # true
Number == String       # false
```

[`None`](./types/none.md) is both a value and a type, and the two compare equal.

A built-in type name can't be bound, so it always means the type ([ADR 006](./adrs/006-no-shadowing.md)). That includes [`Any`](#any):

```flt
let Number = 5      # error: Cannot bind Number: it's a built-in type
let None = 1        # error: Cannot bind None: it's a built-in type
```

A host's [native types](./native-functions-and-values.md#native-types-and-values) are first-class values the same way.

A name that is neither a variable, a function nor a type is an error, e.g. `Foo` raises `Unbound identifier: Foo`. This includes [`Any`](#any), which only appears in signatures.

## `Any`

`Any` is a placeholder used only in type and function signatures, where it means "accepts any value". For example, a function that takes one argument of any type and returns its type has the type `fn(Any) -> Type`.

`Any` is also what a declaration gets when no type is given. This applies to a variable, a parameter, or a function's return type.

`Any` is not a real type. No expression ever evaluates to a value of type `Any`. `Any` can't be used as a value either: `Any` on its own raises `Unbound identifier: Any`.

## Function Types

A function's type is written `fn(P1, P2) -> R`, listing its parameter types and its return type. A function that takes no parameters has the type `fn() -> R`.

`->` groups before `|`, so a union return type needs brackets: `fn(T) -> (T | None)` returns a `T` or `None`, while `fn(T) -> T | None` is either a function or `None`.

Functions are first-class values. With a host that registers a [native function](./native-functions-and-values.md#native-functions) `identity`, which returns its argument and has the type `fn(Any) -> Any`:

```flt
identity              # <fn identity>
let f = identity
f(42)                 # 42
f == identity         # true
```

A function value displays as `<fn name>`. Two function values are equal only if they're the same function.

## Collection Types

- `Map`
- `KeywordArgs`

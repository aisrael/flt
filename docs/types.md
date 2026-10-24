# Types

Types in `flt` are checked at runtime. Every value has exactly one concrete type, which [`typeof`](./functions-and-pipe.md#built-in-functions) returns.

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
| `Type`    | `Number`, `typeof(1)` | The type of a type value; see [Types as Values](#types-as-values)  |

## More About Types

- [None](./types/none.md): the `None` value and type, and the result of an `if` without `else`.

## Types as Values

Types are first-class values. The name of a built-in type evaluates to that type, and types print without quotes:

```flt
Number                 # Number
typeof(1)              # Number
typeof(1) == Number    # true
typeof("a") == Number  # false
typeof(Number)         # Type
typeof(typeof(1))      # Type
```

[`None`](./types/none.md) is both a value and a type, and the two compare equal, so `typeof(None) == None` is `true`.

A `let` binding shadows a built-in type name. There are no scopes yet, so the binding applies everywhere after it:

```flt
let Number = 5
typeof(Number)      # Number, the type of 5
```

A name that is neither a variable nor a built-in type is an error, e.g. `Foo` raises `Unbound identifier: Foo`. This includes [`Any`](#any), which only appears in signatures.

## `Any`

`Any` is a placeholder used only in type and function signatures, where it means "accepts any value". For example, `typeof` has the type `fn(Any) -> Type`.

`Any` is also what a declaration gets when no type is given. This applies to a variable, a parameter, or a function's return type.

`Any` is not a real type. No expression ever evaluates to a value of type `Any`, so `typeof` never returns `Any`. `Any` can't be used as a value either: `Any` on its own raises `Unbound identifier: Any`.

## Collection Types

- `Map`
- `KeywordArgs`

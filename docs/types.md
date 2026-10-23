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
- [Union Types](./types/union_types.md): `A | B`, a value of one of several types, and the type of an `if`.
- [Function Types](./types/functions.md): `fn(A, B) -> R`, and brackets around union return types.

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

`<>` joins a type's name:

```flt
"type: " <> typeof(1)   # "type: Number"
```

A `let` binding shadows a built-in type name. There are no scopes yet, so the binding applies everywhere after it:

```flt
let Number = 5
typeof(Number)      # Number, the type of 5
```

A name that is neither a variable nor a built-in type is an error, e.g. `Foo` raises `Unbound identifier: Foo`. This includes [`Any`](#any), which only appears in signatures.

## `Any`

`Any` is a placeholder used only in type and function signatures, where it means "accepts any value". For example, `typeof` has the type `fn(Any) -> Type`.

`Any` is also what a declaration gets when no type is given. This applies to a variable, a parameter, or a function's return type. Writing `-> Any` explicitly means the same as leaving the return type out. User-defined functions aren't supported yet, but they will work like this:

```flt
def foo(a, b)   # fn(Any, Any) -> Any
foo(1, "x")     # returns a value with a real type, e.g. Number; never Any
```

`Any` is not a real type. No expression ever evaluates to a value of type `Any`, so `typeof` never returns `Any`. `Any` can't be used as a value either: `Any` on its own raises `Unbound identifier: Any`.

## Wrapper Types

### Option

`Option` is an internal type that acts like Rust's generic `Option<T>` type, without formally introducing generics to `flt`.

That is, given a function:

```flt
def read(path: String, compression: ?CompressionType)
```

Then the internal type of `compression` is `Option<CompressionType>`.

`Option<T>` is either a value of the wrapped type `T`, or `None`. It is not the same as the union `T | None`; see [`None` and `Option<T>`](./types/none.md#none-and-optiont). To check if an `Option<T>` has an actual value, compare it to `None`, or use the `?` post-fix operator:

```flt
def read(path: String, compression: ?CompressionType)
  if compression?
    // handle compressed file
  else
    // read file regularly
  end
end
```

Alternatively

```flt
def read(path: String, compression: ?CompressionType)
  if compression == None
    // read file regularly
  else
    // handle compressed file
  end
end
```

## Collection Types

- `Map`
- `KeywordArgs`

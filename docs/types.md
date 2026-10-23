# Types

`flt` has a handful of built-in types:

## Built-In Types

- `None` - the type of the `None` value, e.g. the result of an `if` with no matching `else` when its condition is `false`
- `Boolean`
- `Number`
- `String`
- `Symbol`
- `Array`
- `Map`
- `Type` - the type of a type value, e.g. `typeof(Number)` is `Type`

## `Any`

`Any` is a placeholder used only in type and function signatures, where it means "accepts any value". For example, [`typeof`](./functions-and-pipe.md#built-in-functions) has the type `fn(Any) -> Type`.

`Any` is also what a declaration gets when no type is given. This applies to a variable, a parameter, or a function's return type. Writing `-> Any` explicitly means the same as leaving the return type out. User-defined functions aren't supported yet, but they will work like this:

```flt
def foo(a, b)   # fn(Any, Any) -> Any
foo(1, "x")     # returns a value with a real type, e.g. Number; never Any
```

`Any` is not a real type. No expression ever evaluates to a value of type `Any`, so `typeof` never returns `Any`. `Any` can't be used as a value either: `Any` on its own raises `Unbound identifier: Any`.

## Types as Values

Types are first-class values. The name of a built-in type evaluates to that type, and types print without quotes:

```flt
Number              # Number
typeof(1)           # Number
typeof(1) == Number # true
typeof("a") == Number # false
typeof(Number)      # Type
typeof(typeof(1))   # Type
```

`None` is both a value and a type, and the two compare equal:

```flt
typeof(None)          # None
typeof(None) == None  # true
```

`<>` joins a type's name:

```flt
"type: " <> typeof(1)   # "type: Number"
```

A `let` binding shadows a built-in type name. There are no nested scopes yet, so the binding applies everywhere after it:

```flt
let Number = 5
typeof(Number)      # Number, the type of 5
```

A name that is neither a variable nor a built-in type is an error, e.g. `Foo` raises `Unbound identifier: Foo`. This includes [`Any`](#any), which only appears in signatures.

## Union Types

A union type `A | B` is a value of one of the member types. For example, an `if` with no `else` has type `T | None`. See [Union Types](./types/union_types.md).

## Function Types

A function type such as `fn(String, Number) -> String` describes a function's parameter and return types. See [Function Types](./types/functions.md).

## Wrapper Types

### Option

The `Option` is an internal type acts like Rust's generic `Option<T>` type, without formally introducing generics to `flt`.

That is, given a function:

```flt
def read(path: String, compression: ?CompressionType)
```

Then the internal type of `compressed` is `Option<CompressionType>`.

`Option<T>` is either a value of the wrapped type `T`, or `None`. To check if an `Option<T>` has an actual value, compare it to `None`, or use the `?` post-fix operator:

```flt
def read(path: String, compressed: ?CompressionType)
  if compressed?
    // handle compressed file
  else
    // read file regularly
  end
end
``` 

Alternatively

```flt
def read(path: String, compressed: ?CompressionType)
  if compressed == None
    // read file regularly
  else
    // handle compressed file
  end
end
``` 

## Collection Types

- `Map`
- `KeywordArgs`
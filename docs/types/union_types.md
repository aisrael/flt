# Union Types

A union type `A | B` describes a value that is one of the member types: either an `A` or a `B`.

```flt
Number | String           # a Number, or a String
Number | String | None    # a Number, a String, or None
```

## The Type of `if`

An `if` with an `else` has the type `<then type> | <else type>`. An `if` without an `else` evaluates to [`None`](./none.md) when its condition is `false`, so it has type `<then type> | None`:

| Expression              | Type              |
| ----------------------- | ----------------- |
| `if b 1`                | `Number \| None`  |
| `if b 1 else "one"`     | `Number \| String` |
| `if b 1 else 2`         | `Number`          |

Types are checked at runtime. [`typeof`](../functions-and-pipe.md#built-in-functions) reports the type of the value an expression actually produced, which is always one of the union's members:

```flt
typeof(if true 1)            # Number
typeof(if false 1)           # None
typeof(if false 1 else "a")  # String
```

## Simplification

flt keeps union types in a simple, canonical form:

- **Duplicates are removed.** `Number | Number` is just `Number`, which is why `if b 1 else 2` has type `Number`.
- **Nested unions are flattened.** `(Number | String) | None` is `Number | String | None`.
- **Order is kept.** Members print in the order they first appear, e.g. the then-branch's type before the else-branch's.

## Unions in Function Types

In a [function type](./functions.md), `->` groups before `|`, so a function that returns a union needs brackets around the return type:

```flt
fn(String) -> (String | None)    # a function that returns a String or None
fn(String) -> String | None      # either a function that returns a String, or None
```

## Not Yet Supported

flt source code can't yet contain type annotations, so you can't write a union type in a program yet. The following are planned:

- **Overloads.** The type of an overloaded function will be the union of its signatures. For example, given `foo(String) -> String` and `foo(String, Number) -> String`, the type of `foo` would be `fn(String) -> String | fn(String, Number) -> String`.
- **Generics.** A function `random_choice(a: A, b: B)` returns `A | B`. So `random_choice(0, 1)` would be a `Number`, and `random_choice("S", None)` would be a `String | None`.

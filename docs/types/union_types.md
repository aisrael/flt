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

These are inferred types. flt doesn't infer types yet (see [ADR 002](../adrs/002-union-types.md)), so they can't be observed from a program. Types are checked at runtime: [`typeof`](../functions-and-pipe.md#built-in-functions) reports the type of the value an expression actually produced, which is always one of the union's members:

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

## Not Yet Supported

flt source code can't yet contain type annotations, so you can't write a union type in a program yet.

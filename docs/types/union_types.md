# Union Types

A union type `A | B` describes a value that is one of the member types: either an `A` or a `B`.

```flt
Number | String           # a Number, or a String
Number | String | None    # a Number, a String, or None
```

## The Type of `if`

An `if` with an `else` has the type `<then type> | <else type>`. An `if` without an `else` evaluates to [`None`](./none.md) when its condition is `false`, so it has type `<then type> | None`:

| Expression          | Type                |
| ------------------- | ------------------- |
| `if b 1`            | `Integer \| None`   |
| `if b 1 else "one"` | `Integer \| String` |
| `if b 1 else 2`     | `Integer`           |
| `if b 1 else 2.0`   | `Integer \| Number` |

These are inferred types. flt doesn't infer types yet (see [ADR 002](../adrs/002-union-types.md)), so they can't be observed from a program. Types are checked at runtime: [`typeof`](../functions-and-pipe.md#built-in-functions) reports the type of the value an expression actually produced, which is always one of the union's members:

```flt
typeof(if true 1)            # Integer
typeof(if false 1)           # None
typeof(if false 1 else "a")  # String
```

## Simplification

flt keeps union types in a simple, canonical form:

- **Duplicates are removed.** `Integer | Integer` is just `Integer`, which is why `if b 1 else 2` has type `Integer`. `Integer | Number` isn't simplified to `Number`, though `Number` accepts integers.
- **Nested unions are flattened.** `(Number | String) | None` is `Number | String | None`.
- **Order is kept.** Members print in the order they first appear, e.g. the then-branch's type before the else-branch's.

## Writing a Union Type

A union type can be written in a [type annotation](../types.md#type-annotations) on a `let` binding:

```flt
let x: Number | None = None
x = 1                    # ok: x is 1.0, since Number accepts an Integer
x = "a"                  # error: Variable x expects Number | None, found String
```

Parentheses group a union, e.g. `(Number | String) | None`, which simplifies to `Number | String | None`.

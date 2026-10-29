# Types

Types in `flt` are checked at runtime. Every value has exactly one concrete type.

## Built-In Types

| Type      | Example values        | Notes                                                              |
| --------- | --------------------- | ------------------------------------------------------------------ |
| `None`    | `None`                | The `None` type (and value); see [None](./types/none.md)           |
| `Boolean` | `true`, `false`       |                                                                    |
| `Integer` | `42`, `-7`            | Arbitrary-precision whole number; see [Integers and Numbers](#integers-and-numbers) |
| `Number`  | `3.14`, `42.0`        | Arbitrary-precision decimal; also accepts an `Integer`, converting it |
| `String`  | `"hi"`                |                                                                    |
| `Symbol`  | `:id`                 |                                                                    |
| `Array`   | `[1, 2, 3]`           |                                                                    |
| `Map`     | `{ foo: 1 }`          |                                                                    |
| `Type`    | `Number`, `String`    | The type of a type value; see [Types as Values](#types-as-values)  |

## More About Types

- [None](./types/none.md): the `None` value and type, and the result of an `if` without `else`.
- [Union Types](./types/union_types.md): `A | B`, a value of one of several types, and the type of an `if`.
- [Native Types](./native-functions-and-values.md#native-types-and-values): types a host defines for its own values, which only its native functions create and read.

## Integers and Numbers

flt has two numeric types ([ADR 017](./adrs/017-integers.md)). An `Integer` is a whole number, and a `Number` a decimal. Both are arbitrary-precision. A numeric [literal](./literals.md#numbers) without a `.` is an `Integer`, and one with a `.` is a `Number`:

```flt
typeof(1)          # Integer
typeof(1.5)        # Number
typeof(42.)        # Number
```

Arithmetic on two integers gives an `Integer`, and on a `Number` and anything else a `Number`. That includes `/`, which floors on integers, so `7 / 2` is `3`. Write `7.0 / 2` to get `3.5`. See [Integer and Number arithmetic](./operators.md#integer-and-number-arithmetic).

A `Number` with no digits after the decimal point prints with `.0`, so it never looks like an `Integer`:

```flt
4.0 / 2            # 2.0
42.                # 42.0
```

An `Integer` and a `Number` with the same value are equal, but `typeof` tells them apart:

```flt
1 == 1.0                   # true
typeof(1) == typeof(1.0)   # false
```

### `Number` Accepts an `Integer`

Anything that takes a `Number` also takes an `Integer`, and converts it to a `Number`. This applies to a function's arguments and result, and to a variable declared `Number`:

```flt
let n: Number = 1
n                  # 1.0
typeof(n)          # Number
```

It's the only conversion flt makes without being asked. An `Integer` never accepts a `Number`, not even a whole one:

```flt
let i: Integer = 1.0     # error: Variable i expects Integer, found Number
```

A union converts an `Integer` only if none of its members accepts it as it is:

| Declared type         | `1` becomes | `1.0` becomes |
| --------------------- | ----------- | ------------- |
| `Integer`             | `1`         | an error      |
| `Number`              | `1.0`       | `1.0`         |
| `Integer \| Number`   | `1`         | `1.0`         |
| `Number \| String`    | `1.0`       | `1.0`         |
| `Number \| None`      | `1.0`       | `1.0`         |
| `Any`                 | `1`         | `1.0`         |

## Type Annotations

A `let` binding can declare its variable's type after a `:` ([ADR 016](./adrs/016-typed-let-bindings.md)):

```flt
let n: Number = 1
let x: Number | String | None = None
let t: Type = Number
let a: Any = 1
```

The value must be of that type, by the same rules as a function's arguments, or the binding raises `Variable <name> expects <type>, found <type>`. It's converted by the same rules too, so `let n: Number = 1` binds `1.0` ([`Number` Accepts an `Integer`](#number-accepts-an-integer)). A later assignment, `n = …`, is checked against the declared type too, until the variable is declared again with `let`. See [Statements](./syntax.md#statements).

A type is a built-in type name, including `Any` and `None`, a [native type](./native-functions-and-values.md#native-types-and-values)'s name, a union `A | B`, or a type in parentheses. Function types and `Option<T>` can't be written yet; write `T | None` for `Option<T>`. Any other name raises `Unknown type: <name>`, before the value is evaluated:

```flt
let x: Foo = 1       # error: Unknown type: Foo
let y: typeof = 1    # error: Unknown type: typeof
```

`typeof` reports a value's own type, not its variable's declared type:

```flt
let x: Integer | String = 1
typeof(x)            # Integer
```

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

A function's type is written `fn(P1, P2) -> R`, listing its parameter types and its return type. A function that takes no parameters has the type `fn() -> R`. A [variadic](./native-functions-and-values.md#variadic-arguments) last parameter shows as `*T`, the type of each value it takes, so `fn(String, *Any) -> String` takes a string and then any number of values ([ADR 012](./adrs/012-variadic-notation.md)). Rest keywords show as `**T`.

A union is bracketed wherever `|` would otherwise be read the wrong way:

- After `*` or `**`, since `*` and `**` apply to the whole union: `fn(*(String | Symbol)) -> String` takes any number of strings and symbols, and `fn(**(String | Symbol)) -> Map` takes keywords whose values are strings or symbols.
- Around a return type, since `->` groups before `|`: `fn(T) -> (T | None)` returns a `T` or `None`, while `fn(T) -> T | None` is either a function or `None`.

A union parameter or keyword is left as it is, since the commas and the function's own brackets already separate it: `fn(String | Symbol, k: Number | None) -> String`.

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

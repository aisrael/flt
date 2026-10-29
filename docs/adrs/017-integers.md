# ADR 017: Integers

- **Date:** 2026-10-28
- **Status:** Proposed

## Context

flt has one numeric type, `Number`, an arbitrary-precision decimal. Every numeric literal is a
`Number`, whether it's `1` or `1.5`.

Many arguments only make sense as whole numbers: a row count, a column width, the `n` in datu's
`head(path, n: Number)`. A signature can't say so. A handler that reads `n` as a `usize` fails
only when it runs, with `Cannot convert 2.5 to usize`. Neither the signature, the type check
([ADR 013](013-type-checked-function-calls.md)) nor `typeof` can tell a whole number from any
other.

flt needs an integer type that a signature can declare and a call can be checked against. Then
`foo(i: Integer)` should accept `foo(1)`. Every existing `Number` parameter should still accept
`1` too, or adding the type would break every host.

Before deciding, we compared how Elixir, Python, Ruby and Rust handle integers and
non-integers. The table shows the results that matter here:

| Expression | Elixir | Python | Ruby | Rust |
| --- | --- | --- | --- | --- |
| `7 / 2` | `3.5` | `3.5` | `3` | `3` |
| `-7 / 2` | `-3.5` | `-3.5` | `-4` (floors) | `-3` (truncates) |
| `-7 // 2`, or floor division | `Integer.floor_div` → `-4`; `div` → `-3` | `-4` | `-7.div(2)` → `-4` | `div_floor` → `-4` |
| `1 / 0` | error | error | error | panic |
| `1 + 1.0` | `2.0` | `2.0` | `2.0` | doesn't compile |
| `1 == 1.0` | `true` | `True` | `true` | doesn't compile |
| a whole float prints as | `2.0` | `2.0` | `2.0` | `2` (`{}`), `2.0` (`{:?}`) |
| a float where an integer is expected | error | error | error | doesn't compile |

The three dynamic languages all convert an integer to a float when an operation mixes the two,
and none converts a float to an integer implicitly. Python's type hints let a `float` parameter
take an `int` (PEP 484).

## Decisions

### 1. `Integer` is a built-in type

`Integer` is a built-in type of whole numbers, with arbitrary precision. Like every built-in type,
its name evaluates to the type, and can't be bound ([ADR 006](006-no-shadowing.md)).

A numeric literal without a `.` is an `Integer`. A numeric literal with a `.` is a `Number`, even
if it has no digits after the `.`:

```flt
typeof(1)          # Integer
typeof(-7)         # Integer
typeof(1_000)      # Integer
typeof(1.5)        # Number
typeof(42.0)       # Number
typeof(42.)        # Number
```

Before this ADR, `42.` and `42` were the same value.

### 2. Integer arithmetic results in an integer

| Operation | `Integer`, `Integer` | `Integer` and `Number`, either way round | `Number`, `Number` |
| --- | --- | --- | --- |
| `+`, `-`, `*` | `Integer` | `Number` | `Number` |
| `/` | `Integer`, floored | `Number` | `Number` |
| `//` | `Integer`, floored | `InvalidOperandType` | `InvalidOperandType` |
| unary `-`, `+` | `Integer` | | `Number` |

`/` on two integers rounds the quotient toward negative infinity, as Ruby does, so `7 / 2` is `3`
and `-7 / 2` is `-4`. To get a fraction, an operand has to be a `Number`: `7.0 / 2` is `3.5`.
Unary `-` binds tighter than `/`, so `-7 / 2` is `(-7) / 2`.

```flt
1 + 2              # 3
1 + 2.0            # 3.0
7 / 2              # 3
-7 / 2             # -4
7 / -2             # -4
7.0 / 2            # 3.5
9223372036854775807 + 1    # 9223372036854775808
```

Integers never overflow.

### 3. `//` is integer floor division

`x // y` divides two integers and rounds the quotient toward negative infinity, like Python's `//`
and Elixir's `Integer.floor_div/2`. It has the same precedence as `*` and `/`, and is
left-associative.

`//` takes only integers, for now. An operand that's a `Number`, or anything else that isn't an
`Integer`, raises an `InvalidOperandType` error:

```flt
7 // 2             # 3
-7 // 2            # -4
7 // -2            # -4
7.5 // 2           # InvalidOperandType
7 // 2.0           # InvalidOperandType
```

On two integers, `//` gives the same result as `/`. What `//` adds is the guarantee that its
result is an `Integer`, or it fails. It doesn't depend on the operands happening to be integers.

### 4. Dividing by zero is an error

`/` with a zero divisor, `0` or `0.0`, raises a `DivisionByZero` error, as it does today. So does
`//` with a divisor of `0`, after its operands' types are checked, so `1.0 // 0` raises
`InvalidOperandType`. Neither type has an infinity or a "not a number".

### 5. Integers and numbers compare by value

`==`, `!=`, `<`, `>`, `<=` and `>=` compare an `Integer` with a `Number` by value. That holds inside
arrays and maps too:

```flt
1 == 1.0           # true
1 < 1.5            # true
[1] == [1.0]       # true
{a: 1} == {a: 1.0} # true
```

`typeof` still tells them apart: `typeof(1) == typeof(1.0)` is `false`.

### 6. `Number` accepts an `Integer`, and converts it

A `Number` parameter accepts an `Integer`, and so does a variable declared `Number`. The value is
converted to a `Number` when it's accepted. This is the only implicit conversion between flt's
types. `Integer` never accepts a `Number`, not even a whole one like `1.0`.

The conversion applies wherever a type is checked:

- a positional argument, a variadic value or a keyword value (ADR 013, decision 1);
- a function's result, against its return type (ADR 013, decision 6);
- a declaration, `let n: Number = 1`, and an assignment to a variable declared with a type
  ([ADR 016](016-typed-let-bindings.md), decisions 1 and 3).

A union, or an `Option`, converts a value only if none of its members accepts it as it is:

| Declared type | `1` is bound as | `1.0` is bound as |
| --- | --- | --- |
| `Integer` | `1` | an error |
| `Number` | `1.0` | `1.0` |
| `Integer \| Number` | `1` | `1.0` |
| `Number \| String` | `1.0` | `1.0` |
| `Option<Number>`, or `Number \| None` | `1.0` | `1.0` |
| `Any` | `1` | `1.0` |

```flt
let i: Integer = 1         # ok
let i: Integer = 1.0       # VariableTypeMismatch
let n: Number = 1
n                          # 1.0
typeof(n)                  # Number
n = 2                      # ok: n is 2.0
let k: Integer | String = 1
typeof(k)                  # Integer
let x = 1
typeof(x)                  # Integer: x has no declared type
```

This changes three earlier rules:

- **ADR 013, decision 2**, said a built-in type accepts values of that type, and that there's no
  subtyping. Now `Number` accepts `Integer` values too. Function types still have no subtyping: a
  `fn(Number) -> Number` parameter doesn't accept a `fn(Integer) -> Integer`.
- **ADR 016, decision 1**, bound a declared variable's value "as it is". Now an `Integer` bound
  to a variable declared `Number` is converted.
- **ADR 016 rejected converting a value to the declared type**, since no conversion between flt's
  types was implicit. This conversion, from `Integer` to `Number` only, is now the one exception.

### 7. A whole `Number` prints with `.0`

A `Number` with no digits after the decimal point prints with a trailing `.0`, so that it never
looks like an `Integer`, and reads back as a `Number`:

```flt
4.0 / 2            # 2.0
42.                # 42.0
-0.0               # 0.0
1 + 2.0            # 3.0
2.50 * 2           # 5.00
```

A `Number` that has digits after the decimal point prints them as before. `<>` and string
interpolation convert a number to a string the same way, so `"x" <> 2.0` is `"x2.0"`.

### 8. Overload ties are still unspecified; `SimpleRuntime` picks the narrowest

With `f(n: Number)` and `f(i: Integer)`, both overloads accept `f(1)`. That's a tie, and ADR 013's
decision 4 still applies: the spec doesn't say which overload is called, and flt source shouldn't
depend on it.

`SimpleRuntime` changes how it breaks ties. It used to call the overload declared first. Now it
calls the **narrowest** overload that matches:

- One declared type is at least as narrow as another if they're equal, or the other is `Any`, or
  it's `Integer` and the other is `Number`, or it's a member of the other, a union. A union is at
  least as narrow as a type if each of its members is. `T` and `None` are at least as narrow as
  `Option<T>`.
- One overload is narrower than another if, for each argument the call passes, its type is at
  least as narrow as the other's, and for at least one argument it's strictly narrower.
- If one matching overload is narrower than every other, it's called. Otherwise the one declared
  first is called, as before.

```
f(n: Number) -> String
f(i: Integer) -> String
show(x: Any) -> String
show(x: Number) -> String
g(a: Integer, b: Number) -> String
g(a: Number, b: Integer) -> String
```

```flt
f(1)           # f(i: Integer)
f(1.0)         # f(n: Number): the only match
show(1.0)      # show(x: Number), though show(x: Any) was declared first
g(1, 2)        # neither is narrower, so the first declared: g(a: Integer, b: Number)
```

The overload is chosen before any value is converted, and the chosen overload's types decide the
conversion. So `f(1)` passes `1` to `f(i: Integer)` as an `Integer`.

This is how `SimpleRuntime` narrows. It isn't a language rule, and it doesn't change how overloads
are tried by arity (ADR 013, decision 3) or which error a call that matches no overload gets
([ADR 015](015-closest-overload-errors.md)).

### 9. The rules are the language's, and the Rust API

`Integer`, its literals, the arithmetic and `//`, comparing by value, `Number` accepting and
converting an `Integer`, and the way whole numbers print are part of the language, and go in the
spec. Narrowest-overload selection, and the precision of `Number` division (100 significant digits
in `SimpleRuntime`), are `SimpleRuntime`'s.

In the Rust API:

- `Value::Integer(BigInt)` is a new variant. `Value::from` turns a Rust `u64`, `i64`, `usize` or a
  `BigInt` into an `Integer`, and a `BigDecimal` into a `Number`.
- `Type::integer()` and `Argument::integer(name)` declare the type.
- A handler reads an argument with `args.get` as:
  - `BigInt`, from an `Integer` or a whole `Number`;
  - `BigDecimal`, from a `Number` or an `Integer`.
  
  The Rust integer types (`i32`, `i64`, `u32`, `u64`, `usize`) convert from an `Integer` or a
  whole `Number`, and `f64` from either.
- A handler for a `Number` parameter always gets a `Value::Number`, since an `Integer` argument is
  converted first. A handler declared `-> Number` may return a `Value::Integer`, which is converted
  too.
- flt re-exports `BigInt` and `BigDecimal`, so a host can use them without depending on the same
  versions of `num-bigint` and `bigdecimal` itself.

## Alternatives Considered

- **`Integer` as a constraint on `Number`**, accepting any whole `Number`, with no values of its
  own. It's much less work, and `foo(1)` and `foo(1.0)` would both pass. Rejected: `Integer` would
  be like `Any`, a type that no value has, so `typeof` couldn't report it, and values wouldn't be
  stored as integers.
- **`Integer / Integer` results in a `Number`**, as in Elixir and Python, so `7 / 2` is `3.5`.
  Rejected: integer arithmetic stays integer, and an operand that's a `Number` asks for a fraction.
- **`/` truncates toward zero**, as in Rust and Elixir's `div`, so `-7 / 2` is `-3`. Rejected:
  flooring gives `/` and `//` the same result on integers, and a later `%` can be the floored
  modulo that pairs with both.
- **`//` on `Number`s.** Python's `7.5 // 2` is the float `3.0`, and Ruby's `7.5.div(2)` is the
  integer `3`. Deferred: `//` takes only integers for now, which is easy to relax later.
- **`Number` accepts an `Integer` without converting it**, as Python does with an `int` passed as
  a `float`. Rejected: a handler declared with `Number` would have to handle both kinds of value,
  and a variable declared `Number` could hold an `Integer`, so `typeof` would disagree with the
  declaration.
- **`Number` doesn't accept an `Integer`.** Rejected: every existing `Number` parameter and
  variable would reject `1`.
- **Narrowing as a language rule.** Deferred: with several arguments, two overloads can each be
  narrower for a different argument, and the spec would need a rule for that, such as an error.
  `SimpleRuntime` falls back on declaration order instead.
- **A whole `Number` prints without `.0`**, as it does today. Rejected: `4.0 / 2` and `2` would
  print the same, and the printed `2` would read back as an `Integer`.

## Consequences

- flt has a second numeric type, and a new operator, `//`.
- **Breaking changes:**
  - Integer literals are `Integer`s, so `typeof(1)` is `Integer`, and `typeof(1) == Number` is
    `false`.
  - `7 / 2` is `3`, not `3.5`. Source that divides integer literals and expects a fraction has to
    make an operand a `Number`, e.g. `7.0 / 2`.
  - `42.` is a `Number`, and prints `42.0`.
  - A whole `Number` prints with `.0`, e.g. `4.0 / 2` prints `2.0`.
  - A value bound to a variable declared `Number`, passed to a `Number` parameter, or returned from
    a `-> Number` function is a `Number`, so an integer prints with `.0`: `let n: Number = 1` then
    `n` prints `1.0`. A host function declared `-> Number` that returns a count, like datu's
    `count(plan)`, prints `5.0` instead of `5`, and should be declared `-> Integer`.
  - In the Rust API, `Value::from` turns Rust integers into `Value::Integer`, so a host that
    matches `Value::Number(n)` misses them.
  - `SimpleRuntime` breaks overload ties by narrowness, not declaration order, so with
    `show(x: Any)` declared before `show(x: Number)`, `show(1.0)` calls the second.
- Later:
  - `//` on `Number`s;
  - `%`, the floored modulo that pairs with `/` and `//`;
  - `**`, and the bitwise operators `&`, `|` and `^` on integers, which parse today but raise
    `InvalidOperandType`;
  - functions that convert a `Number` to an `Integer`, such as `trunc`, `floor`, `ceil` and
    `round`, which need a rounding mode;
  - narrowing as a language rule.

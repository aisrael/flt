# Operators

## Unary Operators

| Operator | Meaning | Example |
| --- | --- | --- |
| `!` | logical NOT | `!true` |
| `+` | numeric unary plus | `+42` |
| `-` | numeric negation | `-42` |

## Binary Operators

| Operator | Category | Example |
| --- | --- | --- |
| `+` | arithmetic add | `1 + 2` |
| `-` | arithmetic subtract | `10 - 3` |
| `*` | arithmetic multiply | `4 * 5` |
| `/` | arithmetic divide | `20 / 4` |
| `//` | integer floor division | `7 // 2` |
| `<>` | string concat | `"foo" <> "bar"` |
| `&&` | logical and | `true && false` |
| `||` | logical or | `true || false` |
| `^^` | logical xor | `true ^^ false` |
| `&` | bitwise and (parsed) | `a & b` |
| `|` | bitwise or (parsed) | `a | b` |
| `^` | bitwise xor (parsed) | `a ^ b` |
| `|>` | [pipe](./functions-and-pipe.md#pipe-operator) | `x |> f` |
| `==` | equality | `x == 1` |
| `!=` | inequality | `x != 0` |
| `<` | less than | `a < b` |
| `>` | greater than | `a > b` |
| `<=` | less than or equal | `a <= b` |
| `>=` | greater than or equal | `a >= b` |

## Precedence (Low to High)

1. `||`
2. `&&`
3. `^^`
4. `==`, `!=`
5. `<`, `>`, `<=`, `>=`
6. `|`
7. `^`
8. `&`
9. `|>`
10. `+`, `-`, `<>`
11. `*`, `/`, `//`

All binary levels are left-associative, so `7 // 2 * 2` is `(7 // 2) * 2`.

Comparisons bind looser than arithmetic, string concatenation and the bitwise
operators, so `1 + 2 == 3` is `true` and `x & 1 == 0` is `(x & 1) == 0`.
The [pipe](./functions-and-pipe.md#pipe-operator) sits directly below `+`,
`-` and `<>`, as in Elixir, so `1 + 1 |> f` is `(1 + 1) |> f` and
`x |> f == 3` is `(x |> f) == 3`.

`.` ([field access](./field-access.md)) is a tight postfix operator that
binds even tighter than the unary operators above — it applies directly to
the primary expression before any unary or binary operator sees it.

## Integer and Number Arithmetic

flt has two numeric types, `Integer` and `Number` ([Integers and Numbers](./types.md#integers-and-numbers), [ADR 017](./adrs/017-integers.md)). The type of an arithmetic result depends on its operands' types:

| Operation | `Integer`, `Integer` | `Integer` and `Number`, either way round | `Number`, `Number` |
| --- | --- | --- | --- |
| `+`, `-`, `*` | `Integer` | `Number` | `Number` |
| `/` | `Integer`, floored | `Number` | `Number` |
| `//` | `Integer`, floored | error | error |
| unary `-`, `+` | `Integer` | | `Number` |

```flt
1 + 2              # 3
1 + 2.0            # 3.0
2 * 1.5            # 3.0
7 / 2              # 3
7.0 / 2            # 3.5
4.0 / 2            # 2.0
```

### Division

`/` on two integers gives an `Integer`, rounding the quotient down, toward negative infinity, as Ruby's `/` does. If either operand is a `Number`, `/` gives a `Number`:

```flt
7 / 2              # 3
-7 / 2             # -4
7 / -2             # -4
-7 / -2            # 3
7 / 2.0            # 3.5
```

Unary `-` binds tighter than `/`, so `-7 / 2` is `(-7) / 2`, which is `-4`, while `-(7 / 2)` is `-3`.

`SimpleRuntime` computes a `Number` quotient to 100 significant digits, so `1.0 / 3` is `0.333…3`, with 100 `3`s.

### Floor Division

`//` divides two integers and rounds down, like Python's `//` and Elixir's `Integer.floor_div/2`. On two integers it gives the same result as `/`, but it only takes integers, so its result is always an `Integer`. Any other operand, including a `Number`, raises `Invalid Operand Type`:

```flt
7 // 2             # 3
-7 // 2            # -4
7 // -2            # -4
7.5 // 2           # error: Invalid Operand Type
7 // 2.0           # error: Invalid Operand Type
```

### Dividing by Zero

`/` and `//` raise `Division By Zero` for a divisor of `0`, and `/` for `0.0` too. `//` checks its operands' types first, so `1.0 // 0` raises `Invalid Operand Type`:

```flt
1 / 0              # error: Division By Zero
1.0 / 0            # error: Division By Zero
1 // 0             # error: Division By Zero
```

### Comparing Integers and Numbers

`==`, `!=`, `<`, `>`, `<=` and `>=` compare an `Integer` with a `Number` by value, including the elements of arrays and the values of maps:

```flt
1 == 1.0           # true
1 < 1.5            # true
[1, 2] == [1.0, 2] # true
{a: 1} == {a: 1.0} # true
```

### Large Integers

Integers have arbitrary precision, so they never overflow:

```flt
9223372036854775807 + 1    # 9223372036854775808
```

## Parentheses

Parentheses override precedence:

```flt
(1 + 2) * 3
```

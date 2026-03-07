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
| `<>` | string concat | `"foo" <> "bar"` |
| `&&` | logical and | `true && false` |
| `||` | logical or | `true || false` |
| `^^` | logical xor | `true ^^ false` |
| `&` | bitwise and (parsed) | `a & b` |
| `|` | bitwise or (parsed) | `a | b` |
| `^` | bitwise xor (parsed) | `a ^ b` |
| `|>` | pipe | `x |> f` |

## Precedence (Low to High)

1. `|>`
2. `||`
3. `&&`
4. `^^`
5. `|`
6. `^`
7. `&`
8. `+`, `-`, `<>`
9. `*`, `/`

All binary levels are left-associative.

## Parentheses

Parentheses override precedence:

```flt
(1 + 2) * 3
```

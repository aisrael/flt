# Field Access

`flt` supports dot-access syntax for reading a field off a map value.

## Syntax

```flt
u.foo
{ foo: "bar" }.foo
f().x
a.b.c
```

- Field access is written as `expr.field`, where `field` is a bare
  identifier.
- It binds as a tight postfix suffix directly on the preceding expression:
  no whitespace or comments are allowed between the expression, the `.`,
  and the field name.
- Multiple `.field` suffixes chain left-to-right: `a.b.c` reads `b` off `a`,
  then `c` off the result.
- It applies at every expression precedence level, including after unary
  operators (`!u.active`) and inside `if` conditions (`if u.active { ... }`).

## Runtime Behavior

Field access only works on `Map` values:

- If the base expression evaluates to a `Map` and the field is present, the
  field's value is returned.
- If the field is missing, evaluation raises a runtime error:
  `No such field: <name>`.
- If the base expression is not a `Map`, evaluation raises
  `Invalid Operand Type`.

Examples:

```flt
let u = { name: "Ada" }
u.name        # "Ada"
u.missing     # error: No such field: missing
(1).foo       # error: Invalid Operand Type
```

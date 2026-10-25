# flt Language Documentation

This folder contains initial documentation for the `flt` language as it exists in this repository today.

## Contents

- [`spec.md`](spec.md) - One-page minimal language specification (grammar, types, operators, semantics).
- [`quickstart.md`](quickstart.md) - Running `flt` and trying expressions in the REPL.
- [`syntax.md`](syntax.md) - Core expression forms, literals, comments, identifiers, reserved keywords, and statements.
- [`types.md`](types.md) - Core flt types
  - [`types/none.md`](types/none.md) - `None`: the value, its type, and the result of `if` without `else`.
  - [`types/union_types.md`](types/union_types.md) - Union types (`A | B`) and the type of `if`.
- [`operators.md`](operators.md) - Unary/binary operators and precedence.
- [`field-access.md`](field-access.md) - Dot-access syntax for reading map fields (`u.foo`).
- [`functions-and-pipe.md`](functions-and-pipe.md) - Function call syntax (including keyword arguments) and pipe chaining.
- [`runtime-and-limitations.md`](runtime-and-limitations.md) - Evaluation behavior, errors, and current limitations.

## Architecture Decision Records

- [ADR 001: `None` Types](adrs/001-none-types.md)
- [ADR 002: Union Types](adrs/002-union-types.md)
- [ADR 003: Functions as Values](adrs/003-functions-as-values.md)
- [ADR 004: The Pipe Operator](adrs/004-pipe-operator.md)
- [ADR 005: `typeof` as a Native Function](adrs/005-typeof-native-function.md)
- [ADR 006: Bindings Can't Shadow Types, Functions or Keywords](adrs/006-no-shadowing.md)

## Scope

These docs describe the currently implemented parser and CLI evaluator for version `0.2.0`.

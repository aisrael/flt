# ADR 016: Typed `let` Bindings

- **Date:** 2026-10-28
- **Status:** Proposed

## Context

A function declares the types of its arguments, and every call is checked against them
([ADR 013](013-type-checked-function-calls.md)). A variable can't declare a type. Any value can be
bound to any name, and re-binding a name replaces its value and its type:

```flt
let n = 1
n = "two"        # fine: n is now a String
```

flt source can't contain a type at all. Types appear only in the signatures a host declares, and
[union types](../types/union_types.md) can't be written in a program.

`let x = e` and `x = e` mean the same thing today: both bind `x`, whether or not it's already
bound.

## Decisions

### 1. `let` takes an optional type

```flt
let n: Number = 1          # ok
let n: Number = "one"      # Variable n expects Number, found String
let s: String | None = None
```

The type is written after the name and a `:`. A value matches it if the type accepts it, by the
rules for a function argument (ADR 013, decision 2). The value is bound as it is.

Only `let` takes a type. `x: Number = 1` is a parse error: a type declares the variable, and
declaring is what `let` does (decision 3).

### 2. Types are written as in signatures, for now only names and unions

A type is:

- a built-in type name, including `Any` and `None`;
- the name of a native type the host registered ([ADR 007](007-native-types-and-values.md));
- a union of types, `A | B`;
- a type in parentheses.

```flt
let x: Number | String | None = None
let d: Date = today()                  # Date is a native type
let t: Type = Number
let a: Any = 1
```

Function types (`fn(…) -> …`) and `Option<T>` aren't supported yet. `T | None` is written instead
of `Option<T>`, and means the same thing. A name that isn't a type the runtime knows, such as
`Foo` or `typeof`, raises an `UnknownType` error. A variable holding a type can't be used as one,
so `let T = Number` then `let x: T = 1` raises `UnknownType`.

### 3. `let` declares, `=` assigns

- `let x = e` and `let x: T = e` **declare** `x`. A declaration replaces any earlier declaration
  of `x`, including its type. `let x = e` declares `x` without a type.
- `x = e` **assigns** to `x`. If `x` was declared with a type, the value must be of that type, or
  the assignment raises a `VariableTypeMismatch` error and `x` keeps its value. Otherwise the value
  replaces the binding, as it does today.
- Assigning to a name that isn't bound declares it without a type, as it does today.

```flt
let n: Number = 1
n = 2              # ok
n = "two"          # Variable n expects Number, found String
let n = "two"      # ok: n is declared again, without a type
n = :two           # ok
```

So the type sticks to the variable until it's declared again. Without types, `let` and `=` behave
as they do today.

### 4. The name and the type are checked before the value

For `let x: T = e`:

1. the name `x` is checked, as for any binding ([ADR 006](006-no-shadowing.md));
2. the type `T` is resolved;
3. `e` is evaluated;
4. the value is checked against `T`;
5. `x` is bound to the value, with the type `T`.

So `let x: Foo = f()` raises `UnknownType` without calling `f`, just as `let Number = f()` raises
`CannotBind` without calling it.

### 5. `typeof` shows the value's type

A declaration doesn't change the value, so `typeof` reports the value's own type, not the declared
one:

```flt
let x: Number | String = 1
typeof(x)          # Number
```

A binding still evaluates to the bound value.

### 6. The rules are the language's

Declarations, declared types, checking an assignment against its declaration, and the two new
errors, `VariableTypeMismatch` and `UnknownType`, are part of the language, and go in the spec. The
error messages are `SimpleRuntime`'s:

- `Variable <name> expects <type>, found <type>`, in the style of
  `Argument <argument> of <name> expects <type>, found <type>`;
- `Unknown type: <name>`.

### 7. The Rust API

- `Statement::Let` holds the declaration's name, its optional type and its expression, and the new
  `Statement::Assign` holds an assignment. Each displays as it was written, so `x = 1` displays as
  `x = 1` rather than `let x = 1`.
- A type in source is a `TypeExpr`, which holds names. A runtime resolves them, since only it knows
  its native types.
- `Runtime::set_variable` assigns: it's checked against the variable's declared type, if it has
  one. `Runtime::remove_variable` removes the declared type with the value, so the REPL's
  `/unset x` does too.

## Alternatives Considered

- **A type that checks only its own binding.** `let n: Number = 1` would be checked, but a later
  `n = "two"` wouldn't, and `let` and `=` would stay the same. Rejected: a declared type that any
  later binding can silently replace says little about the variable.
- **A type that sticks for good**, so that a later `let n = "two"` is also an error, and only
  `let n: T = …` changes it. Rejected: it leaves no way to start over with a name in the REPL short
  of `/unset`, and it makes `let x = e` mean "assign" for some variables and "declare" for others.
- **Types on assignments too** (`x: Number = 1`). Rejected: an assignment that changes the type
  would be a declaration under another name.
- **Converting a value to the declared type.** Rejected for now: no conversion between flt's
  types is implicit.
- **Resolving a type by evaluating it**, so that `let x: T = 1` works with `let T = Number`.
  Rejected for now: `|` on two types isn't defined, and a type that depends on a variable can't be
  checked before the program runs.

## Consequences

- flt source can contain types, including union types.
- **Breaking changes:**
  - In the Rust API, `Statement::Let` changes shape and `Statement::Assign` is new. Hosts that build
    or match statements need updating.
  - A statement written `x = e` displays as `x = e`, not `let x = e`.
  - flt source without types behaves as before.
- New errors: `VariableTypeMismatch` and `UnknownType`.
- Later:
  - function types and `Option<T>` in types;
  - declaring a variable without a value, `let t: T`, which would leave `t` unbound until it's
    assigned;
  - a way to see a variable's declared type;
  - a host API for declaring a typed variable.

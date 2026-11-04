# Runtime Errors

This page lists the errors that the reference runtime (`SimpleRuntime`) raises, and when it raises them. The language specification ([`spec.md`](spec.md)) names the errors and links each name to its section here. This page gives the message text and the cause.

## Kinds of error

`eval` returns an `flt::Error`. Its variants are:

| Variant | Message prefix | Cause |
| --- | --- | --- |
| `LexerError` | `Lexer Error:` | The source has a token that the lexer does not know. |
| `ParserError` | `Parser Error:` | The source does not parse. No statement runs. |
| `SyntaxError` | `Syntax Error:` | An identifier is not valid. The parser reports other syntax problems as `Parser Error`. |
| `RuntimeError` | `Runtime Error:` | A statement fails while it runs. The rest of this page describes these. |
| `InterpreterError` | `Interpreter Error:` | A construct is not yet implemented. |
| `HandlerError` | `Handler Error:` | A host handler uses the flt API wrongly. This is a bug in the host. See [handler errors](native-functions-and-values.md#handler-errors). |
| `Native` | (none) | A native function fails with the host's own error. The runtime shows it without change. |

## Runtime errors

Each section below is one `RuntimeError` variant. It gives the message and the cases in which the runtime raises the error. In a message, `<name>` is the name of a function or variable, and `<type>` is a type as flt prints it.

## Operators and expressions

### `InvalidOperandType`

**Message:** `Invalid Operand Type`

An operator gets operands that it does not support. The runtime also raises this error in two other cases:

- an `if` condition is not a `Boolean`;
- a field access reads a value that is not a map.

```flt
1 + true       # InvalidOperandType
!1             # InvalidOperandType
7.5 // 2       # InvalidOperandType: // takes only integers
if 1 { 2 }     # InvalidOperandType
```

### `DivisionByZero`

**Message:** `Division By Zero`

`/` has a divisor of `0` or `0.0`, or `//` has a divisor of `0`. For `//`, the runtime checks the types of the operands first.

```flt
7 / 0          # DivisionByZero
7.0 / 0.0      # DivisionByZero
7 // 0         # DivisionByZero
```

### `NoSuchField`

**Message:** `No such field: <field>`

A field access `m.field` reads a map that has no such key.

```flt
{ a: 1 }.b     # NoSuchField
```

### `CannotCompare`

**Message:** `Cannot compare <a> and <b>`

The variant exists for values that cannot be compared. The reference runtime does not raise it now.

## Names and bindings

### `UnboundIdentifier`

**Message:** `Unbound identifier: <name>`

An expression reads a name that has no binding.

```flt
y              # UnboundIdentifier
```

### `UnknownType`

**Message:** `Unknown type: <name>`

A type in a `let` or a `def` names something that is not a built-in type or a native type. The runtime checks the type before it evaluates the value.

```flt
let x: Foo = 1   # UnknownType
```

### `VariableTypeMismatch`

**Message:** `Variable <name> expects <type>, found <type>`

A `let` or an assignment gives a variable a value that is not of its declared type. A failed assignment leaves the old value.

```flt
let n: Number = 1
n = "two"          # VariableTypeMismatch
let i: Integer = 1.0   # VariableTypeMismatch
```

### `CannotBind`

**Message:** `Cannot bind <name>: it's <reason>`

A `let` or an assignment uses a name that is already a built-in type, a function or a native type. The reason is `a built-in type`, `a function` or `a native type`. See [ADR 006](adrs/006-no-shadowing.md).

```flt
let Number = 5     # CannotBind: it's a built-in type
let typeof = 1     # CannotBind: it's a function
```

## Calls

The runtime checks a call in this order: the number of positional arguments, then the types of the positional arguments, then the keywords in the order of the call. It raises the error of the first check that fails. When several signatures take that number of positional arguments, it raises the error of the closest signature. See [ADR 015](adrs/015-closest-overload-errors.md).

### `UndefinedFunction`

**Message:** `Undefined function: <name>`

A call uses a name that has no binding.

```flt
nope(1)        # UndefinedFunction
```

### `NotCallable`

**Message:** `<name> is not a function`

A call uses a name that is bound to a value that is not a function. The value can be a variable, or the name of a built-in type.

```flt
let x = 1
x(2)           # NotCallable
Number(2)      # NotCallable
```

### `InvalidPipeTarget`

**Message:** `Cannot pipe into <expr>`

The right side of `|>` is not a call or a bare name.

```flt
1 |> 2         # InvalidPipeTarget
```

### `ArgumentCountMismatch`

**Message:** `Function <name> expected <counts> argument(s), found <found>`

No signature of the function takes that number of positional arguments. `<counts>` lists the counts that the signatures take, for example `1`, `at least 1` or `1 or at least 3`.

```flt
mod(1)         # ArgumentCountMismatch: one argument is missing
mod(1, 2, 3)   # ArgumentCountMismatch: one argument is extra
```

### `ArgumentTypeMismatch`

**Message:** `Argument <argument> of <name> expects <type>, found <type>`

Only one signature takes that number of positional arguments, and a positional argument is not of the type of its parameter.

```flt
mod(1, "two")  # ArgumentTypeMismatch: "two" is not an Integer
```

### `UnexpectedKeywordArgument`

**Message:** `Function <name> got an unexpected keyword argument: <key>`

The call has a keyword that the signature does not declare, and the signature has no rest keywords.

```flt
greet("Ann", tone: "loud")   # UnexpectedKeywordArgument
```

### `KeywordTypeMismatch`

**Message:** `Keyword argument <key> of <name> expects <type>, found <type>`

The value of a keyword argument is not of the type of that keyword.

```flt
greet("Ann", greeting: 1)    # KeywordTypeMismatch
```

### `NoMatchingOverload`

**Message:** `No overload of <name> accepts (<type>, …)`

Several signatures take that number of positional arguments, none accepts the call, and none is closest. If one signature is closest, the runtime raises the error of that signature instead.

### `ReturnTypeMismatch`

**Message:** `Function <name> returned <type>, expected <type>`

The result of a function is not of the return type of the signature that the runtime called. This applies to functions that `def` declares and to native functions.

## Declaring functions with `def`

### `DuplicateOverload`

**Message:** `Function <name> already has the overload <type>`

A `def` has the same parameter types as an existing overload of the same name. See [ADR 018](adrs/018-distinct-overload-signatures.md).

### `NameAlreadyBound`

**Message:** `Cannot define <name>: it's <reason>`

A `def` uses a name that is a built-in type, a native type, a variable, or a function that a host registered.

### `DefNotAtTopLevel`

**Message:** `Cannot define <name> here: a function can only be declared at the top level`

A `def` is inside a block or inside the body of a function. See [ADR 021](adrs/021-block-scopes.md).

### `ReturnOutsideFunction`

**Message:** `Cannot return here: no function is running`

A `return` statement runs with no function call active. The parser rejects a `return` outside a `def` in source code, so only a host that builds statements itself can cause this. See [ADR 023](adrs/023-return-statement.md).

## Host API

These errors come from code that a Rust host writes. flt source cannot cause them.

### `CannotRegister`

**Message:** `Cannot register <kind> <name>: it's <reason>`

A host registers a function or a native type under a name that is in use. The kind is `function` or `native type`.

### `UnexpectedType`

**Message:** `Expected <type>, found <type>`

A native function reads an argument as a type that it is not, for example with `Arguments::get`, `Arguments::native` or `downcast_native`.

### `CannotConvert`

**Message:** `Cannot convert <number> to <type>`

A native function reads a number as a Rust number type that cannot hold it, for example `-1` as a `u32`, or `2.5` as any integer type.

### `DeclaredInFlt`

**Message:** `Function <name> is declared in flt; call it through the runtime`

A host calls a function that `def` declared with `NativeFunction::call`, not through the runtime.

## Errors that are not runtime errors

A parse error stops `eval` before any statement runs. Thus a `Parser Error` can never happen after a statement has run. The spec calls these cases syntax errors. I did not check each one against the parser:

- parameters in the wrong order, or a repeated parameter name;
- a positional argument after a keyword argument, or a repeated keyword;
- a reserved keyword as a function name;
- a whitespace-form call next to `|>`.

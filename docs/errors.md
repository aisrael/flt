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

Each section below is one `RuntimeError` variant. It gives the message and the cases in which the runtime raises the error. [Warnings](#warnings) are not errors. In a message, `<name>` is the name of a function or variable, and `<type>` is a type as flt prints it.

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

The body of a function declared with `def` does not see global variables ([ADR 027](adrs/027-function-visibility.md)), so reading one there raises this error too:

```flt
let counter = 0
def read()
  counter      # UnboundIdentifier
end
read()
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

A `let`, a `const` or an assignment uses a name that is already a built-in type, a function or a native type. The reason is `a built-in type`, `a function` or `a native type`. See [ADR 006](adrs/006-no-shadowing.md).

```flt
let Number = 5     # CannotBind: it's a built-in type
let typeof = 1     # CannotBind: it's a function
```

### `ConstantRebound`

**Message:** `Cannot rebind constant <name>`

A program or a host binds a name that is a constant in the same scope: a `let`, an assignment, a `const`, or a host's `set_variable` or `declare_constant`. An assignment raises it when the nearest binding of the name is a constant. The runtime checks this before it evaluates the expression. The constant keeps its value. See [ADR 026](adrs/026-const-in-the-reference-runtime.md).

```flt
const LIMIT = 10
LIMIT = 11            # ConstantRebound
let LIMIT = 11        # ConstantRebound
const LIMIT = 11      # ConstantRebound
{ LIMIT = 11 }        # ConstantRebound
```

## Calls

The runtime checks a call in this order: the number of positional arguments, then the types of the positional arguments, then the keywords in the order of the call. It raises the error of the first check that fails. When several signatures take that number of positional arguments, it raises the error of the closest signature. See [ADR 015](adrs/015-closest-overload-errors.md).

### `UndefinedFunction`

**Message:** `Undefined function: <name>`

A call uses a name that has no binding.

```flt
nope(1)        # UndefinedFunction
```

### `NonConstFunctionCall`

**Message:** `Function <name> is not const and cannot be called here`

The body of a `const def`, or a default of a keyword parameter, calls a function that is not `const`. See [ADR 029](adrs/029-const-functions-in-the-reference-runtime.md).

```flt
def stamp() -> Integer
  now()
end
const def bad() -> Integer
  stamp()        # NonConstFunctionCall
end
bad()
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

### `ConstOverloadMismatch`

**Message:** `Function <name> is const, so all of its overloads must be const` or `Function <name> is not const, so none of its overloads can be const`

A `def` adds an overload to a function declared with `const def`, or a `const def` adds an overload to a function declared with `def`. The runtime checks this before it checks for a duplicate overload. See [ADR 029](adrs/029-const-functions-in-the-reference-runtime.md).

```flt
const def f(n: Integer) -> Integer
  n
end
def f(s: String) -> String      # ConstOverloadMismatch
  s
end
```

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

## Warnings

A warning is not an error. The statement that gives it still runs, and `eval` still returns its value. `SimpleRuntime` collects warnings. A host reads and clears them with `SimpleRuntime::take_warnings`. The `flt` command and the REPL print each warning to standard error, as `Warning: <message>`.

### `UnreachableCode`

**Message:** `Unreachable code in <name>: statements after return never run`

A `def` declares a function whose body has a statement after a `return` in the same list of statements: the body, a brace block, or an arm of an `if`. The runtime checks the body when it runs the `def`, not at each call. It gives one warning for each list of statements, for the first statement after the `return`. See [ADR 024](adrs/024-unreachable-code-warning.md).

The check follows the syntax, not the flow of the program. A statement after an `if` that has a `return` in an arm is reachable, so it gets no warning.

```flt
def f() -> Integer
  return 1
  2                 # UnreachableCode
end

def g(c: Boolean) -> Integer
  if c
    return 1
  end
  2                 # no warning: the `if` may not run its arm
end
```

### `ConstantHidden`

**Message:** `Constant <name> is hidden by a new binding`

A `let` or a `const` in an inner scope, a block or a function call, uses the name of a constant in an outer scope. The new binding hides the constant until the inner scope ends. The constant keeps its value. A parameter that hides a constant gives no warning. See [ADR 026](adrs/026-const-in-the-reference-runtime.md).

```flt
const LIMIT = 10
{
  let LIMIT = 1     # ConstantHidden
  LIMIT             # 1
}
LIMIT               # 10
```

## Errors that are not runtime errors

A parse error stops `eval` before any statement runs. Thus a `Parser Error` can never happen after a statement has run. The spec calls these cases syntax errors. I did not check each one against the parser:

- parameters in the wrong order, or a repeated parameter name;
- a positional argument after a keyword argument, or a repeated keyword;
- a reserved keyword as a function name;
- a whitespace-form call next to `|>`.

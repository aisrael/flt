# Function Types

A function type describes a function's parameter types and return type:

```flt
fn(String, Number) -> String    # takes a String and a Number, returns a String
fn(Number) -> Boolean           # takes a Number, returns a Boolean
```

A function with no parameters has an empty parameter list:

```flt
fn() -> Boolean                 # e.g. flip_a_coin()
```

## Union Return Types

`->` groups before `|`. A function that returns a [union](./union_types.md) needs brackets around its return type:

| Type                             | Meaning                                            |
| -------------------------------- | -------------------------------------------------- |
| `fn(String) -> (String \| None)` | a function that returns a `String` or `None`       |
| `fn(String) -> String \| None`   | either a function that returns a `String`, or `None` |

## Not Yet Supported

User-defined functions can't be declared yet, and function types can't be written in flt source code. Function overloading is planned; see [Union Types](./union_types.md#not-yet-supported).

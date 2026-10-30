# Native Functions, Types and Values

A host application embedding `flt` extends it from Rust. It can register:

- [native functions](#native-functions), which flt source calls like any function;
- [native types](#native-types-and-values), for the host's own values, which its native functions create and read and flt source passes around.

A native function can also fail with the host's own [error](#host-errors). A handler that uses flt's API wrongly gets a [handler error](#handler-errors).

## Native Functions

A host can register native functions, written in Rust, with `runtime.register_function`, a method of the [`Runtime` trait](./runtime-and-limitations.md#the-runtime-trait). The runtime keeps them in its global scope, alongside the built-ins. A function's name then evaluates to a [function value](./types.md#function-types), which can be bound to another name and called through it.

```rust
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use flt::runtime::functions::Argument;
use flt::runtime::functions::Arguments;
use flt::runtime::functions::FunctionDefinition;
use flt::runtime::functions::NativeFunction;
use flt::runtime::types::Type;
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;
use flt::runtime::Value;
use flt::Error;

/// identity(v) returns its argument
fn identity(args: &Arguments) -> Result<Value, Error> {
    Ok(args.get::<&Value>("v")?.clone())
}

/// Makes a handler that counts its calls. A handler can be any `Fn`, so one built by a
/// function like this may capture state. Handlers are Send + Sync, so mutable state needs
/// Arc<Mutex<_>> or atomics rather than Rc<RefCell<_>>.
fn counter(calls: Arc<AtomicU64>) -> impl Fn(&Arguments) -> Result<Value, Error> + Send + Sync {
    move |_args| Ok(Value::from(calls.fetch_add(1, Ordering::SeqCst) + 1))
}

let mut runtime = SimpleRuntime::default();

// identity has type fn(Any) -> Any
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("identity", Type::any(), vec![Argument::new("v", Type::any())]),
    identity,
))?;

// count() returns how many times it has been called, and has type fn() -> Integer
let calls = Arc::new(AtomicU64::new(0));
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("count", Type::integer(), vec![]),
    counter(Arc::clone(&calls)),
))?;
```

Registering a function under a name that's already registered replaces it. That includes the [built-ins](./functions-and-pipe.md#built-in-functions): a host that registers its own `typeof` replaces the default one.

A function's name can't be bound as a variable, whether the function is a built-in or the host's. Registration returns a `Result`. In `SimpleRuntime`, `register_function` fails with `Cannot register function <name>: …` if the name is a built-in type name, such as `Number`, a native type's name, or is already bound as a variable, so register functions before evaluating any flt source. Binding a variable from Rust with `runtime.set_variable` refuses the same names as `let`, and returns an error. It assigns, as `name = value` does, so it also returns an error for a value that isn't of a variable's [declared type](./types.md#type-annotations). `runtime.check_bindable` reports whether a name can be bound without binding it. Other runtimes may have other rules ([Rules Belong to the Runtime](./runtime-and-limitations.md#rules-belong-to-the-runtime)).

```flt
identity              # <fn identity>
typeof(identity)      # fn(Any) -> Any
identity(42)          # 42
let f = identity
f(42)                 # 42
```

Before a native function runs, the runtime checks the call against the function's signature, by the number of arguments and by type, including [native types](#native-types-and-values) ([Calls](./functions-and-pipe.md#calls), [ADR 013](./adrs/013-type-checked-function-calls.md)). A handler only ever sees arguments of the types it declares, and a function with several signatures runs its handler with the arguments of whichever one matched ([Overloads](#overloads)). After the handler returns, the runtime checks its result against the signature's return type: a handler for `-> Number` that returns a string raises `Function <name> returned String, expected Number`. Declare `Any` for an argument or result the function checks itself.

`Number` accepts an `Integer`, and converts it to a `Number` ([ADR 017](./adrs/017-integers.md)). So a handler always gets a `Value::Number` for an argument declared `Number`, even when the call passed `1`. The same goes for a keyword declared `Number`, including its default when the call leaves it out. A handler declared `-> Number` may return a `Value::Integer`, which is converted the same way, so it prints as `1.0`. Declare `Integer` for an argument or result that must be a whole number.

### Reading Arguments

A handler gets the call's arguments as `&Arguments`, and reads each one by the name its signature gives it, converted to a Rust type ([ADR 014](./adrs/014-native-function-arguments.md)):

```rust
/// pad(s, width) pads `s` on the left to `width` characters
fn pad(args: &Arguments) -> Result<Value, Error> {
    let s: &str = args.get("s")?;
    let width: usize = args.get("width")?;
    Ok(Value::String(format!("{s:>width$}")))
}

// pad has type fn(String, Integer) -> String
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new(
        "pad",
        Type::string(),
        vec![Argument::string("s"), Argument::integer("width")],
    ),
    pad,
))?;
```

```flt
pad("ab", 4)          # "  ab"
```

- **`args.get::<T>(name)`** reads an argument or a [keyword argument](#keyword-arguments) as `T`. `T` can be:
  - `&Value`, the value as it is;
  - `String` or `&str`, for a `String`;
  - `bool`, for a `Boolean`;
  - `&Map`, for a `Map`, and `&[Value]`, for an `Array`'s elements;
  - `i32`, `i64`, `u32`, `u64` or `usize`, for an `Integer` or a whole `Number`, and `f64`, for either ([as `try_from` converts it](#defining-and-registering-a-native-type));
  - `BigInt`, for an `Integer` or a whole `Number`, and `BigDecimal`, for a `Number` or an `Integer`. flt re-exports both, as `flt::BigInt` and `flt::BigDecimal`;
  - `Option<T>` of any of these, for an argument declared `Option<…>`, which reads `None` as `None`.

  A host can implement the `FromValue` trait for types of its own.

- **`args.native::<T>(name)`** borrows a [native value](#native-types-and-values).
- **`args.value(name)`** gives the `&Value`, or `None` if the overload that matched has no argument or keyword of that name, or if it's the variadic argument, whose values are in `args.rest()` ([Overloads](#overloads)).
- **`args.rest()`** gives a [variadic argument](#variadic-arguments)'s values, and **`args.keywords()`** the keyword arguments as a map.
- **`args.positional()`** gives every positional value as a slice, **`args.function_name()`** the function's name, and **`args.signature()`** the overload that matched.

Names are those of the overload that matched the call: its positional arguments, then its keywords.

Since calls are type-checked, `get` and `native` only fail if the handler reads an argument as a type its signature doesn't promise, or a number as a Rust type that can't hold it:

- `Expected <type>, found <type>`: an argument declared `Any`, read with `get::<&str>`, given a `Number`;
- `Cannot convert <number> to <type>`: `2.5` read as a `usize`.

Reading a name the overload doesn't declare is a bug in the handler, and raises a [handler error](#handler-errors).

### Maps

A map arrives as `Value::Map(flt::runtime::Map)`, keyed by `MapKey`. A key written in flt source, in a map literal or as a call's [keyword argument](#keyword-arguments), is a symbol, `MapKey::Symbol`. A host can also use string keys, `MapKey::String`, which flt source can't write yet; a symbol and a string with the same text are different keys. In `SimpleRuntime`, maps keep insertion order, so a handler iterating a map literal sees its entries in source order. Hosts build maps with `flt::runtime::Map` too.

```rust
use flt::runtime::Map;
use flt::runtime::MapKey;

/// keys(map) returns the map's keys, in order: keys({b: 1, a: 2}) is [:b, :a]
fn keys(args: &Arguments) -> Result<Value, Error> {
    let map: &Map = args.get("map")?;
    Ok(Value::Array(map.keys().cloned().map(Value::from).collect()))
}

runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("keys", Type::array(), vec![Argument::new("map", Type::map())]),
    keys,
))?;

let mut point = Map::new();
point.insert(MapKey::symbol("x"), Value::from(1u64));
runtime.set_variable("point", Value::Map(point))?;   // point is {x: 1}
```

- `MapKey::symbol(name)` and `MapKey::string(s)` make keys.
- `Value::from(key)` turns a key into the flt value it stands for, a `Symbol` or a `String`.
- A string key prints as `"b" => 1`, which flt can't parse yet.

### Keyword Arguments

A native function can declare optional [keyword arguments](./functions-and-pipe.md#keyword-arguments), each with a type and a default, with `FunctionDefinition::keyword` ([ADR 010](./adrs/010-keyword-arguments.md)). Its handler reads them by name, like its positional arguments:

```rust
/// greet(name, greeting: "Hello") returns "<greeting>, <name>"
fn greet(args: &Arguments) -> Result<Value, Error> {
    let name: &str = args.get("name")?;
    let greeting: &str = args.get("greeting")?;
    Ok(Value::String(format!("{greeting}, {name}")))
}

// greet has type fn(String, greeting: String) -> String
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("greet", Type::string(), vec![Argument::string("name")])
        .keyword("greeting", Type::string(), Value::String("Hello".into())),
    greet,
))?;
```

```flt
greet("Ann")                   # "Hello, Ann"
greet("Ann", greeting: "Hi")   # "Hi, Ann"
greet("Ann", tone: "warm")     # error: Function greet got an unexpected keyword argument: tone
```

- **Every declared keyword is there**, with the call's value or else the default, so `args.get("greeting")` always finds it. `args.keywords()` has them all, keyed by symbols, in declaration order.
- **`keyword` adds to the most recently added overload**, so each overload declares its own keywords, and a call's keywords help pick the overload ([Overloads](#overloads)).
- **Keyword values are type-checked**, like positional arguments, so `greet("Ann", greeting: 1)` raises `Keyword argument greeting of greet expects String, found Number`, and `greet`'s handler only sees a string. Rest keywords are checked against the type given to `rest_keywords`.
- **A default must be of its keyword's type.** `keyword` panics if it isn't, since that's a bug in the definition.
- **A function without keywords never sees any.** A call that passes one raises `Function <name> got an unexpected keyword argument: <key>`.

A function that takes any keywords, such as one whose keywords name things the user picks, declares rest keywords with `FunctionDefinition::rest_keywords`. They have no declared names, so the handler reads them from `args.keywords()`, where they follow the declared keywords, in call order:

```rust
/// options() returns the keywords it was given, as a map
fn options(args: &Arguments) -> Result<Value, Error> {
    Ok(Value::Map(args.keywords().clone()))
}

// options has type fn(**Any) -> Map
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("options", Type::map(), vec![]).rest_keywords(Type::any()),
    options,
))?;
```

```flt
options(b: 1, a: 2)            # {b: 1, a: 2}
```

### Variadic Arguments

A native function's last positional argument can be variadic, made with `Argument::variadic`
([ADR 011](./adrs/011-variadic-arguments.md)). It takes zero or more values, which the handler
gets from `args.rest()`, as a slice:

```rust
/// count_args(*args) returns how many arguments it was given
fn count_args(args: &Arguments) -> Result<Value, Error> {
    Ok(Value::from(args.rest().len()))
}

// count_args has type fn(*Any) -> Integer
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new(
        "count_args",
        Type::integer(),
        vec![Argument::variadic("args", Type::any())],
    ),
    count_args,
))?;
```

```flt
count_args()                   # 0
count_args(1, 2, 3)            # 3
1 |> count_args(2)             # 2
```

- **The arguments before a variadic one are required.** `sprintf`, defined with `vec![Argument::string("format"), Argument::variadic("args", Type::any())]`, has type `fn(String, *Any) -> String`, so `sprintf("x")` passes and `sprintf()` raises `Function sprintf expected at least 1 argument(s), found 0`. Its handler reads `args.get::<&str>("format")`, and `args.rest()` holds only the values after it.
- **Each value is type-checked** against the variadic argument's type: with `total(*values: Number)`, `total(1, "a")` raises `Argument values of total expects Number, found String`.
- **Whether no values is an error is up to the handler.** A call that passes the checks reaches the handler even if the variadic argument got nothing.
- **`get` and `native` read one value**, so reading the variadic argument by name raises a [handler error](#handler-errors).
- **Only the last positional argument can be variadic.** `FunctionDefinition::new` and `add_overload` panic on a variadic argument anywhere else. Keywords are declared separately, so a function can have both: its type lists the variadic parameter before its keywords, e.g. `fn(Any, *Any, k: Number) -> Any`.
- **A variadic argument takes only positional values.** A function can declare keywords too, and reads them by name, as any function does.

### Overloads

`FunctionDefinition::add_overload` gives a function another signature. The overloads share one handler, which gets the arguments of whichever overload matched the call, under that overload's names:

```rust
/// describe(n) or describe(s) says whether it was given a number or text
fn describe(args: &Arguments) -> Result<Value, Error> {
    let kind = match args.value("n") {
        Some(_) => "number",
        None => "text",
    };
    Ok(Value::String(kind.into()))
}

// describe has type fn(Number) -> String | fn(String | Symbol) -> String
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("describe", Type::string(), vec![Argument::number("n")])
        .add_overload(
            Type::string(),
            vec![Argument::new("s", Type::union(Type::string(), Type::symbol()))],
        ),
    describe,
))?;
```

```flt
describe(1)                    # "number"
describe("a")                  # "text"
describe(:a)                   # "text"
describe(true)                 # error: No overload of describe accepts (Boolean)
```

- **The overload is picked by arity, longest first, then by type and keywords**, as described in [Overloads](./functions-and-pipe.md#overloads). The order the overloads are declared in doesn't matter, except that if several of the same arity match a call and none is [narrowest](./functions-and-pipe.md#the-narrowest-overload), `SimpleRuntime` runs the one declared first.
- **A call that doesn't match gets the closest overload's error**, if one is closest, as described in [Overloads](./functions-and-pipe.md#overloads). `describe(true)` fails at the first argument of both, so neither is closer.
- **Each overload has its own return type**, and the handler's result is checked against the one that matched.
- **Use a union when the forms do the same thing**, as in `column: Symbol | String`, and overloads when they do different things or return different types. `count(plan: Plan) -> Integer`, which runs a query, and `count(column: Symbol | String) -> Aggregate`, which builds an aggregate, are two overloads.
- **Give the same thing the same name in every overload**, and the handler can read it without knowing which overload matched. `args.value` tells overloads apart by their names, as `describe` does.

An argument can be positional in one overload and a keyword in another, under the same name. flt has no defaults for positional arguments, so this is how an argument gets a default:

```rust
/// repeat(s, n) or repeat(s, n: 2) repeats `s` `n` times
fn repeat(args: &Arguments) -> Result<Value, Error> {
    let s: &str = args.get("s")?;
    let n: usize = args.get("n")?;
    Ok(Value::String(s.repeat(n)))
}

// repeat has type fn(String, Integer) -> String | fn(String, n: Integer) -> String
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new(
        "repeat",
        Type::string(),
        vec![Argument::string("s"), Argument::integer("n")],
    )
    .add_overload(Type::string(), vec![Argument::string("s")])
    .keyword("n", Type::integer(), Value::from(2u64)),
    repeat,
))?;
```

```flt
repeat("ab", 3)                # "ababab"
repeat("ab")                   # "abab"
repeat("ab", n: 3)             # "ababab"
repeat("ab", 3, n: 3)          # error: Function repeat got an unexpected keyword argument: n
```

## Native Types and Values

Native functions can also pass the host's own Rust values through flt, as native values ([ADR 007](./adrs/007-native-types-and-values.md)). A native value is a value of a native type, such as a date or a lazy query plan, that only the host understands.

### Defining and Registering a Native Type

A Rust type becomes a native type by implementing `NativeType`, which gives its name in flt. It must also implement `Display`, for how its values print, and `PartialEq`, for how they compare. The host registers each native type with the runtime, before the functions that use it and before evaluating any flt source.

This example makes [chrono](https://docs.rs/chrono)'s `NaiveDate` the native type `Date`:

```rust
use std::fmt;

use chrono::NaiveDate;
use flt::runtime::functions::Argument;
use flt::runtime::functions::Arguments;
use flt::runtime::functions::FunctionDefinition;
use flt::runtime::functions::NativeFunction;
use flt::runtime::native::NativeType;
use flt::runtime::types::Type;
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;
use flt::runtime::Value;
use flt::Error;

/// chrono's `NaiveDate` as the native type `Date`. `NativeType` and `NaiveDate` both come
/// from other crates, so the host wraps `NaiveDate` in its own type.
#[derive(PartialEq)]
struct Date(NaiveDate);

/// A date prints as `yyyy-mm-dd`
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NativeType for Date {
    const NAME: &'static str = "Date";
}

/// date(yyyy, mm, dd) creates a date
fn date(args: &Arguments) -> Result<Value, Error> {
    let year: i32 = args.get("yyyy")?;
    let month: u32 = args.get("mm")?;
    let day: u32 = args.get("dd")?;
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(Value::native(Date(date))),
        // A host error, shown unchanged (see Host Errors)
        None => Err(Error::native(std::io::Error::other(format!(
            "{year:04}-{month:02}-{day:02} is not a valid date"
        )))),
    }
}

/// days_between(from, to) counts the days from one date to another
fn days_between(args: &Arguments) -> Result<Value, Error> {
    let from = args.native::<Date>("from")?;
    let to = args.native::<Date>("to")?;
    Ok(Value::from((to.0 - from.0).num_days()))
}

/// Registers `Date` and the functions that use it. It uses only the `Runtime` trait, so it
/// works with any runtime.
fn install(runtime: &mut impl Runtime) -> Result<(), Error> {
    runtime.register_native_type::<Date>()?;

    // date has type fn(Integer, Integer, Integer) -> Date
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "date",
            Type::native::<Date>(),
            vec![Argument::integer("yyyy"), Argument::integer("mm"), Argument::integer("dd")],
        ),
        date,
    ))?;

    // days_between has type fn(Date, Date) -> Integer
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "days_between",
            Type::integer(),
            vec![
                Argument::new("from", Type::native::<Date>()),
                Argument::new("to", Type::native::<Date>()),
            ],
        ),
        days_between,
    ))?;
    Ok(())
}

let mut runtime = SimpleRuntime::default();
install(&mut runtime)?;
```

- `Value::native(Date(date))` makes a native value.
- `args.native::<Date>(name)` reads one back. Given any other value, it raises `Expected Date, found <type>`. `value.downcast_native::<Date>()` does the same for a `&Value`.
- `value.get_native::<Date>()` reads one back as an `Option`, for a handler that accepts more than one kind of value, e.g. one declared with a union type.
- `Type::native::<Date>()` names the native type in a signature.
- `runtime.register_native_type::<Date>()` registers the type. Through a `&mut dyn Runtime`, call `runtime.register_native_type_info(NativeTypeInfo::of::<Date>())` instead, since the generic method can't be called there.
- `Value::from` turns a Rust `u64`, `i64`, `usize` or a `BigInt` into an `Integer`, and a `BigDecimal` into a `Number`.
- `args.get::<i32>(name)` reads an `Integer`, or a whole `Number`, as a Rust integer, as `i32::try_from(&value)` does for a `&Value`. Both work for `i32`, `i64`, `u32`, `u64`, `usize` and `f64`. A value that isn't a number raises `Expected Number, found <type>`, and a number that doesn't fit the type raises `Cannot convert <number> to <type>`, e.g. `Cannot convert -1 to u32` or `Cannot convert 2.5 to u32`. Conversion to `f64` accepts either kind of number, and may lose precision.

In `SimpleRuntime`, `register_native_type` fails with `Cannot register native type <name>: …` if the type's name is a built-in type name, a function's name, a variable's name or another native type's name. A value of a native type that was never registered still works, but its type's name doesn't resolve in flt source.

### Native Values in flt

flt source can bind, pass, pipe, compare and print native values, and get their type, but only the host's functions can create them or look inside them:

```flt
let new_year = date(2026, 1, 1)
new_year                                   # 2026-01-01
typeof(new_year)                           # Date
typeof(new_year) == Date                   # true
typeof(Date)                               # Type
typeof(days_between)                       # fn(Date, Date) -> Integer
new_year |> days_between(date(2026, 10, 25))   # 297
new_year == date(2026, 1, 1)               # true: the host's PartialEq compares the dates
days_between(1, new_year)                  # error: Argument from of days_between expects Date, found Integer
date(2026, -1, 1)                          # error: Cannot convert -1 to u32
Date(2026, 1, 1)                           # error: Date is not a function
new_year.year                              # error: Invalid Operand Type
let Date = 1                               # error: Cannot bind Date: it's a native type
```

- **Native values are opaque.** flt source can't create them, look inside them or change them: `Date(2026, 1, 1)` raises `Date is not a function`, and `new_year.year` raises `Invalid Operand Type`.
- **They print as the host displays them**, with nothing added: a `Date` prints as `2026-01-01`.
- **They compare as the host defines.** `==` and `!=` use the host's `PartialEq`, so two `Date`s for the same day are equal. Values of different native types are never equal, and a native value never equals any other kind of value. Native values can't be ordered: `<`, `>`, `<=` and `>=` raise `Invalid Operand Type`.
- **A native type's name evaluates to that type**, like a [built-in type's name](./types.md#types-as-values). It can't be bound: `let Date = 1` raises `Cannot bind Date: it's a native type`, and `/unset Date` in the REPL reports the same reason.
- **Function types can name native types**, as in `fn(Date, Date) -> Integer`, and the runtime checks them like any type: `days_between(1, new_year)` raises `Argument from of days_between expects Date, found Integer`. Two native types with the same name are still different types.
- **`<>` treats a native value as the empty string**, as it does for function values.

## Host Errors

A handler can fail with the host's own error, wrapped with `Error::native`, as `date` does above for a day that doesn't exist. The runtime reports it exactly as the host's error displays it, without the `Runtime Error:` prefix that flt's own errors have, and the host can get the original error back from `Error::Native`:

```flt
date(2026, 2, 30)     # error: 2026-02-30 is not a valid date
```

## Handler Errors

A handler that uses flt's API wrongly gets an `Error::HandlerError`, a `HandlerError` that names the mistake. It's a bug in the host, not in the flt source, so it isn't a `RuntimeError`, and it prints with a `Handler Error:` prefix. Like any error, it fails the statement, and the host keeps running.

- `Function <name> has no argument <argument>` (`HandlerError::UndeclaredArgument`): `get` or `native` read a name that the overload that matched doesn't declare, e.g. a misspelt one;
- `Argument <argument> of <name> is variadic; read it with Arguments::rest` (`HandlerError::VariadicArgument`): `get` or `native` read a [variadic argument](#variadic-arguments), which has any number of values.

For a function whose overloads use different names, `args.value(name)` asks whether there's an argument of that name, without an error ([Overloads](#overloads)).

```rust
/// greet(name) reads a name its signature doesn't declare
fn greet(args: &Arguments) -> Result<Value, Error> {
    let name: &str = args.get("nmae")?;
    Ok(Value::String(format!("Hello, {name}")))
}
```

```flt
greet("Ann")          # error: Handler Error: Function greet has no argument nmae
```

`Error::HandlerError` and `Error::Native` are different things: `Error::Native` carries an error the host raised on its own terms, such as an I/O error, and `Error::HandlerError` is flt reporting that a handler called it wrongly.

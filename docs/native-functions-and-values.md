# Native Functions, Types and Values

A host application embedding `flt` extends it from Rust. It can register:

- [native functions](#native-functions), which flt source calls like any function;
- [native types](#native-types-and-values), for the host's own values, which its native functions create and read and flt source passes around.

A native function can also fail with the host's own [error](#host-errors).

## Native Functions

A host can register native functions, written in Rust, with `runtime.register_function`, a method of the [`Runtime` trait](./runtime-and-limitations.md#the-runtime-trait). The runtime keeps them in its global scope, alongside the built-ins. A function's name then evaluates to a [function value](./types.md#function-types), which can be bound to another name and called through it.

```rust
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use flt::runtime::functions::Argument;
use flt::runtime::functions::FunctionDefinition;
use flt::runtime::functions::NativeFunction;
use flt::runtime::types::Type;
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;
use flt::runtime::Value;
use flt::Error;

/// identity(v) returns its argument
fn identity(args: &[Value]) -> Result<Value, Error> {
    Ok(args[0].clone())
}

/// Makes a handler that counts its calls. A handler can be any `Fn`, so one built by a
/// function like this may capture state. Handlers are Send + Sync, so mutable state needs
/// Arc<Mutex<_>> or atomics rather than Rc<RefCell<_>>.
fn counter(calls: Arc<AtomicU64>) -> impl Fn(&[Value]) -> Result<Value, Error> + Send + Sync {
    move |_args| Ok(Value::from(calls.fetch_add(1, Ordering::SeqCst) + 1))
}

let mut runtime = SimpleRuntime::default();

// identity has type fn(Any) -> Any
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("identity", Type::any(), vec![Argument::new("v", Type::any())]),
    identity,
))?;

// count() returns how many times it has been called, and has type fn() -> Number
let calls = Arc::new(AtomicU64::new(0));
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("count", Type::number(), vec![]),
    counter(Arc::clone(&calls)),
))?;
```

Registering a function under a name that's already registered replaces it. That includes the [built-ins](./functions-and-pipe.md#built-in-functions): a host that registers its own `typeof` replaces the default one.

A function's name can't be bound as a variable, whether the function is a built-in or the host's. Registration returns a `Result`. In `SimpleRuntime`, `register_function` fails with `Cannot register function <name>: …` if the name is a built-in type name, such as `Number`, a native type's name, or is already bound as a variable, so register functions before evaluating any flt source. Binding a variable from Rust with `runtime.set_variable` refuses the same names as `let`, and returns an error. `runtime.check_bindable` reports whether a name can be bound without binding it. Other runtimes may have other rules ([Rules Belong to the Runtime](./runtime-and-limitations.md#rules-belong-to-the-runtime)).

```flt
identity              # <fn identity>
typeof(identity)      # fn(Any) -> Any
identity(42)          # 42
let f = identity
f(42)                 # 42
```

Before a native function runs, the runtime checks the call against the function's signature, by the number of arguments and by type, including [native types](#native-types-and-values) ([Calls](./functions-and-pipe.md#calls), [ADR 013](./adrs/013-type-checked-function-calls.md)). A handler only ever sees arguments of the types it declares, and a function with several signatures runs its handler with the arguments of whichever one matched ([Overloads](#overloads)). After the handler returns, the runtime checks its result against the signature's return type: a handler for `-> Number` that returns a string raises `Function <name> returned String, expected Number`. Declare `Any` for an argument or result the function checks itself.

A map arrives as `Value::Map(flt::runtime::Map)`, keyed by `MapKey`. A key written in flt source, in a map literal or as a call's [keyword argument](#keyword-arguments), is a symbol, `MapKey::Symbol`. A host can also use string keys, `MapKey::String`, which flt source can't write yet; a symbol and a string with the same text are different keys. In `SimpleRuntime`, maps keep insertion order, so a handler iterating a map literal sees its entries in source order. Hosts build maps with `flt::runtime::Map` too.

```rust
use flt::runtime::Map;
use flt::runtime::MapKey;

/// keys(map) returns the map's keys, in order: keys({b: 1, a: 2}) is [:b, :a]
fn keys(args: &[Value]) -> Result<Value, Error> {
    match &args[0] {
        Value::Map(map) => Ok(Value::Array(map.keys().cloned().map(Value::from).collect())),
        _ => Ok(Value::Array(vec![])),
    }
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

A native function can declare optional [keyword arguments](./functions-and-pipe.md#keyword-arguments), each with a type and a default, with `FunctionDefinition::keyword` ([ADR 010](./adrs/010-keyword-arguments.md)). Its handler, registered with `NativeFunction::with_keywords`, gets the call's keywords as a `Map`, separately from its positional arguments:

```rust
use flt::runtime::Map;
use flt::runtime::MapKey;

/// greet(name, greeting: "Hello") returns "<greeting>, <name>"
fn greet(args: &[Value], keywords: &Map) -> Result<Value, Error> {
    match (&args[0], &keywords[&MapKey::symbol("greeting")]) {
        (Value::String(name), Value::String(greeting)) => {
            Ok(Value::String(format!("{greeting}, {name}")))
        }
        _ => Ok(Value::None),
    }
}

// greet has type fn(String, greeting: String) -> String
runtime.register_function(NativeFunction::with_keywords(
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

- **The map holds every declared keyword**, with the call's value or else the default, so `keywords[&MapKey::symbol("greeting")]` is always there. Its keys are symbols, in declaration order.
- **`keyword` adds to the most recently added overload**, so each overload declares its own keywords, and a call's keywords help pick the overload ([Overloads](#overloads)).
- **Keyword values are type-checked**, like positional arguments, so `greet("Ann", greeting: 1)` raises `Keyword argument greeting of greet expects String, found Number`, and `greet`'s handler only sees a string. Rest keywords are checked against the type given to `rest_keywords`.
- **A default must be of its keyword's type.** `keyword` panics if it isn't, since that's a bug in the definition.
- **`NativeFunction::new` is for functions without keywords.** It panics if the definition declares any, since its handler couldn't see them. Its handler is never called with keywords: a call that passes any raises `Function <name> got an unexpected keyword argument: <key>`.

A function that takes any keywords, such as one whose keywords name things the user picks, declares rest keywords with `FunctionDefinition::rest_keywords`. They follow the declared keywords in the map, in call order:

```rust
/// options() returns the keywords it was given, as a map
fn options(_args: &[Value], keywords: &Map) -> Result<Value, Error> {
    Ok(Value::Map(keywords.clone()))
}

// options has type fn(**Any) -> Map
runtime.register_function(NativeFunction::with_keywords(
    FunctionDefinition::new("options", Type::map(), vec![]).rest_keywords(Type::any()),
    options,
))?;
```

```flt
options(b: 1, a: 2)            # {b: 1, a: 2}
```

### Variadic Arguments

A native function's last positional argument can be variadic, made with `Argument::variadic`
([ADR 011](./adrs/011-variadic-arguments.md)). It takes zero or more values, and the handler gets
them after the arguments before it, all in one flat `&[Value]`:

```rust
/// count_args(*args) returns how many arguments it was given
fn count_args(args: &[Value]) -> Result<Value, Error> {
    Ok(Value::from(args.len() as u64))
}

// count_args has type fn(*Any) -> Number
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new(
        "count_args",
        Type::number(),
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

- **The arguments before a variadic one are required.** `sprintf`, defined with `vec![Argument::string("format"), Argument::variadic("args", Type::any())]`, has type `fn(String, *Any) -> String`, so `sprintf("x")` passes and `sprintf()` raises `Function sprintf expected at least 1 argument(s), found 0`.
- **Each value is type-checked** against the variadic argument's type: with `total(*values: Number)`, `total(1, "a")` raises `Argument values of total expects Number, found String`.
- **Whether no values is an error is up to the handler.** A call that passes the checks reaches the handler even if the variadic argument got nothing.
- **Only the last positional argument can be variadic.** `FunctionDefinition::new` and `add_overload` panic on a variadic argument anywhere else. Keywords are declared separately, so a function can have both: its type lists the variadic parameter before its keywords, e.g. `fn(Any, *Any, k: Number) -> Any`.
- **A variadic argument takes only positional values.** A function can declare keywords too, with `NativeFunction::with_keywords`, and gets them in its `Map`, as for any function.

### Overloads

`FunctionDefinition::add_overload` gives a function another signature. The overloads share one handler, which gets the arguments of whichever overload matched the call:

```rust
/// describe(value) says whether it was given a number or text
fn describe(args: &[Value]) -> Result<Value, Error> {
    let kind = match &args[0] {
        Value::Number(_) => "number",
        _ => "text",
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

- **The overload is picked by arity, longest first, then by type and keywords**, as described in [Overloads](./functions-and-pipe.md#overloads). The order the overloads are declared in doesn't matter, except that if several of the same arity match a call, `SimpleRuntime` runs the one declared first.
- **Each overload has its own return type**, and the handler's result is checked against the one that matched.
- **Use a union when the forms do the same thing**, as in `column: Symbol | String`, and overloads when they do different things or return different types. `count(plan: Plan) -> Number`, which runs a query, and `count(column: Symbol | String) -> Aggregate`, which builds an aggregate, are two overloads.

## Native Types and Values

Native functions can also pass the host's own Rust values through flt, as native values ([ADR 007](./adrs/007-native-types-and-values.md)). A native value is a value of a native type, such as a date or a lazy query plan, that only the host understands.

### Defining and Registering a Native Type

A Rust type becomes a native type by implementing `NativeType`, which gives its name in flt. It must also implement `Display`, for how its values print, and `PartialEq`, for how they compare. The host registers each native type with the runtime, before the functions that use it and before evaluating any flt source.

This example makes [chrono](https://docs.rs/chrono)'s `NaiveDate` the native type `Date`:

```rust
use std::fmt;

use chrono::NaiveDate;
use flt::runtime::functions::Argument;
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
fn date(args: &[Value]) -> Result<Value, Error> {
    let year = i32::try_from(&args[0])?;
    let month = u32::try_from(&args[1])?;
    let day = u32::try_from(&args[2])?;
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(Value::native(Date(date))),
        // A host error, shown unchanged (see Host Errors)
        None => Err(Error::native(std::io::Error::other(format!(
            "{year:04}-{month:02}-{day:02} is not a valid date"
        )))),
    }
}

/// days_between(from, to) counts the days from one date to another
fn days_between(args: &[Value]) -> Result<Value, Error> {
    let from = args[0].downcast_native::<Date>()?;
    let to = args[1].downcast_native::<Date>()?;
    Ok(Value::from((to.0 - from.0).num_days()))
}

/// Registers `Date` and the functions that use it. It uses only the `Runtime` trait, so it
/// works with any runtime.
fn install(runtime: &mut impl Runtime) -> Result<(), Error> {
    runtime.register_native_type::<Date>()?;

    // date has type fn(Number, Number, Number) -> Date
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "date",
            Type::native::<Date>(),
            vec![Argument::number("yyyy"), Argument::number("mm"), Argument::number("dd")],
        ),
        date,
    ))?;

    // days_between has type fn(Date, Date) -> Number
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "days_between",
            Type::number(),
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
- `value.downcast_native::<Date>()` reads one back. Given any other value, it raises `Expected Date, found <type>`.
- `value.get_native::<Date>()` reads one back as an `Option`, for a handler that accepts more than one kind of value, e.g. one declared with a union type.
- `Type::native::<Date>()` names the native type in a signature.
- `runtime.register_native_type::<Date>()` registers the type. Through a `&mut dyn Runtime`, call `runtime.register_native_type_info(NativeTypeInfo::of::<Date>())` instead, since the generic method can't be called there.
- `Value::from` turns a Rust `u64`, `i64` or `usize` into a `Number`.
- `i32::try_from(&value)` reads a `Number` back as a Rust number. It's implemented for `i32`, `i64`, `u32`, `u64`, `usize` and `f64`. A value that isn't a number raises `Expected Number, found <type>`, and a number that doesn't fit the type raises `Cannot convert <number> to <type>`, e.g. `Cannot convert 2.5 to u32`. Conversion to `f64` accepts any number, and may lose precision.

In `SimpleRuntime`, `register_native_type` fails with `Cannot register native type <name>: …` if the type's name is a built-in type name, a function's name, a variable's name or another native type's name. A value of a native type that was never registered still works, but its type's name doesn't resolve in flt source.

### Native Values in flt

flt source can bind, pass, pipe, compare and print native values, and get their type, but only the host's functions can create them or look inside them:

```flt
let new_year = date(2026, 1, 1)
new_year                                   # 2026-01-01
typeof(new_year)                           # Date
typeof(new_year) == Date                   # true
typeof(Date)                               # Type
typeof(days_between)                       # fn(Date, Date) -> Number
new_year |> days_between(date(2026, 10, 25))   # 297
new_year == date(2026, 1, 1)               # true: the host's PartialEq compares the dates
days_between(1, new_year)                  # error: Argument from of days_between expects Date, found Number
date(2026, -1, 1)                          # error: Cannot convert -1 to u32
Date(2026, 1, 1)                           # error: Date is not a function
new_year.year                              # error: Invalid Operand Type
let Date = 1                               # error: Cannot bind Date: it's a native type
```

- **Native values are opaque.** flt source can't create them, look inside them or change them: `Date(2026, 1, 1)` raises `Date is not a function`, and `new_year.year` raises `Invalid Operand Type`.
- **They print as the host displays them**, with nothing added: a `Date` prints as `2026-01-01`.
- **They compare as the host defines.** `==` and `!=` use the host's `PartialEq`, so two `Date`s for the same day are equal. Values of different native types are never equal, and a native value never equals any other kind of value. Native values can't be ordered: `<`, `>`, `<=` and `>=` raise `Invalid Operand Type`.
- **A native type's name evaluates to that type**, like a [built-in type's name](./types.md#types-as-values). It can't be bound: `let Date = 1` raises `Cannot bind Date: it's a native type`, and `/unset Date` in the REPL reports the same reason.
- **Function types can name native types**, as in `fn(Date, Date) -> Number`, and the runtime checks them like any type: `days_between(1, new_year)` raises `Argument from of days_between expects Date, found Number`. Two native types with the same name are still different types.
- **`<>` treats a native value as the empty string**, as it does for function values.

## Host Errors

A handler can fail with the host's own error, wrapped with `Error::native`, as `date` does above for a day that doesn't exist. The runtime reports it exactly as the host's error displays it, without the `Runtime Error:` prefix that flt's own errors have, and the host can get the original error back from `Error::Native`:

```flt
date(2026, 2, 30)     # error: 2026-02-30 is not a valid date
```

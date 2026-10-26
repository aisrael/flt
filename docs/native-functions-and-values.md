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

let mut runtime = SimpleRuntime::default();

// identity(v) returns its argument, and has type fn(Any) -> Any
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("identity", Type::any(), vec![Argument::new("v", Type::any())]),
    |args| Ok(args[0].clone()),
))?;

// A handler is a closure, so it may capture state. Handlers are Send + Sync, so
// mutable state needs Arc<Mutex<_>> or atomics rather than Rc<RefCell<_>>.
let calls = Arc::new(AtomicU64::new(0));
let counter = Arc::clone(&calls);
runtime.register_function(NativeFunction::new(
    FunctionDefinition::new("count", Type::number(), vec![]),
    move |_args| Ok(Value::Number((counter.fetch_add(1, Ordering::SeqCst) + 1).into())),
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

Before a native function runs, the runtime checks the number of arguments against the function's signature ([Calls](./functions-and-pipe.md#calls)). Argument types aren't checked yet, not even [native types](#native-types-and-values); that is left to the function itself.

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

/// Registers `Date` and the functions that use it. It uses only the `Runtime` trait, so it
/// works with any runtime.
fn install(runtime: &mut impl Runtime) -> Result<(), Error> {
    runtime.register_native_type::<Date>()?;

    // date(yyyy, mm, dd) creates a date, and has type fn(Number, Number, Number) -> Date
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "date",
            Type::native::<Date>(),
            vec![Argument::number("yyyy"), Argument::number("mm"), Argument::number("dd")],
        ),
        |args| {
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
        },
    ))?;

    // days_between(from, to) counts the days from one date to another, and has type
    // fn(Date, Date) -> Number
    runtime.register_function(NativeFunction::new(
        FunctionDefinition::new(
            "days_between",
            Type::number(),
            vec![
                Argument::new("from", Type::native::<Date>()),
                Argument::new("to", Type::native::<Date>()),
            ],
        ),
        |args| {
            let from = args[0].downcast_native::<Date>()?;
            let to = args[1].downcast_native::<Date>()?;
            Ok(Value::from((to.0 - from.0).num_days()))
        },
    ))?;
    Ok(())
}

let mut runtime = SimpleRuntime::default();
install(&mut runtime)?;
```

- `Value::native(Date(date))` makes a native value.
- `value.downcast_native::<Date>()` reads one back. Given any other value, it raises `Expected Date, found <type>`.
- `value.get_native::<Date>()` reads one back as an `Option`, for a handler that accepts more than one kind of value.
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
days_between(1, new_year)                  # error: Expected Date, found Number
date(2026, -1, 1)                          # error: Cannot convert -1 to u32
Date(2026, 1, 1)                           # error: Date is not a function
new_year.year                              # error: Invalid Operand Type
let Date = 1                               # error: Cannot bind Date: it's a native type
```

- **Native values are opaque.** flt source can't create them, look inside them or change them: `Date(2026, 1, 1)` raises `Date is not a function`, and `new_year.year` raises `Invalid Operand Type`.
- **They print as the host displays them**, with nothing added: a `Date` prints as `2026-01-01`.
- **They compare as the host defines.** `==` and `!=` use the host's `PartialEq`, so two `Date`s for the same day are equal. Values of different native types are never equal, and a native value never equals any other kind of value. Native values can't be ordered: `<`, `>`, `<=` and `>=` raise `Invalid Operand Type`.
- **A native type's name evaluates to that type**, like a [built-in type's name](./types.md#types-as-values). It can't be bound: `let Date = 1` raises `Cannot bind Date: it's a native type`, and `/unset Date` in the REPL reports the same reason.
- **Function types can name native types**, as in `fn(Date, Date) -> Number`. The runtime still checks only the number of arguments, so a function that takes a `Date` checks the value itself: `days_between(1, new_year)` raises `Expected Date, found Number`.
- **`<>` treats a native value as the empty string**, as it does for function values.

## Host Errors

A handler can fail with the host's own error, wrapped with `Error::native`, as `date` does above for a day that doesn't exist. The runtime reports it exactly as the host's error displays it, without the `Runtime Error:` prefix that flt's own errors have, and the host can get the original error back from `Error::Native`:

```flt
date(2026, 2, 30)     # error: 2026-02-30 is not a valid date
```

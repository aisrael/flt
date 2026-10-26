# ADR 008: The `Runtime` Trait

- **Date:** 2026-10-25
- **Status:** Accepted.

## Context

A host embeds flt by working with a runtime. Until now, that API was split three ways:

- the `Runtime` trait had only `eval`, which took a parsed statement;
- registering functions and native types were inherent methods of `SimpleRuntime`
  ([ADR 007](007-native-types-and-values.md), decision 2);
- binding and reading variables went through `SimpleRuntime`'s public `global_scope`.

Host code was tied to `SimpleRuntime` and to how it stores bindings, and every host wrote its own
loop to parse and evaluate source.

## Decisions

### 1. `Runtime` is flt's runtime API

Every runtime implements the `Runtime` trait. Hosts work with a runtime only through it, and name a
concrete runtime only to create one:

```rust
use flt::runtime::Runtime;
use flt::runtime::SimpleRuntime;

fn install(runtime: &mut impl Runtime) {
    runtime.register_native_type::<Date>();
    runtime.register_function(date);
}

let mut runtime = SimpleRuntime::default();
install(&mut runtime);
runtime.eval("days_between(date(2026, 1, 1), date(2026, 10, 25))")?;   // 297
```

### 2. The trait holds everything a host does

| Method                            | Does                                                                 |
| --------------------------------- | -------------------------------------------------------------------- |
| `eval(source)`                    | parses and evaluates flt source, returning the last statement's value |
| `eval_statement(statement)`       | evaluates one parsed statement                                       |
| `register_function(function)`     | registers a native function, replacing one of the same name          |
| `register_native_type::<T>()`     | registers the native type `T`                                        |
| `register_native_type_info(info)` | the same, given its `NativeTypeInfo`                                 |
| `get_native_type(name)`           | a registered native type                                             |
| `set_variable(name, value)`       | binds a global variable, or fails                                    |
| `get_variable(name)`              | a global variable's value                                            |
| `remove_variable(name)`           | unbinds a global variable, returning its value                       |
| `check_bindable(name)`            | whether `name` can be bound, and if not, why                         |
| `get_function(name)`              | a global function                                                    |

### 3. `eval` takes source; `eval_statement` takes a parsed statement

Hosts usually have flt source, so `eval(&str)` has the short name. The trait provides it:

- it parses all of the source first, so a syntax error raises `Parser Error: …` before any
  statement runs;
- it then evaluates the statements in order, with `eval_statement`, and returns the last one's
  value;
- empty source, or source with only comments, evaluates to `None`.

Statements are separated as in flt source: by newlines, or by `;`.

`eval_statement(&Statement)` is what each runtime implements. It's for hosts that parse source
themselves, like the REPL.

### 4. The trait works as `dyn Runtime`

A host can hold a `&mut dyn Runtime` or a `Box<dyn Runtime>`. `register_native_type::<T>()` is
generic, so the trait provides it on top of `register_native_type_info`, which is the one to call
through `dyn Runtime`:

```rust
runtime.register_native_type_info(NativeTypeInfo::of::<Date>());
```

### 5. Getters return owned values

`get_variable` returns an `Option<Value>`, and `get_function` an `Option<Arc<NativeFunction>>`, so a
runtime needn't store values in a form it can lend out.

### 6. The trait defines operations; each runtime defines its rules

The trait doesn't say which names a runtime refuses:

- `set_variable` and `check_bindable` return an error for a name the runtime won't bind;
- the `register_*` methods may panic for a name the runtime won't register. As in
  [ADR 006](006-no-shadowing.md) (decision 3), that's a host bug.

`SimpleRuntime` keeps the rules of [ADR 006](006-no-shadowing.md) and of
[ADR 007](007-native-types-and-values.md) (decision 7). This refines ADR 007's decision 11:
registration is part of the trait every runtime implements, while protected names stay each
runtime's own, as ADR 006 (decision 5) says.

### 7. `SimpleRuntime`'s global scope is private

`SimpleRuntime`'s `Runtime` methods go through its global scope, so ADR 006's decision 3 still
holds within `SimpleRuntime`: the global scope enforces the rule.

### 8. The REPL runs on any runtime

`FltRepl` is generic over its runtime and uses only the trait. `FltRepl::new()` runs on a
`SimpleRuntime`, and `FltRepl::with_runtime(runtime)` runs on one the host has set up.

## Alternatives Considered

- **Inherent methods on `SimpleRuntime`**, as after ADR 007. Rejected: host code would be tied to
  one runtime.
- **Keeping `eval(&Statement)`** and adding `eval_source(&str)`. Rejected: source is what hosts
  usually have, so it gets the short name.
- **Evaluating each statement as soon as it's parsed.** Rejected: a syntax error later in the
  source would leave the earlier statements' bindings behind.
- **A separate host trait** beside `Runtime`. Rejected: every runtime would implement both, for one
  object.
- **Keeping `global_scope` public** alongside the trait. Rejected: a second way to bind names that
  bypasses the runtime.
- **Protected-name rules in the trait.** Rejected: they're a runtime's decision (ADR 006,
  decision 5).
- **Getters that return references.** Rejected: they'd tie every runtime to storing values a
  certain way.
- **Registration that returns a `Result`.** Rejected: registration mistakes are host bugs, made
  before any flt source runs.

## Consequences

- Breaking changes:
  - `Runtime::eval` takes flt source; what was `eval(&Statement)` is `eval_statement`;
  - hosts `use flt::runtime::Runtime` to call the host methods;
  - `SimpleRuntime.global_scope` and `GlobalScope` are no longer public;
  - any other `Runtime` implementation must implement the new methods;
  - `FltRepl` gains a type parameter, defaulting to `SimpleRuntime`.
- New Rust API: `Runtime::eval_statement`, `register_native_type_info`, `set_variable`,
  `get_variable`, `remove_variable`, `check_bindable` and `get_function`, and
  `FltRepl::with_runtime`.- The spec doesn't change: the trait is the reference implementation's Rust API.

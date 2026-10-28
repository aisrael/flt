# ADR 015: Errors Name the Closest Overload

- **Date:** 2026-10-28
- **Status:** Accepted

## Context

When a call doesn't match a function, the error depends on how many of the function's overloads
take that many positional arguments ([ADR 013](013-type-checked-function-calls.md), decision 7):

- none: `Function <name> expected <counts> argument(s), found <found>`;
- one: the first thing that fails against it, e.g. `Argument n of head expects Number, found String`;
- several: `No overload of <name> accepts (<type>, …)`, which doesn't say what's wrong.

The last case is common in hosts whose overloads share a count. datu's `head` has two overloads
of two arguments, `head(plan: DatuPlan, n: Number)` and `head(path: String, n: Number)`, so

```flt
read("data.parquet") |> head("x")    # No overload of head accepts (DatuPlan, String)
```

The plan is right and `n` is wrong, but the error doesn't say so. It would, if `head` had only the
first overload.

## Decisions

### 1. The closest overload's error

A call is checked against an overload in a fixed order: its positional arguments in order, then its
keywords in call order ([ADR 013](013-type-checked-function-calls.md), decision 7). Among the
overloads that take that many positional arguments, the closest is the one that passes the most
checks before one fails. If one overload is closest, the call raises the error for the first thing
that fails against it, as it would if that overload were the only one:

```flt
read("data.parquet") |> head("x")    # Argument n of head expects Number, found String
```

`read("data.parquet") |> head("x")` is `head(read("data.parquet"), "x")`:

| Overload                          | Check 1: the plan  | Check 2: `"x"`     | Passes |
| --------------------------------- | ------------------ | ------------------ | ------ |
| `head(plan: DatuPlan, n: Number)` | is a `DatuPlan` ✓  | isn't a `Number` ✗ | 1      |
| `head(path: String, n: Number)`   | isn't a `String` ✗ | not checked        | 0      |

`head(plan, n)` passes more checks, so its error is the one raised.

### 2. A keyword of the wrong type is closer than one that isn't accepted

When overloads first fail at the same keyword, one that accepts that keyword, but not its value,
is closer than one that doesn't accept it at all. With
`foo(s: String, n: Number = 0, b: Boolean = false)` and
`foo(s: String, t: String = "", b: Boolean = false)`:

```flt
foo("abc", n: "x")    # Keyword argument n of foo expects Number, found String
foo("abc", t: 1)      # Keyword argument t of foo expects String, found Number
```

An overload accepts a keyword if it declares it, or if its rest keywords accept any others.

### 3. A tie is still `NoMatchingOverload`

If several overloads are equally close, none is named, and the error is
`No overload of <name> accepts (<type>, …)`, as before:

```flt
describe(true)        # No overload of describe accepts (Boolean)
head(1)               # No overload of head accepts (Number)
foo("abc", z: 1)      # No overload of foo accepts (String, z: Number)
```

Both of `describe`'s overloads fail at the first argument, as do `head(plan)` and `head(path)`,
and neither `foo` accepts `z`.

### 4. Declaration order doesn't matter

The closest overload, and whether there's a tie, depend only on the call and the overloads, not
on the order the overloads were declared in.

## Alternatives Considered

- **Keeping `NoMatchingOverload` for every call that several overloads take by count**
  ([ADR 013](013-type-checked-function-calls.md)). Rejected: it hides the specific error whenever
  overloads share a count, which is when the caller most needs to know which one they meant.
- **The overload that accepts the most arguments, wherever they are.** Rejected: with
  `f(a: Number, b: String)` and `f(a: String, b: Number)`, `f(1, 1)` would be a tie, though
  `f(a: Number, b: String)` took its first argument. Counting from the left fits pipes, where the
  first argument is the value piped in, and the overloads a host declares usually differ first by
  what that value is.
- **Listing every overload's error.** Rejected for now: it's long for a function with many
  overloads, and most of the errors are about overloads the caller didn't mean. `typeof` shows the
  overloads.
- **Breaking a tie by declaration order.** Rejected: the error would name whichever overload the
  host declared first, which says nothing about which one the caller meant.
- **Treating every keyword failure the same.** Rejected: `foo("abc", n: "x")` would be a tie, though
  only one `foo` declares `n`.

## Consequences

- Calls that several overloads take by count, and that one overload gets further through than the
  others, raise `ArgumentTypeMismatch`, `UnexpectedKeywordArgument` or `KeywordTypeMismatch`
  instead of `NoMatchingOverload`. Hosts that compare error messages may need to update them.
- `NoMatchingOverload` means that no overload is closest.
- The Rust API doesn't change.

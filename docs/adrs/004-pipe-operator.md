# ADR 004: The Pipe Operator

- **Date:** 2026-10-23
- **Status:** Accepted and implemented.
- **PR:** TBD

## Context

flt parsed `|>` but didn't evaluate it: evaluating a pipe raised `Pipe operator not yet supported`.
Pipes are the natural way to chain data operations in host applications such as datu, e.g.
`read("x.parquet") |> select(:id) |> count()`.

[ADR 003](003-functions-as-values.md) made functions values in the global scope, with one way to
resolve names and an arity check on every call. Pipes build on that call path.

The parser put `|>` at the lowest precedence, and comparisons shared a level with `*` and `/`, so
`x |> f == 3` parsed as `x |> (f == 3)` and `1 + 2 == 3` as `1 + (2 == 3)`.

## Decisions

### 1. The left side becomes the first argument

The pipe follows [Elixir](https://elixirschool.com/en/lessons/basics/pipe_operator): the result of
the left side becomes the **first** argument of the function on the right.

```flt
1 |> typeof              # typeof(1)         → Number
1 |> typeof()            # typeof(1)         → Number
typeof(1) |> typeof      # typeof(typeof(1)) → Type
10 |> sub(3)             # sub(10, 3)        → 7
x |> f(a, k: v)          # f(x, a, {k: v})
a |> f |> g(1)           # g(f(a), 1)
```

- The left side is evaluated first, then the right side's arguments, left to right.
- The right side is never evaluated on its own, so `1 |> typeof()` works although `typeof()` alone
  raises an arity error.
- Keyword arguments are already collected into a trailing `Map`, so prepending the piped value keeps
  them last.
- A pipe calls the function exactly as the equivalent call would: the same name resolution
  (ADR 003, decision 3), arity check (decision 4) and errors (decision 5).
- The right side must be a call or a bare name. Anything else raises `Cannot pipe into <expr>`.

So, for now, these are equivalent, although the last three parse to different trees:

```flt
identity 1               # identity(1); parses the same as identity(1)
identity(1)
1 |> identity
1 |> identity()
```

### 2. A bare function name is allowed on the right

`x |> f` calls `f(x)`. Elixir warns about a bare `f` and prefers `f()`; flt accepts both. Since the
name goes through the normal lookup, a function bound to another name can be piped into.

A bare name isn't ambiguous: it always means "call this with the piped value". It's also the only way
the parser could treat `x |> f` consistently, since it can't tell a global function's name from a
variable bound to a function value. Both are identifiers, resolved at run time by the same lookup
(ADR 003, decision 3).

### 3. Calls next to a pipe need parentheses

A call written without parentheses takes everything after its name as its arguments, which makes it
ambiguous next to a pipe. So the whitespace call form is not allowed on either side of `|>`:

- **On the right**, a call needs parentheses. The right side of `|>` is parsed like an `if`
  condition, where a name followed by whitespace isn't a call. So `1 |> f 2` is `1 |> f` followed by
  a stray `2`, which is a parse error. Write `1 |> f(2)`.
- **On the left**, a call without parentheses can't take a pipe as an argument. `f 1 |> g()` would
  otherwise mean `f(1 |> g())`, not `f(1) |> g()`, so it's a parse error. Write `f(1) |> g()`, or
  `f(1 |> g())` if that's what's meant.

Because the right side is parsed like an `if` condition, `if x |> f "Ok" else "No"` reads as the
condition `x |> f` followed by the `"Ok"` branch.

### 4. Argument counts include the piped value

`1 |> typeof(2)` calls `typeof(1, 2)` and raises
`Function typeof expected 1 argument(s), found 2`, as Elixir reports `typeof/2`.

### 5. `|>` reads like Elixir

`x |> f == 3` is `(x |> f) == 3`, as in Elixir. Elixir puts `|>` below `+`, `-` and `<>`, and above
comparisons and the logical operators:

| Expression        | flt before         | Elixir and flt after |
| ----------------- | ------------------ | -------------------- |
| `1 + 1 \|> f`     | `f(2)`             | `f(2)`               |
| `x \|> f == 3`    | `x \|> (f == 3)`   | `f(x) == 3`          |
| `a \|\| b \|> f`  | `f(a \|\| b)`      | `a \|\| f(b)`        |

That needs comparisons to bind looser than arithmetic, so the precedence becomes, lowest to highest:

| Level | Operators              | Change                                    |
| ----- | ---------------------- | ----------------------------------------- |
| 1     | `\|\|`                 |                                           |
| 2     | `&&`                   |                                           |
| 3     | `^^`                   |                                           |
| 4     | `==` `!=`              | moved down from the `*` `/` level         |
| 5     | `<` `>` `<=` `>=`      | moved down from the `*` `/` level         |
| 6     | `\|`                   |                                           |
| 7     | `^`                    |                                           |
| 8     | `&`                    |                                           |
| 9     | `\|>`                  | moved up from level 1                     |
| 10    | `+` `-` `<>`           |                                           |
| 11    | `*` `/`                |                                           |
| —     | unary, then `.field`   |                                           |

- `|>` sits directly below `+ - <>` and above everything else, so `x |> f & 1` is `(x |> f) & 1`.
- Bitwise operators stay tighter than comparisons (as in Rust), so `x & 1 == 0` is `(x & 1) == 0`.
  Elixir puts `&&&` and `|||` below comparisons, but flt keeps its existing order among the
  non-pipe operators.
- `==`/`!=` bind looser than `<`/`>`/`<=`/`>=`, as in Elixir.
- Comparisons bind looser than arithmetic, `<>` and bitwise operators, so `1 + 2 == 3` is `true`
  and `"a" <> "b" == "ab"` is `true`.

## Alternatives Considered

- **Keep `|>` at the lowest precedence.** Rejected. `x |> f == 3` reading as `x |> (f == 3)`
  surprises anyone coming from Elixir, and makes pipes awkward to compare.
- **Pipe into the last argument** (as in F# or Clojure's `->>`). Rejected. Elixir's first-argument
  rule suits data-first functions like `select(df, :id)`.
- **Require parentheses on every function to the right of `|>`, including bare names.** Rejected.
  The parser can't tell a function's name from a variable bound to a function value, so `x |> f`
  would have to be rejected for both, though neither is ambiguous.
- **Warn about calls without parentheses next to a pipe, instead of rejecting them.** Rejected for
  now. flt has no way to report warnings, and the AST doesn't record whether a call had parentheses.
- **Pipe into any expression that evaluates to a function**, e.g. `x |> (pick_fn())`. Deferred.
  Calls and bare names cover the use cases so far.

## Consequences

- The runtime evaluates `|>` and the `Pipe operator not yet supported` error is removed, replaced by
  `Cannot pipe into <expr>`.
- Calls without parentheses next to a pipe are parse errors. Inside the parentheses of a call on the
  right of `|>`, calls need parentheses too, as in an `if` condition: `x |> f(g(1))`, not
  `x |> f(g 1)`.
- The precedence change is breaking: `"a" <> "b" == "ab"` evaluated to `"afalse"` before, and the
  parse tree changes for AST consumers. Other mixed expressions raised an error before.
- Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not
  supported yet, since the REPL reads one line at a time.

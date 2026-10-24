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

### 2. A bare function name is allowed on the right

`x |> f` calls `f(x)`. Elixir warns about a bare `f` and prefers `f()`; flt accepts both. Since the
name goes through the normal lookup, a function bound to another name can be piped into.

### 3. Argument counts include the piped value

`1 |> typeof(2)` calls `typeof(1, 2)` and raises
`Function typeof expected 1 argument(s), found 2`, as Elixir reports `typeof/2`.

### 4. `|>` reads like Elixir

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
- **Pipe into any expression that evaluates to a function**, e.g. `x |> (pick_fn())`. Deferred.
  Calls and bare names cover the use cases so far.

## Consequences

- The runtime evaluates `|>` and the `Pipe operator not yet supported` error is removed, replaced by
  `Cannot pipe into <expr>`.
- The precedence change is breaking: `"a" <> "b" == "ab"` evaluated to `"afalse"` before, and the
  parse tree changes for AST consumers. Other mixed expressions raised an error before.
- Multi-line pipes in the REPL (a line starting with `|>` continuing the previous one) are not
  supported yet, since the REPL reads one line at a time.

# ADR 009: Symbol Keys and Ordered Maps

- **Date:** 2026-10-26
- **Status:** Accepted.

## Context

A bare key in a map literal looks like a symbol but is a string. `{b: 1, a: 2}` has the string keys
`"b"` and `"a"`, the same as `{"b": 1, "a": 2}`, and prints as `{"b": 1, "a": 2}`. A symbol can't be
a key at all.

That is at odds with how hosts use symbols. In datu, columns are symbols, and keyword arguments,
which reach a function as one trailing map ([ADR 004](004-pipe-operator.md)), name output columns:

```flt
read("x.parquet") |> select(:id, total: sum(:amount), average: avg(:amount))
```

`:id` is a symbol, but `total` and `average` arrive as strings. The handler has to treat the same
kind of name two ways.

Until recently, maps also had no order. `SimpleRuntime` stored them in a `HashMap`, so a handler
iterating a map saw its entries in an arbitrary order, and maps printed with their keys sorted. The
order of `total` and `average` above, which should be the order of the output columns, was lost.
PR #25 made `SimpleRuntime` keep insertion order, as a detail of the runtime. This ADR records that
choice alongside symbol keys, since both decide what a handler sees in a map.

## Decisions

### 1. Every key in a map literal is a symbol

As in [Elixir](https://hexdocs.pm/elixir/keywords-and-maps.html), `b: 1` is shorthand for the key
`:b`, and a quoted key `"c d": 1` for the quoted symbol `:"c d"`:

```flt
{b: 1, a: 2}         # keys :b and :a
{"c d": 1}           # key :"c d"
{b: 1} == {"b": 1}   # true: both have the key :b
```

A bare key follows the rules of a bare symbol: a letter, then letters, digits or `_`. A quoted key
can be any string, so a key needn't be a valid name.

### 2. String keys are left to a later decision

A map can also have string keys, but flt source can't write them yet; only a host can create
them. A later decision is expected to add a way, either a syntax such as `{"b" => 1}` or map
functions such as `Map.put(map, "b", 1)`. A symbol and a string with the same text are different
keys.

### 3. Keyword arguments follow the same rules

Keyword arguments use map-key syntax, so `f(total: 1)` passes the key `:total`. A handler sees the
same kind of name for `:id` and for `total:`.

### 4. Field access reads a symbol key

`m.b` reads the key `:b`:

```flt
{b: 1}.b             # 1
{"b": 1}.b           # 1
```

A key that isn't a valid name, such as `:"c d"`, can't be read with `.`, nor can a string key. A
function for reading any key is left for later.

### 5. Maps print their keys as written

A symbol key prints bare if it's a valid bare key, and quoted otherwise, so a printed map reads back
as the same map:

```flt
{b: 1, "c d": 2}     # {b: 1, "c d": 2}
{"b": 1}             # {b: 1}
```

A string key, which only a host can create, prints as `"b" => 1`, the form decision 2 mentions. It
can't be read back yet.

A symbol value follows the same rule, so it too reads back as written:

```flt
:b                   # :b
:"c d"               # :"c d"
```

### 6. In `SimpleRuntime`, maps keep insertion order

A map keeps its entries in the order they were inserted, which for a literal or a call's keyword
arguments is source order. A handler iterating the map sees the same order, and the map prints in
that order:

```flt
{b: 1, a: 2}         # {b: 1, a: 2}
{n: 1, m: 2, n: 3}   # {n: 3, m: 2}
```

A duplicate key keeps its first position and its last value, so `{b: 1, "b": 2}` is `{b: 2}`. Maps
used to print with their keys sorted; now that they have an order, printing it is the truthful
choice.

### 7. Map equality ignores order

```flt
{a: 1, b: 2} == {b: 2, a: 1}   # true
```

Two maps are equal if they have the same keys with equal values, as before.

### 8. Keys are the language's; order is the runtime's

The spec says what keys a map literal makes (decisions 1 to 5) and that equality ignores order
(decision 7). Whether a map keeps an order is up to each runtime, like the rules of
[ADR 006](006-no-shadowing.md) (decision 5). `SimpleRuntime` keeps insertion order (decision 6).

## Alternatives Considered

- **Keeping string keys.** Rejected: bare keys look like symbols, and hosts that name things with
  symbols would see two kinds of names.
- **Quoted keys as strings**, so that `{"b": 1}` has a string key. Rejected: `"b": 1` reads as the
  same kind of key as `b: 1`, as it does in Elixir, and string keys are better served by a syntax
  or functions of their own (decision 2).
- **Treating `:b` and `"b"` as the same key.** Rejected: two different values would collide, and a
  handler couldn't tell which one a host inserted.
- **Field access falling back to a string key.** Rejected: `m.b` would mean one of two keys,
  depending on what the map holds.
- **Ordering only keyword arguments**, through a separate `KeywordArgs` value. Rejected: two map
  types for one concept, and map literals would still lose their order.
- **Sorted maps.** Rejected: the order is consistent, but isn't the user's.
- **Order-sensitive equality.** Rejected: maps compare by content, and `{a: 1, b: 2}` and
  `{b: 2, a: 1}` hold the same content.
- **Putting order in the spec.** Rejected: the spec stays minimal, and a runtime may have reasons
  not to keep an order.

## Consequences

- New Rust API:
  - `MapKey`, either `MapKey::Symbol(String)` or `MapKey::String(String)`, with
    `MapKey::symbol(name)` and `MapKey::string(s)`, and `From<MapKey> for Value`, which gives a
    `Symbol` or a `String`. String keys are in the API now, so adding a way to write them later
    won't change the type of `Value::Map` again;
  - `flt::runtime::Map`, from PR #25, is now keyed by `MapKey` instead of `String`.
- Breaking changes:
  - every key in a map literal is a symbol;
  - maps print symbol keys bare and in insertion order: `{b: 1, a: 2}` prints `{b: 1, a: 2}`
    instead of `{"a": 2, "b": 1}`;
  - keyword arguments arrive with symbol keys;
  - `Value::Map` holds a `Map` keyed by `MapKey`, instead of a `HashMap<String, Value>`;
  - symbols print quoted when their name isn't a valid bare name, e.g. `:"hello world"`, so they
    read back as the same symbol.
- `typeof` of a map is still `Map`.

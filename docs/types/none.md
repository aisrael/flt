# None

`None` is both a value and its own type:

```flt
None                  # None
typeof(None)          # None
typeof(None) == None  # true
```

Because `None` is a value and a type at the same time, the `None` value compares equal to the `None` type. See [Types as Values](../types.md#types-as-values).

flt has no unit value `()`. `None` fills that role.

## Where `None` Comes From

- **The `None` literal.**
- **An `if` without `else` whose condition is `false`.** An `if` without an `else` produces the then-branch's value when the condition is `true`, and `None` when it is `false`:

  ```flt
  if true 1            # 1
  if false 1           # None
  typeof(if false 1)   # None
  ```

  So an `if` without `else` whose then-branch has type `T` has the [union type](./union_types.md) `T | None`.

## `None` and `Option<T>`

`None` is also the empty value of an [`Option<T>`](../types.md#option), the wrapper type used for optional parameters such as `compression: ?CompressionType`.

`Option<T>` and the union `T | None` describe similar values, a `T` or `None`, but they are different types. `Option<T>` is a wrapper, while `T | None` is a plain union.

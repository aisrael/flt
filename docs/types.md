# Types

`flt` has a handful of built-in types:

## Built-In Types

- `Boolean`
- `Number`
- `String`
- `Symbol`

## Wrapper Types

### Option

The `Option` is an internal type acts like Rust's generic `Option<T>` type, without formally introducing generics to `flt`.

That is, given a function:

```flt
def read(path: String, compression: ?CompressionType)
```

Then the internal type of `compressed` is `Option<CompressionType>`.

`Option<T>` is either a value of the wrapped type `T`, or `None`. To check if an `Option<T>` has an actual value, compare it to `None`, or use the `?` post-fix operator:

```flt
def read(path: String, compressed: ?CompressionType)
  if compressed?
    // handle compressed file
  else
    // read file regularly
  end
end
``` 

Alternatively

```flt
def read(path: String, compressed: ?CompressionType)
  if compressed == None
    // read file regularly
  else
    // handle compressed file
  end
end
``` 

## Collection Types

- `Map`
- `KeywordArgs`
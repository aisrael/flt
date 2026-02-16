# flt - a 'lite' Functional Language

## Example

```
READ("fixtures/table.parquet) |>
HEAD(10) |>
SELECT("two", "four") |>
WRITE("test.json", pretty: true)
```

## Literals

```
3.1415926535 # a Number (a BigDecimal)
"foo"        # a String
true         # a Bool

## Stage 1

```elixir
read("input.parquet") |> # RecordBatchSource
write("output.avro")     # WriteResults
```

### Additional language features needed

- Comments, starting with `#` and up to the end of the line
- function signatures (input arguments and types, output type)
- 'intrinsic' functions, or, functions defined in the runtime
- ability for the interpreter


## Stage 2

```elixir
read("input.parquet")   |> # RecordBatchSource
sort_by(:salary, :desc) |> # RecordBatchSource
write("output.avro")       # WriteResults
```

```elixir

fn sort_by(source: RecordBatchSource)

### Additional language features needed

- Add optional, trailing parameters to function calls:


### Stage 3

read("input.parquet") |>                 # ParquetFileReader -> columnar, RecordBatchSource
group_by(:title) |>                      # key -> array, :title -> RecordBatchSource
transform_values(
    average(:salary)                     # RecordBatchSource -> BigDecimal
, as: :average_salary) |>                # key -> BigDecimal, :title -> BigDecimal (RecordBatchSource with no column names)
sort_by(:average_salary, :desc) |>       # sort by the second element of the tuple (average salary) descending (RecordBatchSource)
write("output.avro") |>                  # AvroFileWriter

read("input.parquet") |>
group_by(:title) |>
transform_values(average(:salary)) |>
sort_by(:average_salary, :desc) |>
write("output.avro") |>


fn average(source: RecordBatchSource, column: Symbol) -> BigDecimalWithMetadata {
    source |>
    reduce(0, |sum, recordbatch|
      recordbatch |> sum_by(column)
    ) |>
    replace_metadata(name: "average_{column}")
end

fn sum_by(recordbatch: RecordBatch, column: Symbol) -> BigDecimalWithMetadata {
    let sum = recordbatch |>
        reduce(0, |sum, row|
            sum + row |> get_or_default(column, 0)
        )

    recordbatch |>
    get_metadata(column) |>
    merge(name: "total_{column}") |>
    BigDecimalWithMetadata(sum)
}

Feature: Native types and values

  A host can define native types for its own Rust values. Its native functions
  create and read values of them; flt source can bind, pass, pipe, print and
  compare them, and get their type, but can't create them or look inside them.
  A native value prints as the host displays it and compares with the host's
  equality. A native type's name evaluates to that type and can't be bound. See
  docs/native-functions-and-values.md#native-types-and-values.

  The fixtures register chrono's `NaiveDate` as the native type `Date`, with
  `date(yyyy, mm, dd) -> Date` and `days_between(from: Date, to: Date) -> Number`.
  `date` reads its arguments with `i32::try_from` and `u32::try_from`.
  `time(hh, mm, ss) -> Time` makes a value of a second native type, `NaiveTime`,
  which the fixtures never register.

  Background:
    Given the native function "date"
    And the native function "days_between"

  Scenario: A native value prints as the host displays it
    Given the input 'date(2026, 10, 25)'
    When I evaluate the input
    Then the result should be "2026-10-25"

  Scenario Outline: A native function reads native values
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<days>"

    Examples:
      | input                                              | days |
      | days_between(date(2026, 1, 1), date(2026, 10, 25)) | 297  |
      | days_between(date(2026, 10, 25), date(2026, 1, 1)) | -297 |
      | days_between(date(2024, 2, 28), date(2024, 3, 1))  | 2    |

  Scenario: Native values pass through bindings and pipes
    Given the input 'let start = date(2026, 1, 1); start |> days_between(date(2026, 10, 25))'
    When I evaluate the input
    Then the result should be "297"

  Scenario: The type of a native value is its native type
    Given the input 'typeof(date(2026, 10, 25))'
    When I evaluate the input
    Then the result should be "Date"

  Scenario: A native type's name evaluates to that type
    Given the input 'typeof(date(2026, 10, 25)) == Date'
    When I evaluate the input
    Then the result should be "true"

  Scenario: A native type is a type
    Given the input 'typeof(Date)'
    When I evaluate the input
    Then the result should be "Type"

  Scenario Outline: Function types can name native types
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<type>"

    Examples:
      | input                 | type                               |
      | typeof(date)          | fn(Number, Number, Number) -> Date |
      | typeof(days_between)  | fn(Date, Date) -> Number           |

  Scenario Outline: Native values compare with the host's equality
    Given the native function "time"
    And the input '<input>'
    When I evaluate the input
    Then the result should be "<result>"

    Examples:
      | input                                         | result |
      | let d = date(2026, 10, 25); d == d            | true   |
      | date(2026, 10, 25) == date(2026, 10, 25)      | true   |
      | date(2026, 10, 25) == date(2026, 10, 26)      | false  |
      | date(2026, 10, 25) != date(2026, 10, 26)      | true   |
      | date(2026, 10, 25) == time(12, 0, 0)          | false  |
      | date(2026, 10, 25) == "2026-10-25"            | false  |

  Scenario: Native values can't be ordered
    Given the input 'date(2026, 10, 25) < date(2026, 10, 26)'
    When I evaluate the input
    Then the evaluation should fail with "Invalid Operand Type"

  Scenario: A native function rejects a value of the wrong type
    Given the input 'days_between(1, date(2026, 10, 25))'
    When I evaluate the input
    Then the evaluation should fail with "Expected Date, found Number"

  Scenario: A native function rejects a value of another native type
    Given the native function "time"
    And the input 'days_between(time(12, 0, 0), date(2026, 10, 25))'
    When I evaluate the input
    Then the evaluation should fail with "Expected Date, found Time"

  Scenario: flt source can't create a native value
    Given the input 'Date(2026, 10, 25)'
    When I evaluate the input
    Then the evaluation should fail with "Date is not a function"

  Scenario: flt source can't look inside a native value
    Given the input 'date(2026, 10, 25).year'
    When I evaluate the input
    Then the evaluation should fail with "Invalid Operand Type"

  Scenario: Concatenating a native value gives the empty string
    Given the input '"date: " <> date(2026, 10, 25)'
    When I evaluate the input
    Then the result should be '"date: "'

  Scenario: A native type's name can't be bound
    Given the input 'let Date = 1'
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind Date: it's a native type"

  Scenario: A value of an unregistered native type still has its type
    Given the native function "time"
    And the input 'typeof(time(12, 0, 0))'
    When I evaluate the input
    Then the result should be "Time"

  Scenario: An unregistered native type's name is unbound
    Given the native function "time"
    And the input 'Time'
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: Time"

  Scenario Outline: A native function's number arguments convert to Rust integers
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "<error>"

    Examples:
      | input                 | error                         |
      | date("2026", 10, 25)  | Expected Number, found String |
      | date(2026.5, 10, 25)  | Cannot convert 2026.5 to i32  |
      | date(2026, -10, 25)   | Cannot convert -10 to u32     |

  Scenario: A host error is shown unchanged
    Given the input 'date(2026, 2, 30)'
    When I evaluate the input
    Then the evaluation should fail with exactly "2026-02-30 is not a valid date"

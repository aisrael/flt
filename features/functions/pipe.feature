Feature: Pipe operator

  The pipe operator `|>` passes the value on its left as the first argument of
  the function on its right: `x |> f(a)` is `f(x, a)`, and `x |> f` is `f(x)`.
  `|>` binds looser than `+`, `-` and `<>`, and tighter than comparisons and
  logical operators. See docs/functions-and-pipe.md#pipe-operator.

  Scenario: Piping into a bare function name
    Given the input '1 |> typeof'
    When I evaluate the input
    Then the result should be "Number"

  Scenario: Piping into a call with no other arguments
    Given the input '1 |> typeof()'
    When I evaluate the input
    Then the result should be "Number"

  Scenario: Chaining pipes
    Given the input '1 |> typeof |> typeof'
    When I evaluate the input
    Then the result should be "Type"

  Scenario: Piping into a function bound to another name
    Given the native function "identity"
    And the input 'let g = identity; 42 |> g'
    When I evaluate the input
    Then the result should be "42"

  Scenario: The piped value counts as an argument
    Given the input '1 |> typeof(2)'
    When I evaluate the input
    Then the evaluation should fail with "Function typeof expected 1 argument(s), found 2"

  Scenario: Piping into something that isn't a function
    Given the input '1 |> 2'
    When I evaluate the input
    Then the evaluation should fail with "Cannot pipe into 2"

  Scenario: Piping into a value that isn't a function
    Given the input '1 |> Number'
    When I evaluate the input
    Then the evaluation should fail with "Number is not a function"

  Scenario: Piping into an unknown function
    Given the input '1 |> nope()'
    When I evaluate the input
    Then the evaluation should fail with "Undefined function: nope"

  Scenario: A pipe binds tighter than a comparison
    Given the input '1 |> typeof == Number'
    When I evaluate the input
    Then the result should be "true"

  Scenario: Arithmetic binds tighter than a pipe
    Given the input '1 + 1 |> typeof'
    When I evaluate the input
    Then the result should be "Number"

  Scenario: Arithmetic binds tighter than a comparison
    Given the input '1 + 2 == 3'
    When I evaluate the input
    Then the result should be "true"

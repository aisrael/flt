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

  Scenario Outline: A call and a pipe into a function are equivalent
    Given the native function "identity"
    And the input '<input>'
    When I evaluate the input
    Then the result should be "<result>"

    Examples:
      | input               | result |
      | identity 1          | 1      |
      | identity(1)         | 1      |
      | 1 \|> identity      | 1      |
      | 1 \|> identity()    | 1      |
      | typeof 1            | Number |
      | typeof(1)           | Number |
      | 1 \|> typeof        | Number |
      | 1 \|> typeof()      | Number |

  Scenario Outline: A call next to a pipe needs parentheses
    Given the input '<input>'
    When I parse the input
    Then parsing should fail

    Examples:
      | input              |
      | 1 \|> f 2          |
      | 1 \|> f 2, 3       |
      | 1 \|> f k: 2       |
      | f 1 \|> g()        |
      | f 1, 2 \|> g()     |
      | f 1 == 2 \|> g()   |
      | f k: 1 \|> g()     |

  Scenario: Piping a parenthesized call
    Given the native function "identity"
    And the input 'identity(1) |> typeof'
    When I evaluate the input
    Then the result should be "Number"

  Scenario: A parenthesized pipe as a call's argument
    Given the input 'typeof (1 |> typeof)'
    When I evaluate the input
    Then the result should be "Type"

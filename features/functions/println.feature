Feature: println

  `println` is a native function that the runtime registers in the global scope,
  of type `fn(String) -> None`. It prints its argument and a new line to standard
  output, and evaluates to `None`. It is not `const`.
  See docs/functions-and-pipe.md#built-in-functions.

  Scenario: println evaluates to None
    Given the input 'println("Hello, world!")'
    When I evaluate the input
    Then the result should be "None"

  Scenario: The type of println
    Given the input 'typeof(println)'
    When I evaluate the input
    Then the result should be "fn(String) -> None"

  Scenario: println can be piped into
    Given the input '"Hello" |> println'
    When I evaluate the input
    Then the result should be "None"

  Scenario: println prints an interpolated string
    Given the input 'let x = 1; println("x is {x}")'
    When I evaluate the input
    Then the result should be "None"

  Scenario: println can't be bound
    Given the input 'let println = 1'
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind println: it's a function"

  Scenario: println accepts only a String
    Given the input 'println(1)'
    When I evaluate the input
    Then the evaluation should fail with "Argument s of println expects String, found Integer"

  Scenario: Calling println with the wrong number of arguments
    Given the input 'println()'
    When I evaluate the input
    Then the evaluation should fail with "Function println expected 1 argument(s), found 0"

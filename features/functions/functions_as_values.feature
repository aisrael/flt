Feature: Functions as values

  A host can register native functions in the global scope. A function's name
  evaluates to a function value, which has a function type and can be bound to
  another name and called through it. Names resolve to variables, then global
  functions, then built-in type names. See docs/functions-and-pipe.md#native-functions.

  Background:
    Given the native function "identity"

  Scenario: A native function's name evaluates to a function value
    Given the input 'identity'
    When I evaluate the input
    Then the result should be "<fn identity>"

  Scenario: Calling a native function
    Given the input 'identity(42)'
    When I evaluate the input
    Then the result should be "42"

  Scenario: The type of a native function
    Given the input 'typeof(identity)'
    When I evaluate the input
    Then the result should be "fn(Any) -> Any"

  Scenario: Calling a function through another name
    Given the input 'let f = identity; f(42)'
    When I evaluate the input
    Then the result should be "42"

  Scenario: A function value equals itself
    Given the input 'let f = identity; f == identity'
    When I evaluate the input
    Then the result should be "true"

  Scenario: A variable shadows a native function
    Given the input 'let identity = 1; identity'
    When I evaluate the input
    Then the result should be "1"

  Scenario: Calling with the wrong number of arguments
    Given the input 'identity(1, 2)'
    When I evaluate the input
    Then the evaluation should fail with "Function identity expected 1 argument(s), found 2"

  Scenario: Calling a non-function
    Given the input 'let x = 1; x(2)'
    When I evaluate the input
    Then the evaluation should fail with "x is not a function"

  Scenario: Calling an unknown function
    Given the input 'nope(1)'
    When I evaluate the input
    Then the evaluation should fail with "Undefined function: nope"

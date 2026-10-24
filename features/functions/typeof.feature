Feature: typeof

  `typeof` is a native function that the runtime registers in the global scope,
  of type `fn(Any) -> Type`. It's a function value like any other, so it can be
  bound to another name, piped into through that name, and shadowed by a
  variable. See docs/functions-and-pipe.md#built-in-functions.

  Scenario: typeof's name evaluates to a function value
    Given the input 'typeof'
    When I evaluate the input
    Then the result should be "<fn typeof>"

  Scenario: The type of typeof
    Given the input 'typeof(typeof)'
    When I evaluate the input
    Then the result should be "fn(Any) -> Type"

  Scenario: Calling typeof through another name
    Given the input 'let t = typeof; t(42)'
    When I evaluate the input
    Then the result should be "Number"

  Scenario: typeof equals itself
    Given the input 'let t = typeof; t == typeof'
    When I evaluate the input
    Then the result should be "true"

  Scenario: A variable shadows typeof
    Given the input 'let typeof = 1; typeof'
    When I evaluate the input
    Then the result should be "1"

  Scenario: Calling a shadowed typeof
    Given the input 'let typeof = 1; typeof(2)'
    When I evaluate the input
    Then the evaluation should fail with "typeof is not a function"

  Scenario: Calling typeof with the wrong number of arguments
    Given the input 'typeof()'
    When I evaluate the input
    Then the evaluation should fail with "Function typeof expected 1 argument(s), found 0"

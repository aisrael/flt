Feature: typeof

  `typeof` is a native function that the runtime registers in the global scope,
  of type `fn(Any) -> Type`. It's a function value like any other, so it can be
  bound to another name and piped into through that name. Like any function's
  name, `typeof` can't be bound. See docs/functions-and-pipe.md#built-in-functions.

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
    Then the result should be "Integer"

  Scenario: typeof equals itself
    Given the input 'let t = typeof; t == typeof'
    When I evaluate the input
    Then the result should be "true"

  Scenario: typeof can't be bound
    Given the input 'let typeof = 1'
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind typeof: it's a function"

  Scenario: Calling typeof with the wrong number of arguments
    Given the input 'typeof()'
    When I evaluate the input
    Then the evaluation should fail with "Function typeof expected 1 argument(s), found 0"

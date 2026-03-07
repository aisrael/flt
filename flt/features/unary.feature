Feature: Unary expressions

  Unary operators `!`, `+`, and `-` apply to expressions. `!` is logical negation,
  `+` is unary plus, and `-` is negation.

  Scenario: Logical not on boolean
    Given the input "!true"
    When I parse the input
    Then the output should parse to unary Not of boolean true

  Scenario: Logical not on identifier
    Given the input "!x"
    When I parse the input
    Then the output should parse to unary Not of identifier "x"

  Scenario: Unary minus on number
    Given the input "-42"
    When I parse the input
    Then the output should parse to unary Minus of number 42

  Scenario: Unary plus on number
    Given the input "+17"
    When I parse the input
    Then the output should parse to unary Plus of number 17

  Scenario: Unary minus on identifier
    Given the input "-x"
    When I parse the input
    Then the output should parse to unary Minus of identifier "x"

  Scenario: Double negation
    Given the input "--42"
    When I parse the input
    Then the output should parse to unary Minus of unary Minus of number 42

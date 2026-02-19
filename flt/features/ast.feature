Feature: AST

  Scenario: Parsing a number
    Given the input "42"
    When I parse the input
    Then the output should be a `Literal::Number(42)`

  Scenario: parsing a string
    Given the input '"hello"'
    When I parse the input
    Then the output should be a `Literal::String("hello")`

  Scenario: parsing a boolean
    Given the input "true"
    When I parse the input
    Then the output should be a `Literal::Boolean(true)`

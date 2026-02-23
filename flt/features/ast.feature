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

  Scenario: parsing string concatenation
    Given the input '"foo" <> "bar"'
    When I parse the input
    Then the output should parse to string concat "foo" and "bar"

  Scenario: parsing string interpolation
    Given the input '"Hello, {who}!"'
    When I parse the input
    Then the output should parse to interpolated string "Hello, " who "!"

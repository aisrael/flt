Feature: AST

  Scenario: Parsing a number
    Given the input "42"
    When I parse the input
    Then the output should be a `Literal::Number(42)`

  Scenario: parsing a string
    Given the input '"hello"'
    When I parse the input
    Then the output should be 'Literal(String("hello"))'

  Scenario: parsing a boolean
    Given the input "true"
    When I parse the input
    Then the output should be a `Literal::Boolean(true)`

  Scenario: parsing an identifier
    Given the input "user_name"
    When I parse the input
    Then the output should be an identifier "user_name"

  Scenario: parsing a symbol
    Given the input ":user_name"
    When I parse the input
    Then the output should be a `Literal::Symbol("user_name")`

  Scenario: parsing a quoted symbol
    Given the input ':"hello world"'
    When I parse the input
    Then the output should be a `Literal::Symbol("hello world")`

  Scenario: parsing an identifier that starts with digits fails
    Given the input "123abc"
    When I parse the input
    Then parsing should fail

  Scenario: parsing a symbol that starts with non-character fails
    Given the input ":_foo"
    When I parse the input
    Then parsing should fail

  Scenario: parsing string concatenation
    Given the input '"foo" <> "bar"'
    When I parse the input
    Then the output should parse to string concat "foo" and "bar"

  Scenario: parsing string interpolation
    Given the input '"Hello, {who}!"'
    When I parse the input
    Then the output should parse to interpolated string "Hello, " who "!"

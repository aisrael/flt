Feature: AST

  Scenario: Parsing a number
    Given the input "42"
    When I parse the input
    Then the output should be a `Literal::Number(42)`

  Scenario: Parsing a string literal
    Given the input '"hello"'
    When I parse the input
    Then the output should be 'Literal(String("hello"))'

  Scenario: Parsing a boolean literal
    Given the input "true"
    When I parse the input
    Then the output should be a `Literal::Boolean(true)`

  Scenario: Parsing an identifier
    Given the input "user_name"
    When I parse the input
    Then the output should be an identifier "user_name"

  Scenario: Parsing a symbol
    Given the input ":user_name"
    When I parse the input
    Then the output should be a `Literal::Symbol("user_name")`

  Scenario: Parsing a quoted symbol
    Given the input ':"hello world"'
    When I parse the input
    Then the output should be a `Literal::Symbol("hello world")`

  Scenario: Parsing an identifier that starts with digits fails
    Given the input "123abc"
    When I parse the input
    Then parsing should fail

  Scenario: Parsing a symbol that starts with non-character fails
    Given the input ":_foo"
    When I parse the input
    Then parsing should fail

  Scenario: Parsing string concatenation
    Given the input '"foo" <> "bar"'
    When I parse the input
    Then the output should parse to string concat "foo" and "bar"

  Scenario: Parsing string interpolation
    Given the input '"Hello, {who}!"'
    When I parse the input
    Then the output should parse to interpolated string "Hello, " who "!"

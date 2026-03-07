Feature: Binary expressions

  Binary operators combine two expressions. Precedence (lowest to highest):
  `|>`, `||`, `&&`, `^^`, `|`, `^`, `&`, `+`/`-`/`<>`, `*`, `/`.

  Scenario: Addition
    Given the input "1 + 2"
    When I parse the input
    Then the output should parse to addition of 1 and 2

  Scenario: Subtraction
    Given the input "10 - 3"
    When I parse the input
    Then the output should parse to subtraction of 10 and 3

  Scenario: Multiplication
    Given the input "4 * 5"
    When I parse the input
    Then the output should parse to multiplication of 4 and 5

  Scenario: Division
    Given the input "20 / 4"
    When I parse the input
    Then the output should parse to division of 20 and 4

  Scenario: Multiplication has higher precedence than addition
    Given the input "1 + 2 * 3"
    When I parse the input
    Then the output should parse to addition with right side multiplied

  Scenario: Parentheses override precedence
    Given the input "(1 + 2) * 3"
    When I parse the input
    Then the output should parse to multiplication of parenthesized sum by 3

  Scenario: Logical and
    Given the input "true && false"
    When I parse the input
    Then the output should parse to logical and of true and false

  Scenario: Logical or
    Given the input "true || false"
    When I parse the input
    Then the output should parse to logical or of true and false

  Scenario: Logical xor
    Given the input "true ^^ false"
    When I parse the input
    Then the output should parse to logical xor of true and false

  Scenario: String concatenation
    Given the input '"foo" <> "bar"'
    When I parse the input
    Then the output should parse to string concat "foo" and "bar"

  Scenario: Pipe operator
    Given the input "x |> f"
    When I parse the input
    Then the output should parse to pipe of "x" into "f"

Feature: Expression statements

  Scenario: Number expression as a statement
    Given the input "42"
    When I parse the input
    Then the output should be a `Literal::Number(42)`


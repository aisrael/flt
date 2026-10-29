Feature: Assignment

  Scenario: Declaring a variable with let
    Given the input "let x = 1;"
    When I parse the input
    Then the statement should be a declaration of "x" without a type
    And the statement should print as "let x = 1"

  Scenario: Assigning a value without a let keyword
    Given the input "x = 1;"
    When I parse the input
    Then the statement should be an assignment to "x"
    And the statement should print as "x = 1"

  Scenario Outline: A reserved keyword can't be a binding name
    Given the input "<input>"
    When I parse the input
    Then parsing should fail

    Examples:
      | input       |
      | let if = 1  |
      | let fn = 1  |
      | let let = 1 |
      | for = 1     |
      | not = 1     |

  Scenario: A name that starts with a keyword can be bound
    Given the input "iffy = 1"
    When I parse the input
    Then the statement should be an assignment to "iffy"

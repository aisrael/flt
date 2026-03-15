Feature: Arrays

  Scenario: Parsing an empty array
    Given the input "[]"
    When I parse the input
    Then the output should be an empty array

  Scenario: Parsing an array with spaces
    Given the input "[  ]"
    When I parse the input
    Then the output should be an empty array

  Scenario: Parsing an array with a single number
    Given the input "[ 42 ]"
    When I parse the input
    Then the output should be an array with 1 element
    And the first element should be the number 42

  Scenario: Parsing an array with a single string
    Given the input '[ "hello" ]'
    When I parse the input
    Then the output should be an array with 1 element
    And the first element should be the string "hello"

  Scenario: Parsing an array with multiple numbers
    Given the input "[ 1, 2, 3 ]"
    When I parse the input
    Then the output should be an array with 3 elements
    And the first element should be the number 1
    And the second element should be the number 2
    And the third element should be the number 3

  Scenario: Parsing an array with mixed types
    Given the input '[ 1, "two", true ]'
    When I parse the input
    Then the output should be an array with 3 elements
    And the first element should be the number 1
    And the second element should be the string "two"
    And the third element should be the boolean true

  Scenario: Parsing an array with trailing comma
    Given the input "[ 1, ]"
    When I parse the input
    Then the output should be an array with 1 element
    And the first element should be the number 1

  Scenario: Parsing an array with nested expressions
    Given the input "[ 1 + 2, 3 * 4 ]"
    When I parse the input
    Then the output should be an array with 2 elements

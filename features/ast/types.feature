Feature: Types

  Built-in types in `flt` include Boolean, Number, String, and Symbol.
  Collection types include Map and KeywordArgs. `None` is the empty Option sentinel.

  Scenario: Boolean type name parses as an identifier
    Given the input "Boolean"
    When I parse the input
    Then the output should be an identifier "Boolean"

  Scenario: Number type name parses as an identifier
    Given the input "Number"
    When I parse the input
    Then the output should be an identifier "Number"

  Scenario: String type name parses as an identifier
    Given the input "String"
    When I parse the input
    Then the output should be an identifier "String"

  Scenario: Symbol type name parses as an identifier
    Given the input "Symbol"
    When I parse the input
    Then the output should be an identifier "Symbol"

  Scenario: Map type name parses as an identifier
    Given the input "Map"
    When I parse the input
    Then the output should be an identifier "Map"

  Scenario: KeywordArgs type name parses as an identifier
    Given the input "KeywordArgs"
    When I parse the input
    Then the output should be an identifier "KeywordArgs"

  Scenario: None sentinel parses as an identifier
    Given the input "None"
    When I parse the input
    Then the output should be an identifier "None"

  Scenario: Comparing a value to None for Option check
    Given the input "compressed == None"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("compressed"), Eq, Ident("None"))'

  Scenario: Checking an option is not None
    Given the input "compressed != None"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("compressed"), Ne, Ident("None"))'

  Scenario: Map literal as a collection-typed value
    Given the input '{ value: 42 }'
    When I parse the input
    Then the output should be a map with key "value" and number value 42

  Scenario: Array literal with mixed built-in types
    Given the input '[ 1, "two", true ]'
    When I parse the input
    Then the output should be an array with 3 elements
    And the first element should be the number 1
    And the second element should be the string "two"
    And the third element should be the boolean true

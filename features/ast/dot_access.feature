Feature: Dot Access

  Scenario: Accessing a field on an identifier
    Given the input "u.foo"
    When I parse the input
    Then the output should be a field access with field "foo"

  Scenario: Chained field access
    Given the input "a.b.c"
    When I parse the input
    Then the output should be a field access with field "c"

  Scenario: Field access on a map literal
    Given the input '{ foo: "bar" }.foo'
    When I parse the input
    Then the output should be a field access with field "foo"

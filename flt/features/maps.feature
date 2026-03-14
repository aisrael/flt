Feature: Maps

  Scenario: Parsing an empty map
    Given the input "{}"
    When I parse the input
    Then the output should be an empty map

  Scenario: Parsing a map with a bare key and string value
    Given the input '{ foo: "bar" }'
    When I parse the input
    Then the output should be a map with key "foo" and string value "bar"

  Scenario: Parsing a map with a bare key and number value
    Given the input "{ abc123: 456 }"
    When I parse the input
    Then the output should be a map with key "abc123" and number value 456

  Scenario: Parsing a map with a quoted key
    Given the input '{ "spaced out": (1 + 1) }'
    When I parse the input
    Then the output should be a map with 1 entry

  Scenario: Parsing a map with multiple entries
    Given the input '{ name: "Alice", age: 30 }'
    When I parse the input
    Then the output should be a map with 2 entries

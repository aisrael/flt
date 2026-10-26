Feature: Function calls

  Scenario: Function call with trailing key-value pairs
    Given the input "foo(1, optional: true)"
    When I parse the input
    Then the output should be a function call "foo" with 1 args and 1 keyword args

  Scenario: Function call with only key-value pairs
    Given the input 'config(format: "csv", header: true)'
    When I parse the input
    Then the output should be a function call "config" with 0 args and 2 keyword args

  Scenario: Function call without parens and trailing key-value pairs
    Given the input "foo 1, optional: true"
    When I parse the input
    Then the output should be a function call "foo" with 1 args and 1 keyword args

  Scenario: Function call with positional args only still works
    Given the input "add(1, 2)"
    When I parse the input
    Then the output should be a function call "add" with 2 args

  Scenario: A map literal is a positional argument
    Given the input "foo({optional: true})"
    When I parse the input
    Then the output should be a function call "foo" with 1 args

  Scenario: A positional argument can't follow a keyword argument
    Given the input "foo(optional: true, 1)"
    When I parse the input
    Then parsing should fail

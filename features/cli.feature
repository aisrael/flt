Feature: flt version

  Scenario: flt version
    When the command `flt version` is run
    Then it should exit with status code 0
    And the output should contain "flt version 0.3.0"

  Scenario: flt -e evaluates an expression
    When the command `flt -e '1 + 2'` is run
    Then it should exit with status code 0
    And the output should contain "3"

  Scenario: flt -e evaluates a program with many statements
    When the command `flt -e 'let x = 2; x * 3'` is run
    Then it should exit with status code 0
    And the output should contain "6"

  Scenario: flt -e fails on a syntax error
    When the command `flt -e '1 +'` is run
    Then it should exit with status code 1
    And the error output should contain "Error"

  Scenario: flt runs a program file
    When the command `flt fixtures/examples/hello.flt` is run
    Then it should exit with status code 0
    And the output should contain "Hello, world!"
    And the output should not contain "None"

  Scenario: flt runs a program file with many statements
    When the command `flt fixtures/examples/multi.flt` is run
    Then it should exit with status code 0
    And the output should contain "c + a is 11"
    And the output should not contain "None"

  Scenario: flt runs a program file that starts with a shebang
    When the command `flt fixtures/examples/shebang.flt` is run
    Then it should exit with status code 0
    And the output should contain "answer: 42"

  Scenario: flt does not print the value of the last statement of a program file
    When the command `flt fixtures/examples/value.flt` is run
    Then it should exit with status code 0
    And the output should not contain "7"

  Scenario: flt fails on a program file that does not exist
    When the command `flt fixtures/examples/missing.flt` is run
    Then it should exit with status code 1
    And the error output should contain "cannot read"

  Scenario: flt fails on a syntax error in a program file
    When the command `flt fixtures/examples/syntax_error.flt` is run
    Then it should exit with status code 1
    And the error output should contain "Error"

  Scenario: flt does not accept -e with a program file
    When the command `flt -e '1' fixtures/examples/hello.flt` is run
    Then it should exit with status code 2

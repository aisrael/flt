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

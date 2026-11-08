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
    When the command `flt features/programs/hello.flt` is run
    Then it should exit with status code 0
    And the output should contain "Hello, world!"

  Scenario: flt runs a program file with many statements
    When the command `flt features/programs/multi.flt` is run
    Then it should exit with status code 0
    And the output should contain "11"

  Scenario: flt runs a program file that starts with a shebang
    When the command `flt features/programs/shebang.flt` is run
    Then it should exit with status code 0
    And the output should contain "42"

  Scenario: flt fails on a program file that does not exist
    When the command `flt features/programs/missing.flt` is run
    Then it should exit with status code 1
    And the error output should contain "cannot read"

  Scenario: flt fails on a syntax error in a program file
    When the command `flt features/programs/syntax_error.flt` is run
    Then it should exit with status code 1
    And the error output should contain "Error"

  Scenario: flt does not accept -e with a program file
    When the command `flt -e '1' features/programs/hello.flt` is run
    Then it should exit with status code 2

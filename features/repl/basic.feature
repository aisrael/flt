Feature: flt repl

  Scenario: evaluate a simple expression
    When the REPL is run and the user types:
      """
      1 + 1
      """
    Then the output should contain "2"

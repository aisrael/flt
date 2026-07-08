Feature: flt repl

  Scenario: evaluate a simple expression
    When the REPL is run and the user types:
      """
      1 + 1
      """
    Then the command should succeed
    And the output should contain "2"

  Scenario: parse an expression without evaluating it
    When the REPL is run and the user types:
      """
      /parse 1 + 1
      """
    Then the command should succeed
    And the output should contain "BinaryExpr"

  Scenario: unknown slash command
    When the REPL is run and the user types:
      """
      /bogus
      """
    Then the command should succeed
    And the output should contain "unknown command"

  Scenario: unset a bound variable
    When the REPL is run and the user types:
      """
      x = 1
      x
      /unset x
      x
      """
    Then the command should succeed
    And the output should contain "UnboundIdentifier"

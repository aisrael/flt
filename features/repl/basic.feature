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

  Scenario: inspect an unbound variable
    When the REPL is run and the user types:
      """
      /inspect x
      """
    Then the command should succeed
    And the output should contain "Unbound variable"

  Scenario: inspect a bound variable
    When the REPL is run and the user types:
      """
      x = 1
      /inspect x
      """
    Then the command should succeed
    And the output should contain "(variable) 1"

  Scenario: inspect a bound variable using the short alias
    When the REPL is run and the user types:
      """
      x = 1
      /i x
      """
    Then the command should succeed
    And the output should contain "(variable) 1"

  Scenario: inspect a scalar literal
    When the REPL is run and the user types:
      """
      /inspect 42
      """
    Then the command should succeed
    And the output should contain "Number"

  Scenario: inspect an array literal
    When the REPL is run and the user types:
      """
      /inspect [1, 2, 3]
      """
    Then the command should succeed
    And the output should contain "Array"

  Scenario: inspect a map literal
    When the REPL is run and the user types:
      """
      /inspect { foo: "bar" }
      """
    Then the command should succeed
    And the output should contain "Map"

  Scenario: quit the REPL
    When the REPL is run and the user types:
      """
      /quit
      1 + 1
      """
    Then the command should succeed
    And the output should not contain "2"

  Scenario: quit the REPL using the short alias
    When the REPL is run and the user types:
      """
      /q
      1 + 1
      """
    Then the command should succeed
    And the output should not contain "2"

  Scenario: show help
    When the REPL is run and the user types:
      """
      /help
      """
    Then the command should succeed
    And the output should contain "/quit"

  Scenario: show help using the short alias
    When the REPL is run and the user types:
      """
      /h
      """
    Then the command should succeed
    And the output should contain "/quit"

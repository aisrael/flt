Feature: flt repl warnings

  The REPL prints each warning to standard error, as `Warning: <message>`,
  after the statement that gave it. See docs/runtime.md#warnings.

  Scenario: a statement after a return
    When the REPL is run and the user types:
      """
      def f() -> Integer return 1; 2 end
      """
    Then the command should succeed
    And the output should contain "Warning: Unreachable code in f: statements after return never run"

  Scenario: the function still works
    When the REPL is run and the user types:
      """
      def f() -> Integer return 1; 2 end
      f()
      """
    Then the command should succeed
    And the output should contain "Warning: Unreachable code in f"
    And the output should contain "1"

  Scenario: no warning without dead code
    When the REPL is run and the user types:
      """
      def f() -> Integer return 1 end
      f()
      """
    Then the command should succeed
    And the output should not contain "Warning"

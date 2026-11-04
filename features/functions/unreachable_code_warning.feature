Feature: A warning for statements after a return

  A statement after a `return` in the same list of statements never runs. The
  reference runtime warns about it when it runs the `def` that declares the
  function, once for each list of statements. A list of statements is a body, a
  brace block or an arm of an `if`. The check follows the syntax, not the flow of
  the program. A warning doesn't stop anything. See docs/errors.md#warnings and
  docs/adrs/024-unreachable-code-warning.md.

  Scenario: A statement after a return in a body
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
      end
      """
    When I evaluate the input
    Then the evaluation should warn with "Unreachable code in f: statements after return never run"
    And the evaluation should give 1 warning

  Scenario: A bare return followed by a statement
    Given the multiline input
      """
      def f()
        return
        1
      end
      """
    When I evaluate the input
    Then the evaluation should warn with "Unreachable code in f: statements after return never run"

  Scenario: A return as the last statement gives no warning
    Given the multiline input
      """
      def f() -> Integer
        let x = 1
        return x
      end
      """
    When I evaluate the input
    Then the evaluation should give no warnings

  Scenario: A function without a return gives no warning
    Given the multiline input
      """
      def f() -> Integer
        1
        2
      end
      """
    When I evaluate the input
    Then the evaluation should give no warnings

  Scenario: Several statements after a return give one warning
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
        3
        4
      end
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  Scenario: A statement after a return in an arm of an expanded if
    Given the multiline input
      """
      def f(c: Boolean) -> Integer
        if c
          return 1
          2
        end
        3
      end
      """
    When I evaluate the input
    Then the evaluation should warn with "Unreachable code in f: statements after return never run"
    And the evaluation should give 1 warning

  Scenario: A statement after a return in an else arm
    Given the multiline input
      """
      def f(c: Boolean) -> Integer
        if c
          1
        else
          return 2
          3
        end
      end
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  Scenario: A statement after a return in a brace block
    Given the multiline input
      """
      def f() -> Integer
        { return 1; 2 }
        3
      end
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  Scenario: A statement after a return in a brace arm
    Given the multiline input
      """
      def f(c: Boolean) -> Integer
        if c { return 1; 2 }
        3
      end
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  Scenario: Each list of statements gives its own warning
    Given the multiline input
      """
      def f(c: Boolean) -> Integer
        if c
          return 1
          2
        end
        return 3
        4
      end
      """
    When I evaluate the input
    Then the evaluation should give 2 warnings

  Scenario: A statement after an if with a return in an arm is reachable
    Given the multiline input
      """
      def f(c: Boolean) -> Integer
        if c
          return 1
        end
        2
      end
      """
    When I evaluate the input
    Then the evaluation should give no warnings

  Scenario: The check does not follow the flow of the program
    Given the multiline input
      """
      def f() -> Integer
        if true { return 1 }
        2
      end
      """
    When I evaluate the input
    Then the evaluation should give no warnings

  Scenario: The unreachable statements are not searched
    Given the multiline input
      """
      def f() -> Integer
        return 1
        { return 2; 3 }
      end
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  # When the runtime warns

  Scenario: The warning names the function
    Given the multiline input
      """
      def first() -> Integer
        1
      end
      def second() -> Integer
        return 2
        3
      end
      """
    When I evaluate the input
    Then the evaluation should warn with "Unreachable code in second: statements after return never run"
    And the evaluation should give 1 warning

  Scenario: A function that is never called still gives the warning
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
      end
      1 + 1
      """
    When I evaluate the input
    Then the evaluation should give 1 warning
    And the result should be "2"

  Scenario: Calling the function does not repeat the warning
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
      end
      f()
      f()
      """
    When I evaluate the input
    Then the evaluation should give 1 warning

  Scenario: Each overload gives its own warning
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        return n
        0
      end
      def f(s: String) -> String
        return s
        ""
      end
      """
    When I evaluate the input
    Then the evaluation should give 2 warnings

  Scenario: A warning does not stop the function from working
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
      end
      f()
      """
    When I evaluate the input
    Then the result should be "1"
    And the evaluation should give 1 warning

  Scenario: A warning does not stop the statements after the def
    Given the multiline input
      """
      def f() -> Integer
        return 1
        2
      end
      let x = f() + 1
      x
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give 1 warning

  Scenario: A def that fails gives no warning
    Given the multiline input
      """
      def f(s: String) -> Integer
        1
      end
      def f(s: String) -> Integer
        return 1
        2
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Function f already has the overload"

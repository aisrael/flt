Feature: Function visibility

  The body of a function declared with `def` sees only its parameters, its
  local bindings, global constants, global functions and type names (ADR 027).
  A global variable does not exist for the body. A default of a keyword
  parameter sees only global constants and type names.

  See docs/runtime.md#function-scope and docs/errors.md.

  # What a body sees

  Scenario: A body reads a global constant
    Given the multiline input
      """
      const STEP = 2
      def next(n: Integer) -> Integer
        n + STEP
      end
      next(1)
      """
    When I evaluate the input
    Then the result should be "3"
    And the evaluation should give no warnings

  Scenario: A body reads a constant that is declared after the function
    Given the multiline input
      """
      def next(n: Integer) -> Integer
        n + STEP
      end
      const STEP = 2
      next(1)
      """
    When I evaluate the input
    Then the result should be "3"

  Scenario: A body calls a global function
    Given the multiline input
      """
      def double(n: Integer) -> Integer
        n * 2
      end
      def quadruple(n: Integer) -> Integer
        double(double(n))
      end
      quadruple(3)
      """
    When I evaluate the input
    Then the result should be "12"

  Scenario: A body calls a native function
    Given the multiline input
      """
      def kind(n: Integer) -> Type
        typeof(n)
      end
      kind(1)
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A body reads a type name
    Given the multiline input
      """
      def kind() -> Type
        Integer
      end
      kind()
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A parameter hides a global variable of the same name
    Given the multiline input
      """
      let n = 100
      def id(n: Integer) -> Integer
        n
      end
      id(1)
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario: A function called in a block sees the same names
    Given the multiline input
      """
      const STEP = 2
      def next(n: Integer) -> Integer
        n + STEP
      end
      { let a = 1; next(a) }
      """
    When I evaluate the input
    Then the result should be "3"

  # A global variable does not exist for a body

  Scenario: A body can't read a global variable
    Given the multiline input
      """
      let base = 10
      def plus(n: Integer) -> Integer
        n + base
      end
      plus(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: base"

  Scenario: A block in a body can't read a global variable
    Given the multiline input
      """
      let base = 10
      def plus(n: Integer) -> Integer
        { n + base }
      end
      plus(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: base"

  Scenario: A body can't call a function that a global variable holds
    Given the multiline input
      """
      let g = typeof
      def kind(n: Integer) -> Type
        g(n)
      end
      kind(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Undefined function: g"

  Scenario: A variable that a body does not read is no problem
    Given the multiline input
      """
      let unused = 10
      def one() -> Integer
        1
      end
      one()
      """
    When I evaluate the input
    Then the result should be "1"

  # Assigning to the name of a global variable

  Scenario: An assignment in a body declares a local and keeps the global variable
    Given the multiline input
      """
      let counter = 0
      def write() -> Integer
        counter = 1
        counter
      end
      [write(), counter]
      """
    When I evaluate the input
    Then the result should be "[1, 0]"
    And the evaluation should give no warnings

  Scenario: A function can't count in a global variable
    Given the multiline input
      """
      let counter = 0
      def bump()
        counter = counter + 1
      end
      bump()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: counter"

  Scenario: A let in a body declares a local and keeps the global variable
    Given the multiline input
      """
      let x = 1
      def f() -> Integer
        let x = 2
        x
      end
      [f(), x]
      """
    When I evaluate the input
    Then the result should be "[2, 1]"
    And the evaluation should give no warnings

  Scenario: An assignment in a body can't change a global constant
    Given the multiline input
      """
      const LIMIT = 10
      def f()
        LIMIT = 1
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: A global variable keeps its value after a call
    Given the multiline input
      """
      let x = 1
      def f() -> Integer
        x = 5
        x
      end
      f()
      f()
      x
      """
    When I evaluate the input
    Then the result should be "1"

  # Defaults of keyword parameters

  Scenario: A default can use a global constant
    Given the multiline input
      """
      const WAIT = 5
      def fetch(timeout: Integer = WAIT) -> Integer
        timeout
      end
      fetch()
      """
    When I evaluate the input
    Then the result should be "5"

  Scenario: A default can use a type name
    Given the multiline input
      """
      def kind(t: Type = Integer) -> Type
        t
      end
      kind()
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A default can't use a global variable
    Given the multiline input
      """
      let base = 1
      def plus(n: Integer, by: Integer = base) -> Integer
        n + by
      end
      plus(10)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: base"

  Scenario: A default can't call a native function
    Given the multiline input
      """
      def kind(t: Type = typeof(1)) -> Type
        t
      end
      kind()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Undefined function: typeof"

  Scenario: A default can't call a function declared with def
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      def plus(n: Integer, by: Integer = one()) -> Integer
        n + by
      end
      plus(10)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Undefined function: one"

  Scenario: A default that the call does not use raises no error
    Given the multiline input
      """
      let base = 1
      def plus(n: Integer, by: Integer = base) -> Integer
        n + by
      end
      plus(10, by: 5)
      """
    When I evaluate the input
    Then the result should be "15"

  Scenario: A default can't use another parameter
    Given the multiline input
      """
      def f(a: Integer, b: Integer = a) -> Integer
        b
      end
      f(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: a"

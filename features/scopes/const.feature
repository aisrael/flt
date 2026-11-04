Feature: Constants

  `const` declares a constant: a binding that the program does not intend to
  change (ADR 025). It has the form of `let`, with an optional type. In the
  reference runtime (ADR 026):

  - Binding a name that is a constant in the same scope raises a
    `ConstantRebound` error. The runtime checks the name before it evaluates
    the expression.
  - A `let` or a `const` in an inner scope that hides a constant gives a
    `ConstantHidden` warning. A parameter that hides one gives none.
  - The expression is evaluated once, when the statement runs.

  See docs/runtime.md#constants and docs/errors.md.

  # Declaring

  Scenario: A constant without a type
    Given the multiline input
      """
      const DEFAULT = "default"
      DEFAULT
      """
    When I evaluate the input
    Then the result should be '"default"'

  Scenario: A const evaluates to the bound value
    Given the input "const LIMIT = 10"
    When I evaluate the input
    Then the result should be "10"

  Scenario: A constant has the type of its value
    Given the multiline input
      """
      const LIMIT = 10
      typeof(LIMIT)
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A constant with a type
    Given the multiline input
      """
      const LIMIT: Integer = 10
      LIMIT
      """
    When I evaluate the input
    Then the result should be "10"

  Scenario: A constant with a type converts the value
    Given the multiline input
      """
      const RATE: Number = 2
      RATE
      """
    When I evaluate the input
    Then the result should be "2.0"

  Scenario: A constant with a union type
    Given the multiline input
      """
      const MAYBE: Number | None = None
      MAYBE
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: A constant with the wrong type
    Given the input "const I: Integer = 1.0"
    When I evaluate the input
    Then the evaluation should fail with "Variable I expects Integer, found Number"

  Scenario: A constant with an unknown type
    Given the input "const T: Foo = 1"
    When I evaluate the input
    Then the evaluation should fail with "Unknown type: Foo"

  Scenario: The name is not a constant name by rule
    Given the multiline input
      """
      const limit = 10
      limit
      """
    When I evaluate the input
    Then the result should be "10"
    And the evaluation should give no warnings

  Scenario: The expression is evaluated once
    Given the multiline input
      """
      const A = 1 + 2
      A + A
      """
    When I evaluate the input
    Then the result should be "6"

  Scenario: A constant is used in an expression
    Given the multiline input
      """
      const BASE = 10
      BASE * 2 + 1
      """
    When I evaluate the input
    Then the result should be "21"

  Scenario: A variable that copies a constant is a variable
    Given the multiline input
      """
      const A = 1
      let b = A
      b = 2
      b
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

  # Names

  Scenario: A keyword is not a name
    Given the input "const if = 1"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: const is a keyword
    Given the input "let const = 1"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: const needs a value
    Given the input "const A"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A constant cannot use a built-in type name
    Given the input "const Number = 5"
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind Number: it's a built-in type"

  Scenario: A constant cannot use the name of a function
    Given the input "const typeof = 1"
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind typeof: it's a function"

  # Rebinding

  Scenario: A let rebinds a constant
    Given the multiline input
      """
      const LIMIT = 10
      let LIMIT = 11
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: An assignment rebinds a constant
    Given the multiline input
      """
      const LIMIT = 10
      LIMIT = 11
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: A second const rebinds a constant
    Given the multiline input
      """
      const LIMIT = 10
      const LIMIT = 11
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: A typed let rebinds a constant
    Given the multiline input
      """
      const LIMIT = 10
      let LIMIT: Integer = 11
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: The value of a rebind is not evaluated
    Given the multiline input
      """
      const A = 1
      const A = unbound_function()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant A"

  Scenario: A constant may be declared again after a variable
    Given the multiline input
      """
      let a = 1
      const a = 2
      a
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

  Scenario: A variable is not a constant
    Given the multiline input
      """
      let a = 1
      a = 2
      let a = 3
      a
      """
    When I evaluate the input
    Then the result should be "3"
    And the evaluation should give no warnings

  # Scopes

  Scenario: A let in a block hides a constant, with a warning
    Given the multiline input
      """
      const LIMIT = 10
      {
        let LIMIT = 1
        LIMIT + 1
      }
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should warn with "Constant LIMIT is hidden by a new binding"
    And the evaluation should give 1 warning

  Scenario: The constant is back after the block
    Given the multiline input
      """
      const LIMIT = 10
      { let LIMIT = 1 }
      LIMIT
      """
    When I evaluate the input
    Then the result should be "10"

  Scenario: A const in a block hides a constant, with a warning
    Given the multiline input
      """
      const LIMIT = 10
      {
        const LIMIT = 1
        LIMIT
      }
      """
    When I evaluate the input
    Then the result should be "1"
    And the evaluation should warn with "Constant LIMIT is hidden by a new binding"

  Scenario: A let in a function body hides a global constant, with a warning
    Given the multiline input
      """
      const LIMIT = 10
      def f() -> Integer
        let LIMIT = 1
        LIMIT
      end
      f()
      """
    When I evaluate the input
    Then the result should be "1"
    And the evaluation should warn with "Constant LIMIT is hidden by a new binding"

  Scenario: A let in a block inside a block hides a constant of the outer block
    Given the multiline input
      """
      {
        const A = 1
        { let A = 2 }
        A
      }
      """
    When I evaluate the input
    Then the result should be "1"
    And the evaluation should warn with "Constant A is hidden by a new binding"

  Scenario: An assignment in a block reaches the constant
    Given the multiline input
      """
      const LIMIT = 10
      { LIMIT = 2 }
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Cannot rebind constant LIMIT"

  Scenario: An assignment after a hiding let changes the block variable
    Given the multiline input
      """
      const LIMIT = 10
      {
        let LIMIT = 1
        LIMIT = 2
        LIMIT
      }
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give 1 warning

  Scenario: A hidden constant is unchanged
    Given the multiline input
      """
      const LIMIT = 10
      { let LIMIT = 1 }
      LIMIT
      """
    When I evaluate the input
    Then the result should be "10"
    And the evaluation should give 1 warning

  Scenario: A failed hiding declaration gives no warning
    Given the multiline input
      """
      const LIMIT = 10
      { let LIMIT: Integer = "x" }
      """
    When I evaluate the input
    Then the evaluation should fail with "Variable LIMIT expects Integer, found String"

  Scenario: A parameter hides a global constant with no warning
    Given the multiline input
      """
      const LIMIT = 10
      def f(LIMIT: Integer) -> Integer
        LIMIT + 1
      end
      f(1)
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

  Scenario: A function reads a global constant
    Given the multiline input
      """
      const LIMIT = 10
      def f(n: Integer) -> Integer
        n + LIMIT
      end
      f(1)
      """
    When I evaluate the input
    Then the result should be "11"
    And the evaluation should give no warnings

  Scenario: A function cannot assign a global constant
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

  Scenario: A constant in a function body ends with the call
    Given the multiline input
      """
      def f() -> Integer
        const A = 1
        A
      end
      f() + f()
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

  Scenario: A constant in a block is gone after the block
    Given the multiline input
      """
      { const A = 1 }
      A
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: A"

  Scenario: A constant in a block can be declared again after the block
    Given the multiline input
      """
      { const A = 1 }
      const A = 2
      A
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

  Scenario: A constant in an if arm is gone after the if
    Given the multiline input
      """
      if true { const A = 1 }
      let A = 2
      A
      """
    When I evaluate the input
    Then the result should be "2"
    And the evaluation should give no warnings

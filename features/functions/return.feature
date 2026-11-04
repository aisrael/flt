Feature: The return statement

  `return expr` stops the call of the enclosing function and gives `expr`.
  `return` alone gives `None`. A `return` as the last statement is optional.
  See docs/functions-and-pipe.md#returning-early and
  docs/adrs/023-return-statement.md.

  Scenario: A return as the last statement
    Given the multiline input
      """
      def foo() -> Boolean
        return true
      end
      foo()
      """
    When I evaluate the input
    Then the result should be "true"

  Scenario: A return as the last statement gives the same value as the expression alone
    Given the multiline input
      """
      def with_return() -> Integer
        return 1 + 2
      end
      def without_return() -> Integer
        1 + 2
      end
      [with_return(), without_return()]
      """
    When I evaluate the input
    Then the result should be "[3, 3]"

  Scenario: An early return from an expanded if
    Given the multiline input
      """
      def check(x: Integer) -> String
        if x // 2 * 2 == x
          return "even"
        end

        "odd"
      end
      [check(4), check(3)]
      """
    When I evaluate the input
    Then the result should be '["even", "odd"]'

  Scenario: An early return from a brace arm
    Given the multiline input
      """
      def check(x: Integer) -> String
        if x < 0 { return "negative" }
        "not negative"
      end
      [check(-1), check(1)]
      """
    When I evaluate the input
    Then the result should be '["negative", "not negative"]'

  Scenario: An early return in a recursive function
    Given the multiline input
      """
      def check(x: Integer) -> String
        if x // 2 * 2 == x
          return "even"
        end

        check(x * 3 + 1)
      end
      check(3)
      """
    When I evaluate the input
    Then the result should be '"even"'

  Scenario: A return from nested blocks leaves the function
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        if n > 0
          if n > 10
            return 100
          end
          return 10
        end
        0
      end
      [f(20), f(5), f(-1)]
      """
    When I evaluate the input
    Then the result should be "[100, 10, 0]"

  Scenario: A return from elsif arms
    Given the multiline input
      """
      def size(n: Integer) -> Symbol
        if n < 10
          return :small
        elsif n < 100
          return :medium
        end
        :large
      end
      [size(1), size(50), size(500)]
      """
    When I evaluate the input
    Then the result should be "[:small, :medium, :large]"

  Scenario: A bare return gives None
    Given the multiline input
      """
      def f()
        return
      end
      f()
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: A bare return before end, a semicolon and a brace
    Given the multiline input
      """
      def a()
        return; 1
      end
      def b()
        { return }
        1
      end
      [a(), b()]
      """
    When I evaluate the input
    Then the result should be "[None, None]"

  Scenario: A bare return in a function that returns an Integer raises an error
    Given the multiline input
      """
      def f() -> Integer
        return
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f returned None, expected Integer"

  Scenario: A return of the wrong type raises ReturnTypeMismatch
    Given the multiline input
      """
      def f() -> Integer
        return "one"
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f returned String, expected Integer"

  Scenario: A function without a return type can only return None
    Given the multiline input
      """
      def f()
        return 1
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f returned Integer, expected None"

  Scenario: A return converts an Integer for a Number return type
    Given the multiline input
      """
      def f() -> Number
        return 1
      end
      f()
      """
    When I evaluate the input
    Then the result should be "1.0"

  Scenario: A return with a pipe
    Given the multiline input
      """
      def double(n: Integer) -> Integer
        n * 2
      end
      def f() -> Integer
        return 3 |> double
      end
      f()
      """
    When I evaluate the input
    Then the result should be "6"

  Scenario: The statements after a return never run
    Given the multiline input
      """
      def f() -> Integer
        return 1
        1 / 0
        2
      end
      f()
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario: A return stops the rest of the expression that holds it
    Given the multiline input
      """
      def f() -> Integer
        let x = 1 + if true { return 5 } else { 2 }
        7
      end
      f()
      """
    When I evaluate the input
    Then the result should be "5"

  Scenario: A return in a call that is part of an expression
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        if n > 1
          return n
        end
        0
      end
      1 + f(2) + f(0)
      """
    When I evaluate the input
    Then the result should be "3"

  Scenario: A return only leaves the call that runs it
    Given the multiline input
      """
      def inner() -> Integer
        return 1
      end
      def outer() -> Integer
        let n = inner()
        n + 1
      end
      outer()
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: A return closes the scopes that the call opened
    Given the multiline input
      """
      def f() -> Integer
        let x = 1
        { let x = 2; return x }
        3
      end
      let results = [f(), f()]
      results
      """
    When I evaluate the input
    Then the result should be "[2, 2]"

  Scenario: A return does not change the type of the function
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        return n
      end
      typeof(f)
      """
    When I evaluate the input
    Then the result should be "fn(Integer) -> Integer"

  Scenario: A return in an overload
    Given the multiline input
      """
      def f(n: Integer) -> String
        return "integer"
      end
      def f(s: String) -> String
        return "string"
      end
      [f(1), f("a")]
      """
    When I evaluate the input
    Then the result should be '["integer", "string"]'

  Scenario: A later call works after an early return
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        if n == 0
          return 0
        end
        n
      end
      [f(0), f(2), f(0)]
      """
    When I evaluate the input
    Then the result should be "[0, 2, 0]"

  # Where return is not allowed

  Scenario: A return at the top level is a parse error
    Given the input "return 1"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A bare return at the top level is a parse error
    Given the input "return"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return in a top-level block is a parse error
    Given the input "{ return 1 }"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return in a top-level expanded if is a parse error
    Given the multiline input
      """
      if true
        return 1
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return inside an operand at the top level is a parse error
    Given the input "1 + { return 2 }"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return in an argument of a call at the top level is a parse error
    Given the input "typeof({ return 1 })"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return in the default of a keyword parameter is a parse error
    Given the multiline input
      """
      def f(k: Integer = { return 1 }) -> Integer
        k
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return in a let at the top level is a parse error
    Given the input "let x = { return 1 }"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A parse error means that no statement runs
    Given the multiline input
      """
      let x = 1
      return x
      """
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

  Scenario: A return is not an identifier or a function name
    Given the input 'def return() 1 end'
    When I parse the input
    Then parsing should fail

  # Parsing

  Scenario: Parsing return with an expression
    Given the input "return 1"
    When I parse the input
    Then the statement should print as "return 1"

  Scenario: Parsing a bare return
    Given the input "return"
    When I parse the input
    Then the statement should print as "return"

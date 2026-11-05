Feature: const functions

  `const def` declares a function that is meant to be pure: the same arguments
  give the same result. In the reference runtime, the body of a `const def`
  and a default of a keyword parameter can only call `const` functions. All
  overloads of a function are `const`, or none is. See
  docs/functions-and-pipe.md#const-functions, docs/runtime.md#const-functions
  and docs/adrs/028-const-functions.md, docs/adrs/029-const-functions-in-the-reference-runtime.md.

  # Declaring and calling

  Scenario: A const def gives None
    Given the multiline input
      """
      const def one() -> Integer
        1
      end
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: Calling a const function
    Given the multiline input
      """
      const def area(w: Integer, h: Integer) -> Integer
        w * h
      end
      area(3, 4)
      """
    When I evaluate the input
    Then the result should be "12"

  Scenario: A const function can have keywords, a variadic parameter and a return
    Given the multiline input
      """
      const def sign(n: Integer, *rest: Any, zero: Integer = 0) -> Integer
        if n < 0
          return -1
        end
        if n == 0
          return zero
        end
        1
      end
      [sign(-5), sign(0), sign(0, zero: 7), sign(9)]
      """
    When I evaluate the input
    Then the result should be "[-1, 0, 7, 1]"

  Scenario: A const function can have a block and a local binding
    Given the multiline input
      """
      const def f(n: Integer) -> Integer
        let a = n + 1
        { let b = a * 2; b }
      end
      f(1)
      """
    When I evaluate the input
    Then the result should be "4"

  Scenario: A const function can call itself
    Given the multiline input
      """
      const def fact(n: Integer) -> Integer
        if n <= 1 1 else n * fact(n - 1)
      end
      fact(5)
      """
    When I evaluate the input
    Then the result should be "120"

  Scenario: A const function can be called in the whitespace form and in a pipe
    Given the multiline input
      """
      const def double(n: Integer) -> Integer
        n * 2
      end
      let a = double(2)
      let b = double 3
      let c = 4 |> double
      [a, b, c]
      """
    When I evaluate the input
    Then the result should be "[4, 6, 8]"

  Scenario: A const function can only be declared at the top level
    Given the multiline input
      """
      { const def f() -> Integer
        1
      end }
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot define f here"

  # What a const function can call

  Scenario: A const function calls a const function
    Given the multiline input
      """
      const def area(w: Integer, h: Integer) -> Integer
        w * h
      end
      const def double_area(w: Integer, h: Integer) -> Integer
        area(w, h) * 2
      end
      double_area(3, 4)
      """
    When I evaluate the input
    Then the result should be "24"

  Scenario: A const function calls typeof
    Given the multiline input
      """
      const def kind(n: Integer) -> Type
        typeof(n)
      end
      kind(1)
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A const function calls a native function that is const
    Given the native function "square(n: Integer) -> Integer"
    And the multiline input
      """
      const def area(n: Integer) -> Integer
        square(n)
      end
      area(3)
      """
    When I evaluate the input
    Then the result should be "9"

  Scenario: A const function can't call a function declared with def
    Given the multiline input
      """
      def stamp() -> Integer
        1
      end
      const def bad() -> Integer
        stamp()
      end
      bad()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function stamp is not const and cannot be called here"

  Scenario: A const function can't call a native function that is not const
    Given the native function "identity(v: Any) -> Any"
    And the multiline input
      """
      const def bad(n: Integer) -> Any
        identity(n)
      end
      bad(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function identity is not const and cannot be called here"

  Scenario: A call in a pipe is a call
    Given the native function "identity(v: Any) -> Any"
    And the multiline input
      """
      const def bad(n: Integer) -> Any
        n |> identity
      end
      bad(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function identity is not const and cannot be called here"

  Scenario: A call in a block of a const function is checked
    Given the multiline input
      """
      def stamp() -> Integer
        1
      end
      const def bad() -> Integer
        { stamp() }
      end
      bad()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function stamp is not const and cannot be called here"

  Scenario: A function that a const function calls is const too
    Given the multiline input
      """
      def stamp() -> Integer
        1
      end
      const def inner() -> Integer
        stamp()
      end
      const def outer() -> Integer
        inner()
      end
      outer()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function stamp is not const and cannot be called here"

  Scenario: A const function reads a global constant
    Given the multiline input
      """
      const STEP = 2
      const def next(n: Integer) -> Integer
        n + STEP
      end
      next(1)
      """
    When I evaluate the input
    Then the result should be "3"

  Scenario: A const function can't read a global variable
    Given the multiline input
      """
      let base = 10
      const def plus(n: Integer) -> Integer
        n + base
      end
      plus(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Unbound identifier: base"

  Scenario: The check comes when the call runs
    Given the multiline input
      """
      def stamp() -> Integer
        1
      end
      const def pick(flag: Boolean) -> Integer
        if flag stamp() else 0
      end
      pick(false)
      """
    When I evaluate the input
    Then the result should be "0"

  Scenario: A const function can call a function that is declared later
    Given the multiline input
      """
      const def outer(n: Integer) -> Integer
        inner(n)
      end
      const def inner(n: Integer) -> Integer
        n + 1
      end
      outer(1)
      """
    When I evaluate the input
    Then the result should be "2"

  # Callers

  Scenario: A def function calls a const function
    Given the multiline input
      """
      const def area(w: Integer, h: Integer) -> Integer
        w * h
      end
      def report(w: Integer, h: Integer) -> Integer
        area(w, h)
      end
      report(2, 3)
      """
    When I evaluate the input
    Then the result should be "6"

  Scenario: A def function that calls a const function can still call other functions
    Given the native function "identity(v: Any) -> Any"
    And the multiline input
      """
      const def area(w: Integer, h: Integer) -> Integer
        w * h
      end
      def report(w: Integer, h: Integer) -> Any
        identity(area(w, h))
      end
      report(2, 3)
      """
    When I evaluate the input
    Then the result should be "6"

  # Defaults

  Scenario: A default can call a const function
    Given the multiline input
      """
      const def seconds(m: Integer) -> Integer
        m * 60
      end
      def wait(timeout: Integer = seconds(2)) -> Integer
        timeout
      end
      wait()
      """
    When I evaluate the input
    Then the result should be "120"

  Scenario: A default of a const function can call a const function
    Given the multiline input
      """
      const def seconds(m: Integer) -> Integer
        m * 60
      end
      const def wait(timeout: Integer = seconds(2)) -> Integer
        timeout
      end
      wait()
      """
    When I evaluate the input
    Then the result should be "120"

  Scenario: A default can call typeof
    Given the multiline input
      """
      def kind(t: Type = typeof(1)) -> Type
        t
      end
      kind()
      """
    When I evaluate the input
    Then the result should be "Integer"

  Scenario: A default can't call a function that is not const
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
    Then the evaluation should fail with exactly "Runtime Error: Function one is not const and cannot be called here"

  Scenario: A default can't call a native function that is not const
    Given the native function "identity(v: Any) -> Any"
    And the multiline input
      """
      def f(v: Any = identity(1)) -> Any
        v
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function identity is not const and cannot be called here"

  Scenario: A default that the call gives raises no error
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      def plus(n: Integer, by: Integer = one()) -> Integer
        n + by
      end
      plus(10, by: 5)
      """
    When I evaluate the input
    Then the result should be "15"

  # Overloads

  Scenario: A const function can have several overloads
    Given the multiline input
      """
      const def show(n: Integer) -> String
        "integer"
      end
      const def show(s: String) -> String
        "string"
      end
      [show(1), show("a")]
      """
    When I evaluate the input
    Then the result should be '["integer", "string"]'

  Scenario: A def can't add an overload to a const function
    Given the multiline input
      """
      const def f(n: Integer) -> Integer
        n
      end
      def f(s: String) -> String
        s
      end
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f is const, so all of its overloads must be const"

  Scenario: A const def can't add an overload to a function declared with def
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        n
      end
      const def f(s: String) -> String
        s
      end
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f is not const, so none of its overloads can be const"

  Scenario: The const check comes before the duplicate check
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        n
      end
      const def f(n: Integer) -> Integer
        n
      end
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function f is not const, so none of its overloads can be const"

  Scenario: Two const overloads can't have the same parameters
    Given the multiline input
      """
      const def f(n: Integer) -> Integer
        n
      end
      const def f(m: Integer) -> String
        "x"
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "already has the overload"

  # Types

  Scenario: The type of a const function shows const
    Given the multiline input
      """
      const def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      typeof(mod)
      """
    When I evaluate the input
    Then the result should be "const fn(Integer, Integer) -> Integer"

  Scenario: The type of a function with several const overloads shows const on each
    Given the multiline input
      """
      const def show(n: Integer) -> String
        "integer"
      end
      const def show(s: String) -> String
        "string"
      end
      typeof(show)
      """
    When I evaluate the input
    Then the result should be "const fn(Integer) -> String | const fn(String) -> String"

  Scenario: The type of a def function does not show const
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      typeof(mod)
      """
    When I evaluate the input
    Then the result should be "fn(Integer, Integer) -> Integer"

  Scenario: The type of a native const function shows const
    Given the native function "square(n: Integer) -> Integer"
    And the input 'typeof(square)'
    When I evaluate the input
    Then the result should be "const fn(Integer) -> Integer"

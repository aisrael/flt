Feature: Pattern matching in function parameters

  A positional parameter can be a literal, such as `def foo(0)`. The parameter
  accepts only that value. A pattern matches by type and by value, so `def
  foo(2)` accepts the `Integer` 2 and not the `Number` 2.0. A pattern is the
  narrowest overload for its value, so `SimpleRuntime` runs it whatever the
  order of the `def`s. Only a literal is a pattern. See
  docs/functions-and-pipe.md#pattern-matching and
  docs/adrs/030-pattern-matching.md.

  # Choosing an overload

  Scenario: A pattern runs for its value and the typed name for the others
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      def foo(i: Integer) -> Integer
        i * 2
      end
      [foo(0), foo(3)]
      """
    When I evaluate the input
    Then the result should be "[1, 6]"

  Scenario: The order of the defs does not matter
    Given the multiline input
      """
      def foo(i: Integer) -> Integer
        i * 2
      end
      def foo(0) -> Integer
        1
      end
      [foo(0), foo(3)]
      """
    When I evaluate the input
    Then the result should be "[1, 6]"

  Scenario: A pattern alone accepts only its value
    Given the multiline input
      """
      def zero(0) -> String
        "zero"
      end
      zero(0)
      """
    When I evaluate the input
    Then the result should be '"zero"'

  Scenario: A pattern is narrower than Integer, Number and Any
    Given the multiline input
      """
      def f(n: Number) -> String
        "number"
      end
      def f(i: Integer) -> String
        "integer"
      end
      def f(0) -> String
        "zero"
      end
      def g(x: Any) -> String
        "any"
      end
      def g(0) -> String
        "zero"
      end
      [f(0), f(1), f(1.0), g(0), g("a")]
      """
    When I evaluate the input
    Then the result should be '["zero", "integer", "number", "zero", "any"]'

  Scenario: A pattern is narrower than a union
    Given the multiline input
      """
      def f(x: Integer | String) -> String
        "union"
      end
      def f(0) -> String
        "zero"
      end
      [f(0), f(1), f("a")]
      """
    When I evaluate the input
    Then the result should be '["zero", "union", "union"]'

  # Matching is by type and by value

  Scenario: An integer pattern does not accept a number
    Given the multiline input
      """
      def foo(2) -> String
        "two"
      end
      foo(2.0)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 1 of foo expects 2, found 2.0"

  Scenario: A number pattern does not accept an integer
    Given the multiline input
      """
      def foo(2.0) -> String
        "two"
      end
      foo(2)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 1 of foo expects 2.0, found 2"

  Scenario: An integer pattern and a number pattern are different overloads
    Given the multiline input
      """
      def foo(2) -> String
        "integer"
      end
      def foo(2.0) -> String
        "number"
      end
      [foo(2), foo(2.0)]
      """
    When I evaluate the input
    Then the result should be '["integer", "number"]'

  Scenario: A string pattern
    Given the multiline input
      """
      def greet("en") -> String
        "Hello"
      end
      def greet("fr") -> String
        "Bonjour"
      end
      def greet(code: String) -> String
        "Hi"
      end
      [greet("en"), greet("fr"), greet("de")]
      """
    When I evaluate the input
    Then the result should be '["Hello", "Bonjour", "Hi"]'

  Scenario: A string pattern can hold an escaped brace
    Given the multiline input
      """
      def f("\{") -> String
        "brace"
      end
      f("\{")
      """
    When I evaluate the input
    Then the result should be '"brace"'

  Scenario: A boolean pattern
    Given the multiline input
      """
      def yes(true) -> String
        "yes"
      end
      def yes(false) -> String
        "no"
      end
      [yes(true), yes(false)]
      """
    When I evaluate the input
    Then the result should be '["yes", "no"]'

  Scenario: A None pattern
    Given the multiline input
      """
      def show(None) -> String
        "nothing"
      end
      def show(x: Any) -> String
        "something"
      end
      [show(None), show(1)]
      """
    When I evaluate the input
    Then the result should be '["nothing", "something"]'

  Scenario: A symbol pattern
    Given the multiline input
      """
      def mode(:fast) -> Integer
        1
      end
      def mode(:slow) -> Integer
        2
      end
      [mode(:fast), mode(:slow)]
      """
    When I evaluate the input
    Then the result should be "[1, 2]"

  Scenario: A negative integer pattern
    Given the multiline input
      """
      def sign(-1) -> String
        "minus one"
      end
      def sign(n: Integer) -> String
        "other"
      end
      [sign(-1), sign(1), sign(0), sign(-2)]
      """
    When I evaluate the input
    Then the result should be '["minus one", "other", "other", "other"]'

  Scenario: A negative number pattern
    Given the multiline input
      """
      def f(-2.5) -> String
        "minus two and a half"
      end
      def f(n: Number) -> String
        "other"
      end
      [f(-2.5), f(2.5), f(-2.0)]
      """
    When I evaluate the input
    Then the result should be '["minus two and a half", "other", "other"]'

  Scenario: A plus sign is part of the number
    Given the multiline input
      """
      def foo(+1) -> String
        "one"
      end
      def foo(+2.5) -> String
        "two and a half"
      end
      [foo(1), foo(2.5), typeof(foo)]
      """
    When I evaluate the input
    Then the result should be '["one", "two and a half", fn(1) -> String | fn(2.5) -> String]'

  Scenario: A pattern with a plus sign is the same as one without
    Given the multiline input
      """
      def foo(1) -> Integer
        1
      end
      def foo(+1) -> Integer
        2
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Function foo already has the overload"

  Scenario: A negative integer pattern does not accept a number
    Given the multiline input
      """
      def foo(-1) -> String
        "minus one"
      end
      foo(-1.0)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 1 of foo expects -1, found -1.0"

  Scenario: A negative integer pattern does not accept its opposite
    Given the multiline input
      """
      def foo(-1) -> String
        "minus one"
      end
      foo(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 1 of foo expects -1, found 1"

  Scenario: A negative integer pattern and a positive one are different overloads
    Given the multiline input
      """
      def foo(-1) -> Integer
        1
      end
      def foo(1) -> Integer
        2
      end
      [foo(-1), foo(1)]
      """
    When I evaluate the input
    Then the result should be "[1, 2]"

  Scenario: A negative pattern can follow another pattern
    Given the multiline input
      """
      def f(0, -1) -> String
        "zero, minus one"
      end
      def f(a: Integer, b: Integer) -> String
        "other"
      end
      [f(0, -1), f(-1, 0)]
      """
    When I evaluate the input
    Then the result should be '["zero, minus one", "other"]'

  Scenario: The type of a function shows a negative pattern
    Given the multiline input
      """
      def foo(-1) -> Integer
        1
      end
      def bar(-2.5) -> Integer
        1
      end
      [typeof(foo), typeof(bar)]
      """
    When I evaluate the input
    Then the result should be '[fn(-1) -> Integer, fn(-2.5) -> Integer]'

  # Several parameters

  Scenario: A recursive function ends on its patterns
    Given the multiline input
      """
      def fib(0) -> Integer
        0
      end
      def fib(1) -> Integer
        1
      end
      def fib(n: Integer) -> Integer
        fib(n - 1) + fib(n - 2)
      end
      fib(10)
      """
    When I evaluate the input
    Then the result should be "55"

  Scenario: Patterns in more than one parameter
    Given the multiline input
      """
      def add(0, 0) -> String
        "both zero"
      end
      def add(0, y: Integer) -> String
        "first zero"
      end
      def add(x: Integer, y: Integer) -> String
        "none"
      end
      [add(0, 0), add(0, 5), add(5, 0)]
      """
    When I evaluate the input
    Then the result should be '["both zero", "first zero", "none"]'

  Scenario: A pattern with a keyword parameter
    Given the multiline input
      """
      def scale(0, factor: Integer = 1) -> Integer
        factor
      end
      def scale(n: Integer, factor: Integer = 1) -> Integer
        n * factor
      end
      [scale(0, factor: 7), scale(2, factor: 7), scale(2)]
      """
    When I evaluate the input
    Then the result should be "[7, 14, 2]"

  Scenario: A pattern with a variadic parameter
    Given the multiline input
      """
      def f(0, *rest: Any) -> Array
        rest
      end
      def f(n: Integer, *rest: Any) -> Array
        []
      end
      [f(0, 1, 2), f(5, 1, 2)]
      """
    When I evaluate the input
    Then the result should be "[[1, 2], []]"

  # Declarations

  Scenario: A pattern is part of the signature
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      def foo(0) -> Integer
        2
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Function foo already has the overload"

  Scenario: A pattern and a typed name are different overloads
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      def foo(i: Integer) -> Integer
        2
      end
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: A const def can have patterns
    Given the multiline input
      """
      const def fact(0) -> Integer
        1
      end
      const def fact(n: Integer) -> Integer
        n * fact(n - 1)
      end
      fact(5)
      """
    When I evaluate the input
    Then the result should be "120"

  Scenario: A const def with a pattern can't be mixed with a def
    Given the multiline input
      """
      const def f(0) -> Integer
        1
      end
      def f(n: Integer) -> Integer
        n
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Function f is const, so all of its overloads must be const"

  Scenario: A def with a pattern can't be mixed with a const def
    Given the multiline input
      """
      def f(0) -> Integer
        1
      end
      const def f(n: Integer) -> Integer
        n
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "Function f is not const, so none of its overloads can be const"

  # Types

  Scenario: The type of a function shows its pattern
    Given the multiline input
      """
      def foo(i: Integer) -> Integer
        i * 2
      end
      def foo(0) -> Integer
        1
      end
      typeof(foo)
      """
    When I evaluate the input
    Then the result should be "fn(Integer) -> Integer | fn(0) -> Integer"

  Scenario: The type of a pattern shows each kind of literal
    Given the multiline input
      """
      def a(2.5) -> None
      end
      def b("x") -> None
      end
      def c(true) -> None
      end
      def d(None) -> None
      end
      def e(:k) -> None
      end
      [typeof(a), typeof(b), typeof(c), typeof(d), typeof(e)]
      """
    When I evaluate the input
    Then the result should be '[fn(2.5) -> None, fn("x") -> None, fn(true) -> None, fn(None) -> None, fn(:k) -> None]'

  Scenario: A pattern shows next to a typed name
    Given the multiline input
      """
      def f(0, s: String, k: Integer = 1) -> None
      end
      typeof(f)
      """
    When I evaluate the input
    Then the result should be "fn(0, String, k: Integer) -> None"

  # Errors

  Scenario: No overload accepts the value
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      foo(1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 1 of foo expects 0, found 1"

  Scenario: No overload accepts the type
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      foo("a")
      """
    When I evaluate the input
    Then the evaluation should fail with exactly 'Runtime Error: Argument 1 of foo expects 0, found "a"'

  Scenario: The error names the position of the pattern
    Given the multiline input
      """
      def foo(s: String, 0) -> Integer
        1
      end
      foo("a", 1)
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument 2 of foo expects 0, found 1"

  Scenario: The closest overload gives the error
    Given the multiline input
      """
      def foo(0, s: String) -> Integer
        1
      end
      def foo(n: Integer, s: Integer) -> Integer
        2
      end
      foo(1, "x")
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Argument s of foo expects Integer, found String"

  Scenario: Overloads that are equally close give no overload accepts
    Given the multiline input
      """
      def foo(0, a: String) -> Integer
        1
      end
      def foo(1, a: String) -> Integer
        2
      end
      foo(2, "x")
      """
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: No overload of foo accepts (Integer, String)"

  Scenario: A call with the wrong number of arguments is unchanged
    Given the multiline input
      """
      def foo(0) -> Integer
        1
      end
      foo()
      """
    When I evaluate the input
    Then the evaluation should fail with "Function foo expected 1 argument(s), found 0"

  # Only a literal is a pattern

  Scenario: A name without a type is not a pattern
    Given the input 'def foo(x) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A const name is not a pattern
    Given the input 'def foo(MAX) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A call is not a pattern
    Given the input 'def foo(twice(DEFAULT)) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: An expression is not a pattern
    Given the input 'def foo(1 + 1) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A string pattern has no interpolation
    Given the input 'def foo("a{x}") -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A pattern can't be a variadic parameter
    Given the input 'def foo(*0) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A pattern can't be a keyword parameter
    Given the input 'def foo(k: 0 = 0) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A pattern can't be a rest keywords parameter
    Given the input 'def foo(**0) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A pattern can't follow a keyword parameter
    Given the input 'def foo(k: Integer = 1, 0) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A minus sign is only for a number
    Given the input 'def foo(-"a") -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A minus sign is not for a name
    Given the input 'def foo(-x) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: Only one minus sign
    Given the input 'def foo(--1) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A parenthesized number is not a pattern
    Given the input 'def foo(-(1)) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: The minus sign touches the number
    Given the input 'def foo(- 1) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A pattern can't have a type
    Given the input 'def foo(0: Integer) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

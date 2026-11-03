Feature: Declaring functions with def

  flt source can declare a function with `def name(params) [-> type] ... end`.
  The parameters come in this order: positional, variadic, keyword, rest
  keywords. Every parameter has a type. A call has its own scope. A second
  `def` with the same name adds an overload. See
  docs/functions-and-pipe.md#declaring-functions and
  docs/adrs/019-def-function-declarations.md.

  Scenario: A def gives None
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: Calling a declared function
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      mod(7, 3)
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario: A declared function can be called in the whitespace form
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      mod 7, 3
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario: A declared function can be piped into
    Given the multiline input
      """
      def double(n: Number) -> Number
        n * 2
      end
      21 |> double
      """
    When I evaluate the input
    Then the result should be "42.0"

  Scenario: The type of a declared function
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      typeof(mod)
      """
    When I evaluate the input
    Then the result should be "fn(Integer, Integer) -> Integer"

  Scenario: A function with no parameters
    Given the multiline input
      """
      def flip_a_coin() -> Boolean
        true
      end
      flip_a_coin()
      """
    When I evaluate the input
    Then the result should be "true"

  Scenario: A declared function is a function value
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      let f = one
      f()
      """
    When I evaluate the input
    Then the result should be "1"

  # Parameters

  Scenario: The parameters can be positional, variadic and keyword
    Given the multiline input
      """
      def log(label: String, *values: Any, level: Symbol = :info)
        None
      end
      typeof(log)
      """
    When I evaluate the input
    Then the result should be "fn(String, *Any, level: Symbol) -> None"

  Scenario: A rest keywords parameter shows as **T
    Given the multiline input
      """
      def options(**opts: Any) -> Map
        opts
      end
      typeof(options)
      """
    When I evaluate the input
    Then the result should be "fn(**Any) -> Map"

  Scenario: A variadic parameter is an Array in the body
    Given the multiline input
      """
      def collect(*values: Any) -> Array
        values
      end
      collect(1, :a, "b")
      """
    When I evaluate the input
    Then the result should be '[1, :a, "b"]'

  Scenario: A variadic parameter with no values is an empty Array
    Given the multiline input
      """
      def collect(*values: Any) -> Array
        values
      end
      collect()
      """
    When I evaluate the input
    Then the result should be "[]"

  Scenario: A rest keywords parameter is a Map in the body
    Given the multiline input
      """
      def options(**opts: Any) -> Map
        opts
      end
      options(color: :red, size: 3)
      """
    When I evaluate the input
    Then the result should be "{ color: :red, size: 3 }"

  Scenario: A parameter in the wrong order is a parse error
    Given the input 'def f(k: Integer = 1, n: Integer) -> Integer n end'
    When I parse the input
    Then parsing should fail

  Scenario: A variadic parameter after a keyword parameter is a parse error
    Given the input 'def f(k: Integer = 1, *rest: Any) -> Integer k end'
    When I parse the input
    Then parsing should fail

  Scenario: Two variadic parameters are a parse error
    Given the input 'def f(*a: Any, *b: Any) -> Integer 1 end'
    When I parse the input
    Then parsing should fail

  Scenario: A parameter name can't occur twice
    Given the input 'def f(n: Integer, n: Integer) -> Integer n end'
    When I parse the input
    Then parsing should fail

  Scenario: A parameter without a type is a parse error
    Given the input 'def f(n) -> Integer n end'
    When I parse the input
    Then parsing should fail

  Scenario: A parameter after the rest keywords parameter is a parse error
    Given the input 'def f(n: Integer, **k: Any, j: Integer) -> Integer n end'
    When I parse the input
    Then parsing should fail

  Scenario: A def without a body is a parse error
    Given the input 'def f(n: Integer) -> Integer'
    When I parse the input
    Then parsing should fail

  Scenario: A reserved keyword can't be a function name
    Given the input 'def end() 1 end'
    When I parse the input
    Then parsing should fail

  # Calls are type-checked

  Scenario: An argument of the wrong type
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      mod(1, "two")
      """
    When I evaluate the input
    Then the evaluation should fail with "Argument d of mod expects Integer, found String"

  Scenario: A missing argument
    Given the multiline input
      """
      def mod(n: Integer, d: Integer) -> Integer
        n - (n // d) * d
      end
      mod(1)
      """
    When I evaluate the input
    Then the evaluation should fail with "Function mod expected 2 argument(s), found 1"

  Scenario: A Number parameter converts an Integer
    Given the multiline input
      """
      def same(n: Number) -> Number
        n
      end
      same(1)
      """
    When I evaluate the input
    Then the result should be "1.0"

  Scenario: An unexpected keyword
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      one(k: 2)
      """
    When I evaluate the input
    Then the evaluation should fail with "Function one got an unexpected keyword argument: k"

  # Keyword defaults

  Scenario: An omitted keyword takes its default
    Given the multiline input
      """
      def greet(name: String, greeting: String = "Hello") -> String
        greeting <> ", " <> name
      end
      greet("Ann")
      """
    When I evaluate the input
    Then the result should be '"Hello, Ann"'

  Scenario: A keyword argument replaces the default
    Given the multiline input
      """
      def greet(name: String, greeting: String = "Hello") -> String
        greeting <> ", " <> name
      end
      greet("Ann", greeting: "Hi")
      """
    When I evaluate the input
    Then the result should be '"Hi, Ann"'

  Scenario: A keyword of the wrong type
    Given the multiline input
      """
      def greet(name: String, greeting: String = "Hello") -> String
        greeting <> ", " <> name
      end
      greet("Ann", greeting: 1)
      """
    When I evaluate the input
    Then the evaluation should fail with "Keyword argument greeting of greet expects String, found Integer"

  Scenario: A default is evaluated for each call that omits the keyword
    Given the multiline input
      """
      let base = 1
      def plus(n: Integer, by: Integer = base) -> Integer
        n + by
      end
      let first = plus(10)
      base = 5
      [first, plus(10)]
      """
    When I evaluate the input
    Then the result should be "[11, 15]"

  Scenario: A default can't use another parameter
    Given the multiline input
      """
      def f(a: Integer, b: Integer = a) -> Integer
        b
      end
      f(1)
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: The value of a default must be of the type of the keyword
    Given the multiline input
      """
      def f(k: Integer = "one") -> Integer
        k
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with "Keyword argument k of f expects Integer, found String"

  # Return type and body

  Scenario: The function gives the value of its last statement
    Given the multiline input
      """
      def two() -> Integer
        1
        2
      end
      two()
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: An empty body gives None
    Given the multiline input
      """
      def nothing()
      end
      nothing()
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: Without a return type, the return type is None
    Given the multiline input
      """
      def nothing()
      end
      typeof(nothing)
      """
    When I evaluate the input
    Then the result should be "fn() -> None"

  Scenario: Without a return type, the body must give None
    Given the multiline input
      """
      def one()
        1
      end
      one()
      """
    When I evaluate the input
    Then the evaluation should fail with "Function one returned Integer, expected None"

  Scenario: The result must be of the return type
    Given the multiline input
      """
      def f() -> String
        1
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with "Function f returned Integer, expected String"

  Scenario: A Number result converts an Integer
    Given the multiline input
      """
      def f() -> Number
        1
      end
      f()
      """
    When I evaluate the input
    Then the result should be "1.0"

  # Scope

  Scenario: Parameters are not visible after the call
    Given the multiline input
      """
      def id(n: Integer) -> Integer
        n
      end
      id(1)
      n
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: n"

  Scenario: A let in the body is local to the call
    Given the multiline input
      """
      def f() -> Integer
        let inner = 1
        inner
      end
      f()
      inner
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: inner"

  Scenario: A body reads global variables
    Given the multiline input
      """
      let base = 10
      def plus(n: Integer) -> Integer
        n + base
      end
      plus(1)
      """
    When I evaluate the input
    Then the result should be "11"

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

  Scenario: A parameter keeps its declared type
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        n = "two"
        1
      end
      f(1)
      """
    When I evaluate the input
    Then the evaluation should fail with "Variable n expects Integer, found String"

  Scenario: An assignment in the body changes the nearest binding
    Given the multiline input
      """
      let count = 0
      def bump()
        count = count + 1
        None
      end
      bump()
      count
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario: An assignment to an unbound name in the body declares it in the call
    Given the multiline input
      """
      def f() -> Integer
        x = 1
        x
      end
      f()
      x
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: x"

  Scenario: A function can call itself
    Given the multiline input
      """
      def factorial(n: Integer) -> Integer
        if n <= 1 { 1 } else { n * factorial(n - 1) }
      end
      factorial(5)
      """
    When I evaluate the input
    Then the result should be "120"

  Scenario: Each call has its own scope
    Given the multiline input
      """
      def depth(n: Integer) -> Integer
        let here = n
        if n == 0 { 0 } else { depth(n - 1) + here }
      end
      depth(3)
      """
    When I evaluate the input
    Then the result should be "6"

  # Overloads and names

  Scenario: A second def with the same name adds an overload
    Given the multiline input
      """
      def show(x: Integer) -> String
        "integer"
      end
      def show(x: String) -> String
        "string"
      end
      [show(1), show("a")]
      """
    When I evaluate the input
    Then the result should be '["integer", "string"]'

  Scenario: The type of an overloaded function shows every overload
    Given the multiline input
      """
      def show(x: Integer) -> String
        "integer"
      end
      def show(x: String) -> String
        "string"
      end
      typeof(show)
      """
    When I evaluate the input
    Then the result should be "fn(Integer) -> String | fn(String) -> String"

  Scenario: Overloads can differ in arity
    Given the multiline input
      """
      def f() -> String
        "none"
      end
      def f(x: Any) -> String
        "one"
      end
      [f(), f(1)]
      """
    When I evaluate the input
    Then the result should be '["none", "one"]'

  Scenario: An overload with equal parameter types is a DuplicateOverload error
    Given the multiline input
      """
      def foo(s: String) -> Number
        1
      end
      def foo(s: String) -> String
        "a"
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "DuplicateOverload"

  Scenario: Overloads that differ only in a positional name are duplicates
    Given the multiline input
      """
      def foo(s: String) -> Number
        1
      end
      def foo(text: String) -> Number
        2
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "DuplicateOverload"

  Scenario: Overloads that differ only in a default are duplicates
    Given the multiline input
      """
      def foo(s: String, n: Number = 0) -> Number
        n
      end
      def foo(s: String, n: Number = 1) -> Number
        n
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "DuplicateOverload"

  Scenario: A def can't use the name of a built-in type
    Given the multiline input
      """
      def Number() -> Integer
        1
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "NameAlreadyBound"

  Scenario: A def can't use the name of a variable
    Given the multiline input
      """
      let x = 1
      def x() -> Integer
        1
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "NameAlreadyBound"

  Scenario: A def can't add an overload to a native function
    Given the native function "identity(v: Any) -> Any"
    And the multiline input
      """
      def identity(v: String) -> String
        v
      end
      """
    When I evaluate the input
    Then the evaluation should fail with "NameAlreadyBound"

  Scenario: A function's name can't be bound as a variable
    Given the multiline input
      """
      def one() -> Integer
        1
      end
      let one = 2
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind one: it's a function"

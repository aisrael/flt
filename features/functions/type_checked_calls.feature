Feature: Type-checked function calls

  Every call is checked against the function's signatures by type: its
  positional arguments, variadic values and keyword values, and the
  function's result. Overloads are tried by arity, longest first, and the
  first whose types and keywords match is called. See
  docs/functions-and-pipe.md#calls and
  docs/adrs/013-type-checked-function-calls.md.

  Background:
    Given the native function "describe(n: Number) -> String"
    And the native function "describe(s: String | Symbol) -> String"
    And the native function "pad(s: String, width: Number) -> String"
    And the native function "total(*values: Number) -> Number"
    And the native function "liar() -> Number"
    And the native function "scores(**kwargs: Number) -> Map"
    And the native function 'greet(name: String, greeting: String = "Hello") -> String'
    And the native function "date(yyyy: Number, mm: Number, dd: Number) -> Date"
    And the native function "days_between(from: Date, to: Date) -> Number"

  Scenario Outline: Arguments of the declared types pass
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                    | result  |
      | pad("ab", 4)             | "  ab"  |
      | total()                  | 0       |
      | total(1, 2, 3)           | 6       |
      | scores(a: 1, b: 2)       | {a: 1, b: 2} |
      | greet("Ann", greeting: "Hi") | "Hi, Ann" |

  Scenario Outline: An argument of the wrong type is an error
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                       | error                                                                    |
      | pad(1, 2)                   | Runtime Error: Argument s of pad expects String, found Number            |
      | pad("x", "2")               | Runtime Error: Argument width of pad expects Number, found String        |
      | total(1, "a")               | Runtime Error: Argument values of total expects Number, found String     |
      | days_between(1, 2)          | Runtime Error: Argument from of days_between expects Date, found Number  |
      | greet("Ann", greeting: 1)   | Runtime Error: Keyword argument greeting of greet expects String, found Number |
      | scores(a: "x")              | Runtime Error: Keyword argument a of scores expects Number, found String |

  Scenario: The return value is checked
    Given the input 'liar()'
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: Function liar returned String, expected Number"

  Scenario Outline: The argument types pick the overload
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input           | result   |
      | describe(1)     | "number" |
      | describe("a")   | "text"   |
      | describe(:a)    | "text"   |
      | 1 \|> describe  | "number" |

  Scenario: No overload accepts the arguments
    Given the input 'describe(true)'
    When I evaluate the input
    Then the evaluation should fail with exactly "Runtime Error: No overload of describe accepts (Boolean)"

  Scenario: An overloaded function's type shows every overload
    Given the input 'typeof(describe)'
    When I evaluate the input
    Then the result should be "fn(Number) -> String | fn(String | Symbol) -> String"

  Scenario Outline: Overloads are tried by arity, longest first, whatever their order
    Given the native function 'pick(*args: Any, which: String = "arity 0") -> String'
    And the native function 'pick(a: Number, *args: Any, which: String = "arity 1") -> String'
    And the native function 'kcip(a: Number, *args: Any, which: String = "arity 1") -> String'
    And the native function 'kcip(*args: Any, which: String = "arity 0") -> String'
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input         | result    |
      | pick(1, :b)   | "arity 1" |
      | pick(1)       | "arity 1" |
      | pick(:b)      | "arity 0" |
      | pick()        | "arity 0" |
      | kcip(1, :b)   | "arity 1" |
      | kcip(:b)      | "arity 0" |

  Scenario Outline: Keywords pick among overloads of the same arity, whatever their order
    Given the native function 'foo(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the native function 'foo(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                      | result |
      | foo("abc", n: 1)           | "n"    |
      | foo("abc", b: true, n: 1)  | "n"    |
      | foo("abc", t: "x")         | "t"    |
      | oof("abc", b: true, n: 1)  | "n"    |
      | oof("abc", t: "x")         | "t"    |

  Scenario Outline: A tie goes to the overload declared first
    Given the native function 'foo(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the native function 'foo(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                | result |
      | foo("abc", b: true)  | "n"    |
      | oof("abc", b: true)  | "t"    |

  Scenario Outline: No overload accepts the keywords
    Given the native function 'foo(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the native function 'foo(s: String, t: String = "", b: Boolean = false) -> String'
    And the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                | error                                                          |
      | foo("abc", n: "x")   | Runtime Error: No overload of foo accepts (String, n: String)  |
      | foo("abc", z: 1)     | Runtime Error: No overload of foo accepts (String, z: Number)  |

  Scenario Outline: The wrong number of arguments lists every accepted count
    Given the native function "two(x: Any) -> Any"
    And the native function "two(x: Any, y: Any) -> Any"
    And the native function "mixed(a: Any, b: Any, c: Any, *rest: Any) -> Any"
    And the native function "mixed(a: Any) -> Any"
    And the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input              | error                                                                  |
      | two(1, 2, 3)       | Runtime Error: Function two expected 1 or 2 argument(s), found 3       |
      | mixed()            | Runtime Error: Function mixed expected 1 or at least 3 argument(s), found 0 |
      | mixed(1, 2)        | Runtime Error: Function mixed expected 1 or at least 3 argument(s), found 2 |

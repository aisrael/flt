Feature: Type-checked function calls

  Every call is checked against the function's signatures by type: its
  positional arguments, variadic values and keyword values, and the
  function's result. Overloads are tried by arity, longest first, and the
  first whose types and keywords match is called. See
  docs/functions-and-pipe.md#calls,
  docs/adrs/013-type-checked-function-calls.md and, for the errors when
  several overloads take a call's count, docs/adrs/015-closest-overload-errors.md.

  Background:
    Given the native function "describe(n: Number) -> String"
    And the native function "describe(s: String | Symbol) -> String"
    And the native function "pad(s: String, width: Integer) -> String"
    And the native function "total(*values: Number) -> Number"
    And the native function "liar() -> Number"
    And the native function "scores(**kwargs: Number) -> Map"
    And the native function 'greet(name: String, greeting: String = "Hello") -> String'
    And the native function "date(yyyy: Integer, mm: Integer, dd: Integer) -> Date"
    And the native function "days_between(from: Date, to: Date) -> Integer"

  Scenario Outline: Arguments of the declared types pass
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                    | result  |
      | pad("ab", 4)             | "  ab"  |
      | total()                  | 0.0     |
      | total(1, 2, 3)           | 6.0     |
      | scores(a: 1, b: 2)       | {a: 1.0, b: 2.0} |
      | greet("Ann", greeting: "Hi") | "Hi, Ann" |

  Scenario Outline: An argument of the wrong type is an error
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                       | error                                                                    |
      | pad(1, 2)                   | Runtime Error: Argument s of pad expects String, found Integer           |
      | pad("x", "2")               | Runtime Error: Argument width of pad expects Integer, found String       |
      | total(1, "a")               | Runtime Error: Argument values of total expects Number, found String     |
      | days_between(1, 2)          | Runtime Error: Argument from of days_between expects Date, found Integer |
      | greet("Ann", greeting: 1)   | Runtime Error: Keyword argument greeting of greet expects String, found Integer |
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

  Scenario: Overloads that fail at the same argument name none
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

  Scenario Outline: Overloads of the same arity can differ in their first argument alone
    Given the native function "head(items: Array, n: Integer) -> Array"
    And the native function "head(s: String, n: Integer) -> String"
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                 | result |
      | [1, 2, 3] \|> head(2) | [1, 2] |
      | "abc" \|> head(2)     | "ab"   |

  Scenario Outline: The error names the overload that passes the most checks
    Given the native function "head(items: Array, n: Integer) -> Array"
    And the native function "head(s: String, n: Integer) -> String"
    And the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                  | error                                                            |
      | [1, 2, 3] \|> head("x") | Runtime Error: Argument n of head expects Integer, found String  |
      | "abc" \|> head(:x)      | Runtime Error: Argument n of head expects Integer, found Symbol  |

  Scenario Outline: A keyword of the wrong type is closer than one that isn't accepted, whatever the order
    Given the native function 'foo(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the native function 'foo(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, t: String = "", b: Boolean = false) -> String'
    And the native function 'oof(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                       | error                                                                 |
      | foo("abc", n: "x")          | Runtime Error: Keyword argument n of foo expects Number, found String |
      | oof("abc", n: "x")          | Runtime Error: Keyword argument n of oof expects Number, found String |
      | foo("abc", t: 1)            | Runtime Error: Keyword argument t of foo expects String, found Integer |
      | foo("abc", b: true, t: 1)   | Runtime Error: Keyword argument t of foo expects String, found Integer |
      | foo("abc", n: 1, z: 1)      | Runtime Error: Function foo got an unexpected keyword argument: z     |
      | oof("abc", n: 1, z: 1)      | Runtime Error: Function oof got an unexpected keyword argument: z     |

  Scenario Outline: Overloads that are equally close name none
    Given the native function "head(items: Array, n: Integer) -> Array"
    And the native function "head(s: String, n: Integer) -> String"
    And the native function 'foo(s: String, n: Number = 0, b: Boolean = false) -> String'
    And the native function 'foo(s: String, t: String = "", b: Boolean = false) -> String'
    And the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                | error                                                          |
      | head(1, 2)           | Runtime Error: No overload of head accepts (Integer, Integer)  |
      | foo("abc", z: 1)     | Runtime Error: No overload of foo accepts (String, z: Integer) |
      | foo("abc", b: 1)     | Runtime Error: No overload of foo accepts (String, b: Integer) |

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

Feature: Integer arguments

  An `Integer` parameter accepts only integers. A `Number` parameter accepts
  an `Integer` too, and converts it to a `Number` before the function runs, as
  does a `Number` keyword, variadic value or result. A union converts an
  `Integer` only if none of its members accepts it as it is. When several
  overloads of the same arity match a call, `SimpleRuntime` runs the
  narrowest. See docs/types.md#number-accepts-an-integer,
  docs/functions-and-pipe.md#the-narrowest-overload and ADR 017.

  Background:
    Given the native function "pad(s: String, width: Integer) -> String"
    And the native function "scale(x: Number) -> Number"
    And the native function "either(x: Integer | Number) -> Any"
    And the native function "num_or_str(x: Number | String) -> Any"
    And the native function "maybe(x: Option<Number>) -> Any"
    And the native function "identity(v: Any) -> Any"
    And the native function "total(*values: Number) -> Number"
    And the native function "scores(**kwargs: Number) -> Map"
    And the native function "step(by: Number = 1) -> Number"
    And the native function "rows() -> Number"

  Scenario Outline: An Integer parameter accepts an Integer
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input               | result |
      | pad("ab", 4)        | "  ab" |
      | pad("ab", 8 / 2)    | "  ab" |
      | pad("ab", 9 // 2)   | "  ab" |

  Scenario Outline: An Integer parameter doesn't accept a Number, even a whole one
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input               | error                                                              |
      | pad("ab", 4.0)      | Runtime Error: Argument width of pad expects Integer, found Number |
      | pad("ab", 4.5)      | Runtime Error: Argument width of pad expects Integer, found Number |
      | pad("ab", 8.0 / 2)  | Runtime Error: Argument width of pad expects Integer, found Number |

  Scenario Outline: A Number parameter converts an Integer
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                  | result  |
      | scale(1)               | 1.0     |
      | scale(1.5)             | 1.5     |
      | typeof(scale(1))       | Number  |
      | 2 \|> scale            | 2.0     |

  Scenario Outline: A union converts an Integer only if no member accepts it as it is
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                  | result  |
      | either(1)              | 1       |
      | either(1.0)            | 1.0     |
      | num_or_str(1)          | 1.0     |
      | num_or_str("a")        | "a"     |
      | maybe(1)               | 1.0     |
      | maybe(None)            | None    |
      | identity(1)            | 1       |
      | typeof(identity(1))    | Integer |

  Scenario Outline: Variadic values and keyword values are converted too
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                  | result       |
      | total(1, 2, 3)         | 6.0          |
      | total(1, 2.5)          | 3.5          |
      | total()                | 0.0          |
      | scores(a: 1, b: 2.5)   | {a: 1.0, b: 2.5} |
      | step(by: 2)            | 2.0          |

  Scenario: A keyword's Integer default is converted when it's used
    Given the input 'step()'
    When I evaluate the input
    Then the result should be "1.0"

  Scenario: A Number result is converted
    Given the input 'rows()'
    When I evaluate the input
    Then the result should be "5.0"

  Scenario: A function's type shows the declared types
    Given the input 'typeof(scale)'
    When I evaluate the input
    Then the result should be "fn(Number) -> Number"

  Scenario Outline: A handler reads a Number argument as a Rust integer, if it's whole
    Given the native function "take(n: Number) -> Integer"
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input      | result |
      | take(3)    | 3      |
      | take(3.0)  | 3      |

  Scenario: A handler can't read a fraction as a Rust integer
    Given the native function "take(n: Number) -> Integer"
    And the input 'take(2.5)'
    When I evaluate the input
    Then the evaluation should fail with "Cannot convert 2.5 to usize"

  Scenario Outline: A handler reads an Integer as a BigInt, and a Number as a BigDecimal
    Given the native function "double(n: Integer) -> Integer"
    And the native function "halve(x: Number) -> Number"
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                           | result               |
      | double(9223372036854775807)     | 18446744073709551614 |
      | double(-3)                      | -6                   |
      | halve(3)                        | 1.5                  |
      | halve(4)                        | 2.0                  |

  Scenario Outline: SimpleRuntime runs the narrowest overload that matches
    Given the native function "f(n: Number) -> String"
    And the native function "f(i: Integer) -> String"
    And the native function "show(x: Any) -> String"
    And the native function "show(x: Number) -> String"
    And the native function "g(a: Integer, b: Number) -> String"
    And the native function "g(a: Number, b: Integer) -> String"
    And the native function "kw(n: Number = 0) -> String"
    And the native function "kw(n: Integer = 0) -> String"
    And the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input          | result                      |
      | f(1)           | "(Integer) 1"               |
      | f(1.0)         | "(Number) 1.0"              |
      | 1 \|> f        | "(Integer) 1"               |
      | show(1.0)      | "(Number) 1.0"              |
      | show(1)        | "(Number) 1.0"              |
      | show(:a)       | "(Any) :a"                  |
      | g(1, 2.0)      | "(Integer, Number) 1, 2.0"  |
      | g(1.0, 2)      | "(Number, Integer) 1.0, 2"  |
      | g(1, 2)        | "(Integer, Number) 1, 2.0"  |
      | kw(n: 1)       | "(n: Integer) 1"            |
      | kw(n: 1.0)     | "(n: Number) 1.0"           |
      | kw()           | "(n: Number) 0.0"           |

  Scenario: A function's type lists both overloads
    Given the native function "f(n: Number) -> String"
    And the native function "f(i: Integer) -> String"
    And the input 'typeof(f)'
    When I evaluate the input
    Then the result should be "fn(Number) -> String | fn(Integer) -> String"

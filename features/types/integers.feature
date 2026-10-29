Feature: Integers

  A numeric literal without a `.` is an `Integer`, an arbitrary-precision
  whole number, and one with a `.` is a `Number`, an arbitrary-precision
  decimal. Arithmetic on two integers gives an `Integer`, including `/`, which
  floors. `//` is floor division on integers only. An `Integer` and a `Number`
  compare by value. A whole `Number` prints with `.0`. See
  docs/types.md#integers-and-numbers, docs/operators.md and ADR 017.

  Scenario Outline: A literal's type depends on whether it has a `.`
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<type>'

    Examples:
      | input          | type    |
      | typeof(1)      | Integer |
      | typeof(-7)     | Integer |
      | typeof(+7)     | Integer |
      | typeof(1_000)  | Integer |
      | typeof(1.5)    | Number  |
      | typeof(-0.5)   | Number  |
      | typeof(42.0)   | Number  |

  Scenario Outline: Integers and numbers print as they're written, with `.0` on a whole number
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<printed>'

    Examples:
      | input          | printed |
      | 42             | 42      |
      | -7             | -7      |
      | 1_000          | 1000    |
      | 3.14           | 3.14    |
      | 42.0           | 42.0    |
      | -0.0           | 0.0     |
      | 2.50           | 2.50    |

  Scenario Outline: The type of an arithmetic result depends on its operands
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result  |
      | 1 + 2              | 3       |
      | 1 + 2.0            | 3.0     |
      | 1.0 + 2            | 3.0     |
      | 1.5 + 1.5          | 3.0     |
      | 5 - 7              | -2      |
      | 5 - 7.0            | -2.0    |
      | 2 * 3              | 6       |
      | 2 * 1.5            | 3.0     |
      | 2.50 * 2           | 5.00    |
      | 0.1 + 0.2          | 0.3     |
      | -(1 + 2)           | -3      |
      | -(1.5)             | -1.5    |
      | +(2)               | 2       |
      | typeof(1 + 2)      | Integer |
      | typeof(1 + 2.0)    | Number  |
      | typeof(2 * 3)      | Integer |
      | typeof(2.0 * 3)    | Number  |
      | typeof(-(1 + 2))   | Integer |

  Scenario Outline: `/` on two integers floors, and otherwise gives a Number
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result  |
      | 7 / 2              | 3       |
      | 6 / 2              | 3       |
      | -7 / 2             | -4      |
      | 7 / -2             | -4      |
      | -7 / -2            | 3       |
      | -(7 / 2)           | -3      |
      | 1 / 3              | 0       |
      | -1 / 3             | -1      |
      | 7.0 / 2            | 3.5     |
      | 7 / 2.0            | 3.5     |
      | -7 / 2.0           | -3.5    |
      | 4.0 / 2            | 2.0     |
      | typeof(7 / 2)      | Integer |
      | typeof(7 / 2.0)    | Number  |
      | typeof(4.0 / 2)    | Number  |

  Scenario Outline: `//` floor-divides two integers
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result  |
      | 7 // 2             | 3       |
      | 6 // 2             | 3       |
      | -7 // 2            | -4      |
      | 7 // -2            | -4      |
      | -7 // -2           | 3       |
      | -(7 // 2)          | -3      |
      | typeof(7 // 2)     | Integer |
      | 7 // 2 * 2         | 6       |
      | 1 + 7 // 2         | 4       |
      | 20 // 2 // 3       | 3       |

  Scenario Outline: `//` takes only integers
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "Invalid Operand Type"

    Examples:
      | input        |
      | 7.5 // 2     |
      | 7 // 2.0     |
      | 7.0 // 2.0   |
      | 1.0 // 0     |
      | "7" // 2     |
      | 7 // true    |

  Scenario Outline: Dividing by zero is an error
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "Division By Zero"

    Examples:
      | input        |
      | 1 / 0        |
      | 0 / 0        |
      | 1.0 / 0      |
      | 1 / 0.0      |
      | 1.0 / 0.0    |
      | 1 // 0       |
      | 0 // 0       |

  Scenario Outline: Integers and numbers compare by value
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                       | result |
      | 1 == 1.0                    | true   |
      | 1.0 == 1                    | true   |
      | 1 != 1.0                    | false  |
      | 1 == 1.5                    | false  |
      | 1 < 1.5                     | true   |
      | 2 > 1.5                     | true   |
      | 1 <= 1.0                    | true   |
      | 1.0 >= 1                    | true   |
      | -7 < -6.5                   | true   |
      | [1, 2] == [1.0, 2]          | true   |
      | [1, 2] == [1.0, 2.5]        | false  |
      | {a: 1} == {a: 1.0}          | true   |
      | {a: [1]} == {a: [1.0]}      | true   |
      | typeof(1) == typeof(1.0)    | false  |
      | 1 == "1"                    | false  |

  Scenario Outline: Integers don't overflow
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                                          | result                                  |
      | 9223372036854775807 + 1                                        | 9223372036854775808                     |
      | -9223372036854775808 - 1                                       | -9223372036854775809                    |
      | 18446744073709551615 * 18446744073709551615                    | 340282366920938463426481119284349108225 |
      | 340282366920938463426481119284349108225 / 18446744073709551615 | 18446744073709551615                    |
      | 340282366920938463426481119284349108225 // 2                   | 170141183460469231713240559642174554112 |
      | typeof(18446744073709551615 * 18446744073709551615)            | Integer                                 |

  Scenario Outline: Numbers convert to strings as they print
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result     |
      | "x" <> 2           | "x2"       |
      | "x" <> 2.0         | "x2.0"     |
      | "x" <> 4.0 / 2     | "x2.0"     |
      | "x" <> 7 / 2       | "x3"       |
      | "n = {1 + 1}"      | "n = 2"    |
      | "n = {4.0 / 2}"    | "n = 2.0"  |

  Scenario Outline: Other operands are an error
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "Invalid Operand Type"

    Examples:
      | input        |
      | 1 + true     |
      | 1.0 * "2"    |
      | -"1"         |
      | 7 / None     |

Feature: Map values

  In SimpleRuntime, a map keeps its entries in insertion order, which for a
  literal is source order, and prints them in that order. A duplicate key keeps
  its first position and its last value. Equality ignores order. See
  docs/runtime-and-limitations.md#what-evaluates-today.

  Scenario Outline: Maps print in insertion order
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result                   |
      | {}                 | {}                       |
      | {b: 1, a: 2}       | {"b": 1, "a": 2}         |
      | {z: 1, y: 2, x: 3} | {"z": 1, "y": 2, "x": 3} |
      | {n: 1, m: 2, n: 3} | {"n": 3, "m": 2}         |

  Scenario Outline: Map equality ignores order
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<result>"

    Examples:
      | input                        | result |
      | {a: 1, b: 2} == {b: 2, a: 1} | true   |
      | {a: 1, b: 2} == {a: 1, b: 3} | false  |
      | {a: 1} == {a: 1, b: 2}       | false  |
      | {a: 1, b: 2} != {b: 2, a: 1} | false  |

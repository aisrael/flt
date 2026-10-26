Feature: Map values

  Every key in a map literal is a symbol: `b: 1` is shorthand for the key `:b`,
  and `"c d": 1` for `:"c d"`. A map prints its keys as they'd be written. In
  SimpleRuntime, a map keeps its entries in insertion order, which for a literal
  is source order, and prints them in that order. A duplicate key keeps its
  first position and its last value. Equality ignores order. See
  docs/literals.md#maps and docs/adrs/009-symbol-keys-and-ordered-maps.md.

  Scenario Outline: Maps print their keys as written, in insertion order
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input              | result             |
      | {}                 | {}                 |
      | {b: 1, a: 2}       | {b: 1, a: 2}       |
      | {z: 1, y: 2, x: 3} | {z: 1, y: 2, x: 3} |
      | {"b": 1}           | {b: 1}             |
      | {b: 1, "c d": 2}   | {b: 1, "c d": 2}   |
      | {"_x": 1, "1x": 2} | {"_x": 1, "1x": 2} |
      | {n: 1, m: 2, n: 3} | {n: 3, m: 2}       |
      | {b: 1, "b": 2}     | {b: 2}             |

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
      | {b: 1} == {"b": 1}           | true   |

  Scenario Outline: Field access reads a symbol key
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<result>"

    Examples:
      | input       | result |
      | {b: 1}.b    | 1      |
      | {"b": 1}.b  | 1      |

  Scenario Outline: Symbols print as they'd be written
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input          | result         |
      | :b             | :b             |
      | :"b"           | :b             |
      | :"hello world" | :"hello world" |
      | :"_x"          | :"_x"          |

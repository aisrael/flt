Feature: Const declarations

  A `const` declaration has the form of `let`: a name, an optional type after a
  `:`, and an expression. See docs/syntax.md#statements and ADR 025.

  Scenario Outline: A const declaration without a type
    Given the input "<input>"
    When I parse the input
    Then the statement should be a constant declaration of "<name>" without a type
    And the statement should print as "<input>"

    Examples:
      | input                     | name        |
      | const A = 1               | A           |
      | const DEFAULT = :default  | DEFAULT     |
      | const MAX_RETRIES = 2 + 3 | MAX_RETRIES |
      | const limit = 10          | limit       |

  Scenario Outline: A const declaration with a type
    Given the input "<input>"
    When I parse the input
    Then the statement should be a constant declaration of "<name>" with type "<type>"
    And the statement should print as "<input>"

    Examples:
      | input                          | name | type           |
      | const N: Number = 1            | N    | Number         |
      | const S: String \| None = None | S    | String \| None |
      | const T: Type = Number         | T    | Type           |

  Scenario Outline: Whitespace around the type is optional
    Given the input "<input>"
    When I parse the input
    Then the statement should be a constant declaration of "N" with type "Number"

    Examples:
      | input               |
      | const N:Number=1    |
      | const N :Number = 1 |

  Scenario Outline: Input that is not a const declaration
    Given the input "<input>"
    When I parse the input
    Then parsing should fail

    Examples:
      | input              |
      | const = 1          |
      | const 1 = 2        |
      | const if = 1       |
      | let const = 1      |
      | const: Integer = 1 |

  Scenario: A word that starts with const is a name
    Given the input "constant = 1"
    When I parse the input
    Then the statement should be an assignment to "constant"

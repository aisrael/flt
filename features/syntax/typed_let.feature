Feature: Typed let declarations

  A `let` binding can declare its variable's type after a `:`. A type is a
  name, a union `A | B` of types, or a type in parentheses. Only `let` takes a
  type: a binding without `let` is an assignment. See docs/syntax.md#statements
  and ADR 016.

  Scenario Outline: A let binding with a type
    Given the input "<input>"
    When I parse the input
    Then the statement should be a declaration of "<name>" with type "<type>"
    And the statement should print as "<input>"

    Examples:
      | input                                | name | type                    |
      | let n: Number = 1                    | n    | Number                  |
      | let s: String \| None = None         | s    | String \| None          |
      | let x: Number \| String \| None = 1  | x    | Number \| String \| None |
      | let a: Any = 1                       | a    | Any                     |
      | let t: Type = Number                 | t    | Type                    |
      | let d: Date = today()                | d    | Date                    |

  Scenario Outline: Whitespace around the type is optional
    Given the input "<input>"
    When I parse the input
    Then the statement should be a declaration of "n" with type "<type>"

    Examples:
      | input                    | type           |
      | let n:Number=1           | Number         |
      | let n :Number = 1        | Number         |
      | let n: Number\|None = 1  | Number \| None |

  Scenario: Parentheses group a type
    Given the input "let x: (Number | String) | None = None"
    When I parse the input
    Then the statement should be a declaration of "x" with type "(Number | String) | None"
    And the statement should print as "let x: (Number | String) | None = None"

  Scenario: A let binding without a type
    Given the input "let x = 1"
    When I parse the input
    Then the statement should be a declaration of "x" without a type
    And the statement should print as "let x = 1"

  Scenario: A binding without let is an assignment
    Given the input "x = 1"
    When I parse the input
    Then the statement should be an assignment to "x"
    And the statement should print as "x = 1"

  Scenario Outline: A type that doesn't parse
    Given the input "<input>"
    When I evaluate the input
    Then the evaluation should fail with "Parser Error"

    Examples:
      | input                       |
      | x: Number = 1               |
      | let x: = 1                  |
      | let x: Number               |
      | let x: Number \| = 1        |
      | let x: Number \|\| None = 1 |
      | let x: (Number = 1          |
      | let x: fn(Any) -> Any = 1   |
      | let x: 1 = 1                |

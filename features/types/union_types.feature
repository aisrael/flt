Feature: Union types

  A union type `A | B` is a value of one of the member types. An `if` with an
  `else` has type `<then type> | <else type>`, and an `if` without `else` has type
  `<then type> | None`. Types are checked at runtime, so `typeof` reports the type
  of the value the `if` actually produced, which is always one of the union's
  members. See docs/types/union_types.md.

  Scenario Outline: typeof reports the type of the branch that ran
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<type>"

    Examples:
      | input                         | type   |
      | typeof(if true 1)             | Number |
      | typeof(if false 1)            | None   |
      | typeof(if true 1 else "one")  | Number |
      | typeof(if false 1 else "one") | String |
      | typeof(if true 1 else 2)      | Number |
      | typeof(if false 1 else 2)     | Number |

  Scenario: An if with else evaluates to the then-branch when the condition is true
    Given the input 'if true 1 else "one"'
    When I evaluate the input
    Then the result should be "1"

  Scenario: An if with else evaluates to the else-branch when the condition is false
    Given the input 'if false 1 else "one"'
    When I evaluate the input
    Then the result should be '"one"'

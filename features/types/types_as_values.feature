Feature: Types as values

  Types are first-class values. The name of a built-in type evaluates to that
  type, types print without quotes, and `typeof` returns a type. A `let` binding
  shadows a built-in type name. `Any` is only a placeholder in signatures, so it
  is not a value. See docs/types.md#types-as-values.

  Scenario Outline: Types as values
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<result>"

    Examples:
      | input                 | result  |
      | Number                | Number  |
      | Boolean               | Boolean |
      | String                | String  |
      | Symbol                | Symbol  |
      | Array                 | Array   |
      | Map                   | Map     |
      | Type                  | Type    |
      | typeof(1)             | Number  |
      | typeof(1) == Number   | true    |
      | typeof("a") == Number | false   |
      | typeof(Number)        | Type    |
      | typeof(typeof(1))     | Type    |
      | typeof(Type)          | Type    |

  Scenario: A let binding shadows a built-in type name
    Given the multiline input
      """
      let Number = 5
      typeof(Number)
      """
    When I evaluate the input
    Then the result should be "Number"

  Scenario: The shadowed name evaluates to the bound value
    Given the multiline input
      """
      let Number = 5
      Number
      """
    When I evaluate the input
    Then the result should be "5"

  Scenario: Re-binding a variable changes its type
    Given the multiline input
      """
      let x = 1
      x = "a"
      typeof(x)
      """
    When I evaluate the input
    Then the result should be "String"

  Scenario Outline: A name that is neither a variable nor a built-in type is unbound
    Given the input "<input>"
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: <input>"

    Examples:
      | input |
      | Foo   |
      | Any   |

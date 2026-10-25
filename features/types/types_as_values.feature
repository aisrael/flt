Feature: Types as values

  Types are first-class values. The name of a built-in type evaluates to that
  type, types print without quotes, and `typeof` returns a type. A built-in type
  name can't be bound. `Any` is only a placeholder in signatures, so it is not a
  value. See docs/types.md#types-as-values.

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

  Scenario Outline: A built-in type name can't be bound
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "<error>"

    Examples:
      | input              | error                                    |
      | let None = 1       | Cannot bind None: it's a built-in type   |
      | let Number = false | Cannot bind Number: it's a built-in type |
      | Number = 1         | Cannot bind Number: it's a built-in type |
      | let Any = 1        | Cannot bind Any: it's a built-in type    |

  Scenario: A refused binding stops evaluation
    Given the multiline input
      """
      let Number = 5
      typeof(1) == Number
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind Number: it's a built-in type"

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

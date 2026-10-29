Feature: Typed bindings

  `let x: T = e` declares `x` with the type `T`, and `e`'s value must be of
  that type, by the same rules as a function argument. A later assignment,
  `x = e`, is checked against the declared type, until `x` is declared again
  with `let`. A name that isn't a built-in or native type raises an
  `UnknownType` error before the value is evaluated. See
  docs/types.md#type-annotations and ADR 016.

  Scenario Outline: A value of the declared type is bound
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                              | result |
      | let n: Number = 1.5; n                             | 1.5    |
      | let s: String = "a"; s                             | "a"    |
      | let x: Number \| None = None; x                    | None   |
      | let x: Number \| None = 1.5; x                     | 1.5    |
      | let x: (Number \| String) \| None = "a"; x         | "a"    |
      | let a: Any = :a; a                                 | :a     |
      | let t: Type = Number; t                            | Number |
      | let n: None = None; n                              | None   |
      | let m: Map = {a: 1}; m                             | {a: 1} |
      | let a: Array = [1]; a                              | [1]    |
      | let f: Any = typeof; f(1)                          | Integer |

  Scenario: A declaration evaluates to the bound value
    Given the input 'let n: Number = 2.5'
    When I evaluate the input
    Then the result should be "2.5"

  Scenario Outline: A value of another type isn't bound
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "<error>"

    Examples:
      | input                             | error                                                |
      | let n: Number = "one"             | Variable n expects Number, found String              |
      | let s: String = 1                 | Variable s expects String, found Integer             |
      | let x: Number \| None = "a"       | Variable x expects Number \| None, found String      |
      | let n: None = 1                   | Variable n expects None, found Integer               |
      | let t: Type = 1                   | Variable t expects Type, found Integer               |
      | let b: Boolean = None             | Variable b expects Boolean, found None               |

  Scenario Outline: An assignment keeps to the declared type
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                         | result |
      | let n: Number = 1.5; n = 2.5; n               | 2.5    |
      | let x: Number \| None = 1; x = None; x        | None   |
      | let a: Any = 1; a = "a"; a                    | "a"    |

  Scenario: An assignment of another type fails
    Given the input 'let n: Number = 1; n = "two"'
    When I evaluate the input
    Then the evaluation should fail with "Variable n expects Number, found String"

  Scenario: A let without a type declares the variable again
    Given the input 'let n: Number = 1; let n = "two"; n = :two; n'
    When I evaluate the input
    Then the result should be ":two"

  Scenario: A let with a type replaces the declared type
    Given the input 'let n: Number = 1; let n: String = "two"; n = "three"; n'
    When I evaluate the input
    Then the result should be '"three"'

  Scenario: A let with a new type checks later assignments against it
    Given the input 'let n: Number = 1; let n: String = "two"; n = 3'
    When I evaluate the input
    Then the evaluation should fail with "Variable n expects String, found Integer"

  Scenario: Without types, a binding replaces the value and its type
    Given the input 'let x = 1; x = "a"; typeof(x)'
    When I evaluate the input
    Then the result should be "String"

  Scenario: Assigning to an unbound name declares it without a type
    Given the input 'x = 1; x = "a"; x'
    When I evaluate the input
    Then the result should be '"a"'

  Scenario: typeof reports the value's type, not the declared type
    Given the input 'let x: Integer | String = 1; typeof(x)'
    When I evaluate the input
    Then the result should be "Integer"

  Scenario Outline: An unknown type name
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "<error>"

    Examples:
      | input                             | error                |
      | let x: Foo = 1                    | Unknown type: Foo    |
      | let x: typeof = 1                 | Unknown type: typeof |
      | let x: Number \| Foo = 1          | Unknown type: Foo    |
      | let T = Number; let x: T = 1      | Unknown type: T      |

  Scenario: The type is checked before the value is evaluated
    Given the input 'let x: Foo = 1 / 0'
    When I evaluate the input
    Then the evaluation should fail with "Unknown type: Foo"

  Scenario: The name is checked before the type
    Given the input 'let Number: Foo = 1'
    When I evaluate the input
    Then the evaluation should fail with "Cannot bind Number: it's a built-in type"

  Scenario: A native type in a declaration
    Given the native function "date(yyyy: Integer, mm: Integer, dd: Integer) -> Date"
    And the input 'let d: Date = date(2026, 10, 28); d'
    When I evaluate the input
    Then the result should be "2026-10-28"

  Scenario: A value of another type isn't bound to a native type
    Given the native function "date(yyyy: Integer, mm: Integer, dd: Integer) -> Date"
    And the input 'let d: Date = 1'
    When I evaluate the input
    Then the evaluation should fail with "Variable d expects Date, found Integer"

  Scenario Outline: A variable declared Integer takes only integers
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                 | result  |
      | let i: Integer = 1; i                 | 1       |
      | let i: Integer = 7 / 2; i             | 3       |
      | let i: Integer = 1; i = 2; i          | 2       |
      | let i: Integer \| None = None; i      | None    |

  Scenario Outline: A variable declared Integer doesn't take a Number, even a whole one
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with "<error>"

    Examples:
      | input                               | error                                     |
      | let i: Integer = 1.0                | Variable i expects Integer, found Number  |
      | let i: Integer = 4.0 / 2            | Variable i expects Integer, found Number  |
      | let i: Integer = 1; i = 2.5         | Variable i expects Integer, found Number  |

  Scenario Outline: A variable declared Number converts an Integer
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                       | result  |
      | let n: Number = 1; n                        | 1.0     |
      | let n: Number = 1; typeof(n)                | Number  |
      | let n: Number = 1                           | 1.0     |
      | let n: Number = 1.5; n = 2; n               | 2.0     |
      | let n: Number = 1.5; n = 2                  | 2.0     |
      | let x: Number \| None = 1; x                | 1.0     |
      | let x: Number \| String = 1; typeof(x)      | Number  |
      | let x: Integer \| Number = 1; typeof(x)     | Integer |
      | let x: Integer \| String = 1; x             | 1       |
      | let a: Any = 1; typeof(a)                   | Integer |
      | let x = 1; typeof(x)                        | Integer |

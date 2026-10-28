Feature: Native function arguments

  A native function's handler reads each argument by the name its signature
  gives it, converted to a Rust type, whether the call passed it positionally
  or as a keyword. Names are those of the overload that matched. A handler that
  reads a name the overload doesn't declare, or reads a variadic argument as one
  value, gets a handler error, which is a bug in the host rather than in the flt
  source. See docs/native-functions-and-values.md#reading-arguments and
  docs/adrs/014-native-function-arguments.md.

  The fixtures:

  - `repeat` reads `s` with `get::<&str>` and `n` with `get::<usize>`. `n` is
    positional in one overload and a keyword in the other.
  - `twice` reads `n` with `get::<Option<i64>>`, and doubles it if it isn't `None`.
  - `shout` declares `v: Any`, and reads it with `get::<&str>`.
  - `size` reads `items` with `get::<&[Value]>`.
  - `typo` reads `nmae`, which its signature doesn't declare.
  - `first` reads its variadic argument `values` with `get::<&Value>`.

  Background:
    Given the native function "repeat(s: String, n: Number) -> String"
    And the native function "repeat(s: String, n: Number = 2) -> String"

  Scenario Outline: An argument is read by name, positional or keyword
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                 | result   |
      | repeat("ab", 3)       | "ababab" |
      | repeat("ab")          | "abab"   |
      | repeat("ab", n: 3)    | "ababab" |
      | "ab" \|> repeat(3)    | "ababab" |
      | "ab" \|> repeat(n: 0) | ""       |

  Scenario: An argument can't be both positional and a keyword
    Given the input 'repeat("ab", 3, n: 3)'
    When I evaluate the input
    Then the evaluation should fail with "Function repeat got an unexpected keyword argument: n"

  Scenario: A number that doesn't fit the Rust type isn't converted
    Given the input 'repeat("ab", -1)'
    When I evaluate the input
    Then the evaluation should fail with "Cannot convert -1 to usize"

  # Plain scenarios, since an outline would read <Number> as a placeholder.
  Scenario: An Option argument reads a value
    Given the native function "twice(n: Option<Number>) -> Option<Number>"
    And the input 'twice(21)'
    When I evaluate the input
    Then the result should be "42"

  Scenario: An Option argument reads None as None
    Given the native function "twice(n: Option<Number>) -> Option<Number>"
    And the input 'twice(None)'
    When I evaluate the input
    Then the result should be "None"

  Scenario: A handler reads an array's elements
    Given the native function "size(items: Array) -> Number"
    And the input 'size([1, :a, "b"])'
    When I evaluate the input
    Then the result should be "3"

  Scenario: An Any argument read as a String
    Given the native function "shout(v: Any) -> String"
    And the input 'shout("hi")'
    When I evaluate the input
    Then the result should be '"HI!"'

  Scenario: An Any argument that isn't a String can't be read as one
    Given the native function "shout(v: Any) -> String"
    And the input 'shout(1)'
    When I evaluate the input
    Then the evaluation should fail with "Expected String, found Number"

  Scenario: Reading an undeclared name is a handler error
    Given the native function "typo(name: String) -> String"
    And the input 'typo("Ann")'
    When I evaluate the input
    Then the evaluation should fail with exactly "Handler Error: Function typo has no argument nmae"

  Scenario: Reading a variadic argument as one value is a handler error
    Given the native function "first(*values: Any) -> Any"
    And the input 'first(1, 2)'
    When I evaluate the input
    Then the evaluation should fail with exactly "Handler Error: Argument values of first is variadic; read it with Arguments::rest"

  Scenario: A handler error is raised whether or not the call has values for it
    Given the native function "first(*values: Any) -> Any"
    And the input 'first()'
    When I evaluate the input
    Then the evaluation should fail with exactly "Handler Error: Argument values of first is variadic; read it with Arguments::rest"

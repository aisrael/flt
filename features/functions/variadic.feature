Feature: Variadic arguments

  A function's last argument can be variadic: it takes zero or more positional
  values, after the arguments before it, which are required. See
  docs/native-functions-and-values.md#variadic-arguments and
  docs/adrs/011-variadic-arguments.md.

  Background:
    Given the native function "count_args(*args: Any) -> Number"
    And the native function "sprintf(format: String, *args: Any) -> String"

  Scenario Outline: A variadic argument takes any number of arguments
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<count>"

    Examples:
      | input                  | count |
      | count_args()           | 0     |
      | count_args(1)          | 1     |
      | count_args(1, 2, 3)    | 3     |
      | 1 \|> count_args(2, 3) | 3     |
      | 1 \|> count_args       | 1     |

  Scenario: The type of a function with only a variadic argument
    Given the input 'typeof(count_args)'
    When I evaluate the input
    Then the result should be "fn(*Any) -> Number"

  Scenario Outline: Fixed arguments are filled before the variadic one
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<variadic>'

    Examples:
      | input                    | variadic |
      | sprintf("none")          | []       |
      | sprintf("%d", 1)         | [1]      |
      | sprintf("%d %s", 1, "a") | [1, "a"] |
      | "%d" \|> sprintf(1)      | [1]      |
      | "none" \|> sprintf       | []       |

  Scenario: A fixed argument before the variadic one is required
    Given the input 'sprintf()'
    When I evaluate the input
    Then the evaluation should fail with "Function sprintf expected at least 1 argument(s), found 0"

  Scenario: The type of a function with a fixed argument before the variadic one
    Given the input 'typeof(sprintf)'
    When I evaluate the input
    Then the result should be "fn(String, *Any) -> String"

  Scenario: A variadic argument doesn't receive keywords
    Given the input 'count_args(1, 2, k: 3)'
    When I evaluate the input
    Then the evaluation should fail with "Function count_args got an unexpected keyword argument: k"

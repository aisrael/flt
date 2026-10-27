Feature: Keyword arguments

  A call's keyword arguments are passed by name, separately from its positional
  arguments, and checked against the keywords the function declares. Each
  declared keyword is optional and has a default. An undeclared keyword is an
  error, and a repeated keyword doesn't parse. See
  docs/functions-and-pipe.md#keyword-arguments and
  docs/adrs/010-keyword-arguments.md.

  Background:
    Given the native function 'greet(name: String, greeting: String = "Hello") -> String'
    And the native function "options(**kwargs: Any) -> Map"
    And the native function "keys(map: Map) -> Array"

  Scenario: An omitted keyword takes its default
    Given the input 'greet("Ann")'
    When I evaluate the input
    Then the result should be '"Hello, Ann"'

  Scenario: A keyword overrides its default
    Given the input 'greet("Ann", greeting: "Hi")'
    When I evaluate the input
    Then the result should be '"Hi, Ann"'

  Scenario: A quoted keyword names the same keyword
    Given the input 'greet("Ann", "greeting": "Hi")'
    When I evaluate the input
    Then the result should be '"Hi, Ann"'

  Scenario: Piping passes the piped value positionally
    Given the input '"Ann" |> greet(greeting: "Hi")'
    When I evaluate the input
    Then the result should be '"Hi, Ann"'

  Scenario: A keyword is never filled by position
    Given the input 'greet("Ann", "Hi")'
    When I evaluate the input
    Then the evaluation should fail with "Function greet expected 1 argument(s), found 2"

  Scenario: An undeclared keyword is an error
    Given the input 'greet("Ann", tone: "warm")'
    When I evaluate the input
    Then the evaluation should fail with "Function greet got an unexpected keyword argument: tone"

  Scenario: A function without keywords rejects them
    Given the input 'keys({}, b: 1)'
    When I evaluate the input
    Then the evaluation should fail with "Function keys got an unexpected keyword argument: b"

  Scenario: A built-in rejects keywords
    Given the input 'typeof(1, k: 2)'
    When I evaluate the input
    Then the evaluation should fail with "Function typeof got an unexpected keyword argument: k"

  Scenario: A map literal is a positional argument
    Given the input 'keys({b: 1, a: 2})'
    When I evaluate the input
    Then the result should be "[:b, :a]"

  Scenario: Rest keywords arrive in call order
    Given the input 'options(b: 1, a: 2)'
    When I evaluate the input
    Then the result should be "{b: 1, a: 2}"

  Scenario: No keywords give an empty map
    Given the input 'options()'
    When I evaluate the input
    Then the result should be "{}"

  Scenario Outline: A repeated keyword doesn't parse
    Given the input '<input>'
    When I parse the input
    Then parsing should fail

    Examples:
      | input                                           |
      | greet("Ann", greeting: "Hi", greeting: "Yo")    |
      | greet("Ann", greeting: "Hi", "greeting": "Yo")  |
      | greet "Ann", greeting: "Hi", greeting: "Yo"     |
      | "Ann" \|> greet(greeting: "Hi", greeting: "Yo") |

  Scenario Outline: A repeated keyword's error names the keyword and the call
    Given the input '<input>'
    When I evaluate the input
    Then the evaluation should fail with exactly '<error>'

    Examples:
      | input                                          | error                                                                  |
      | greet("Ann", greeting: "Hi", greeting: "Yo")   | Parser Error: Duplicate keyword argument greeting in the call to greet |
      | greet("Ann", greeting: "Hi", "greeting": "Yo") | Parser Error: Duplicate keyword argument greeting in the call to greet |
      | greet "Ann", greeting: "Hi", greeting: "Yo"    | Parser Error: Duplicate keyword argument greeting in the call to greet |
      | options("c d": 1, "c d": 2)                    | Parser Error: Duplicate keyword argument "c d" in the call to options  |

  Scenario: A map literal keeps the last duplicate key
    Given the input 'keys({n: 1, n: 2})'
    When I evaluate the input
    Then the result should be "[:n]"

  Scenario Outline: Function types show keywords
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<type>"

    Examples:
      | input           | type                                   |
      | typeof(greet)   | fn(String, greeting: String) -> String |
      | typeof(options) | fn(**Any) -> Map                       |

Feature: Brace blocks

  A brace block starts with `{` and ends with `}`. It holds statements, and it is
  an expression. It evaluates to its last statement. Statements are separated by
  a line break or a `;`. A `{` starts a map if the next token is `}`, or a key
  followed by `:`. Otherwise it starts a block (ADR 020).

  Scenario: A block with one expression
    Given the input "{ 1 }"
    When I parse the input
    Then the output should be 'Block([Expr(Literal(Integer(1)))])'
    And the output should print as "{ 1 }"

  Scenario: A block with an identifier is a block, not a map
    Given the input "{ a }"
    When I parse the input
    Then the output should be 'Block([Expr(Ident("a"))])'

  Scenario: A block with two statements on one line
    Given the input "{ let a = 1; a * 2 }"
    When I parse the input
    Then the output should print as "{ let a = 1; a * 2 }"
    And the printed output should parse back to the same expression

  Scenario: A block with statements on separate lines
    Given the multiline input
      """
      {
        let a = 1
        a + 1
      }
      """
    When I parse the input
    Then the output should print as "{ let a = 1; a + 1 }"

  Scenario: A block can hold an assignment
    Given the input "{ a = 1; a }"
    When I parse the input
    Then the output should print as "{ a = 1; a }"

  Scenario: A block can hold another block
    Given the input "{ { 1 } }"
    When I parse the input
    Then the output should print as "{ { 1 } }"

  Scenario: A block is an operand
    Given the input "{ 1 } + { 2 }"
    When I parse the input
    Then the output should print as "{ 1 } + { 2 }"

  Scenario: A key followed by a colon starts a map
    Given the input "{ a: 1 }"
    When I parse the input
    Then the output should be a map with key "a" and integer value 1

  Scenario: A quoted key followed by a colon starts a map
    Given the input '{ "a b": 1 }'
    When I parse the input
    Then the output should be a map with key "a b" and integer value 1

  Scenario: An empty pair of braces is an empty map
    Given the input "{}"
    When I parse the input
    Then the output should be an empty map

  Scenario Outline: A block that is not closed doesn't parse
    Given the input "<input>"
    When I parse the input
    Then parsing should fail

    Examples:
      | input         |
      | { 1           |
      | { let a = 1;  |
      | { { 1 }       |

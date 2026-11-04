Feature: Expanded if and elsif

  An if has a compact form (`if c { a } else { b }`) and an expanded form
  (`if c`, statements, `else`, statements, `end`) (ADR 022). Both give the same
  AST: an IfExpr with block arms. `elsif` is sugar for an else arm that holds
  one nested if, and one `end` closes the whole chain.

  Scenario: The expanded form gives the same AST as the compact form
    Given the multiline input
      """
      if true
        1
      else
        2
      end
      """
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: Some(Block([Expr(Literal(Integer(2)))])) }'

  Scenario: The expanded form prints in the compact form
    Given the multiline input
      """
      if ready
        let n = 2
        n * 3
      else
        0
      end
      """
    When I parse the input
    Then the output should print as "if ready { let n = 2; n * 3 } else { 0 }"
    And the printed output should parse back to the same expression

  Scenario: The expanded form without else
    Given the multiline input
      """
      if ready
        1
      end
      """
    When I parse the input
    Then the output should print as "if ready { 1 }"

  Scenario: A semicolon after the condition starts the expanded form
    Given the input "if x; a; else b; end"
    When I parse the input
    Then the output should print as "if x { a } else { b }"

  Scenario: Statements separated by semicolons in an arm
    Given the input "if x; let y = 1; y; else 0; end"
    When I parse the input
    Then the output should print as "if x { let y = 1; y } else { 0 }"

  Scenario: The first statement after else can be on the same line
    Given the multiline input
      """
      if x
        a
      else b
      end
      """
    When I parse the input
    Then the output should print as "if x { a } else { b }"

  Scenario: A comment after the condition starts the expanded form
    Given the multiline input
      """
      if x # is x set?
        a
      else
        b
      end
      """
    When I parse the input
    Then the output should print as "if x { a } else { b }"

  Scenario: An arm with no statements is an empty block
    Given the multiline input
      """
      if x
      else
        b
      end
      """
    When I parse the input
    Then the output should print as "if x {} else { b }"

  Scenario: An empty expanded if evaluates to None
    Given the multiline input
      """
      if true
      else
      end
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: The expanded form evaluates to the last statement of the arm that ran
    Given the multiline input
      """
      if true
        let a = 1
        a * 2
      else
        0
      end
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: The expanded form evaluates the else arm when the condition is false
    Given the multiline input
      """
      if false
        0
      else
        let b = 3
        b
      end
      """
    When I evaluate the input
    Then the result should be "3"

  Scenario: An expanded if without else and a false condition evaluates to None
    Given the multiline input
      """
      if false
        1
      end
      """
    When I evaluate the input
    Then the result should be "None"

  Scenario: A name declared in an expanded arm is gone after end
    Given the multiline input
      """
      if true
        let a = 1
      end
      a
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: An expanded if inside an expanded if
    Given the multiline input
      """
      if true
        if false
          1
        else
          2
        end
      else
        3
      end
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: An expanded if in a brace block
    Given the multiline input
      """
      {
        let a = 1
        if a == 1
          "one"
        else
          "other"
        end
      }
      """
    When I evaluate the input
    Then the result should be '"one"'

  Scenario: An expanded if in a function body
    Given the multiline input
      """
      def sign(n: Number) -> String
        if n < 0
          "negative"
        else
          "other"
        end
      end
      sign(-1)
      """
    When I evaluate the input
    Then the result should be '"negative"'

  Scenario: An expanded if after a let statement
    Given the multiline input
      """
      let x = true
      if x
        1
      else
        2
      end
      """
    When I evaluate the input
    Then the result should be "1"

  Scenario Outline: An expanded if needs end
    Given the input "<input>"
    When I parse the input
    Then parsing should fail

    Examples:
      | input                  |
      | if x; a; else b        |
      | if x; a                |
      | if x; a; elsif y; b    |
      | if x; a; else b; elsif y; c; end |

  Scenario: An expanded if on several lines without end is an error
    Given the multiline input
      """
      if x
        1
      else
        2
      """
    When I parse the input
    Then parsing should fail

  Scenario: elsif in the compact form is a nested if in the else arm
    Given the input "if a { 1 } elsif b { 2 } else { 3 }"
    When I parse the input
    Then the output should print as "if a { 1 } else { if b { 2 } else { 3 } }"
    And the printed output should parse back to the same expression

  Scenario: elsif gives the same AST as the written-out form
    Given the input "if a { 1 } elsif b { 2 } else { 3 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Ident("a"), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: Some(Block([Expr(IfExpr { condition: Ident("b"), then_branch: Block([Expr(Literal(Integer(2)))]), else_branch: Some(Block([Expr(Literal(Integer(3)))])) })])) }'

  Scenario: elsif without else
    Given the input "if a { 1 } elsif b { 2 }"
    When I parse the input
    Then the output should print as "if a { 1 } else { if b { 2 } }"

  Scenario: elsif with expression arms
    Given the input 'if a 1 elsif b 2 else 3'
    When I parse the input
    Then the output should print as "if a 1 else { if b 2 else 3 }"

  Scenario: Several elsif arms
    Given the input "if a { 1 } elsif b { 2 } elsif c { 3 } else { 4 }"
    When I parse the input
    Then the output should print as "if a { 1 } else { if b { 2 } else { if c { 3 } else { 4 } } }"

  Scenario: elsif in the expanded form, closed by one end
    Given the multiline input
      """
      if a
        "first"
      elsif b
        "second"
      else
        "third"
      end
      """
    When I parse the input
    Then the output should print as 'if a { "first" } else { if b { "second" } else { "third" } }'
    And the printed output should parse back to the same expression

  Scenario: elsif in the expanded form on one line
    Given the input "if a; 1; elsif b; 2; else 3; end"
    When I parse the input
    Then the output should print as "if a { 1 } else { if b { 2 } else { 3 } }"

  Scenario Outline: The first true condition runs its arm
    Given the multiline input
      """
      let n = <n>
      if n < 0
        "negative"
      elsif n == 0
        "zero"
      else
        "positive"
      end
      """
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | n  | result     |
      | -5 | "negative" |
      | 0  | "zero"     |
      | 7  | "positive" |

  Scenario Outline: elsif in the compact form
    Given the input '<input>'
    When I evaluate the input
    Then the result should be '<result>'

    Examples:
      | input                                                  | result  |
      | if true { "a" } elsif true { "b" } else { "c" }        | "a"     |
      | if false { "a" } elsif true { "b" } else { "c" }       | "b"     |
      | if false { "a" } elsif false { "b" } else { "c" }      | "c"     |
      | if false { "a" } elsif false { "b" }                   | None    |
      | if false 1 elsif false 2 elsif true 3 else 4           | 3       |

  Scenario: Later conditions are not evaluated after a true one
    Given the input "if true { 1 } elsif missing { 2 } else { 3 }"
    When I evaluate the input
    Then the result should be "1"

  Scenario: An elsif condition is evaluated in the enclosing scope
    Given the multiline input
      """
      let a = false
      let b = true
      if a
        "a"
      elsif b
        "b"
      end
      """
    When I evaluate the input
    Then the result should be '"b"'

  Scenario: A name declared in an elsif arm is gone after end
    Given the multiline input
      """
      if false
        0
      elsif true
        let c = 1
      end
      c
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: c"

  Scenario Outline: elsif is used wrongly
    Given the input "<input>"
    When I parse the input
    Then parsing should fail

    Examples:
      | input                         |
      | if a { 1 } else { 2 } elsif b { 3 } |
      | if a; 1; else 2; elsif b; 3; end |
      | if a { 1 } elsif { 2 }        |
      | if a; 1; elsif; 2; end        |
      | let elsif = 1                 |

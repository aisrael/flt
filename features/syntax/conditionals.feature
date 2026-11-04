Feature: Conditionals

  If expressions evaluate a condition (must be boolean), then either the then-branch
  or the else-branch. The else branch is optional; when omitted and the condition
  is false, the expression evaluates to `None`. A branch in braces is a brace block
  (ADR 020), so it can hold statements. A branch without braces is a plain expression.

  Scenario: If with block branches and else
    Given the input "if true { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: Some(Block([Expr(Literal(Integer(2)))])) }'

  Scenario: If with block branches and else (condition false)
    Given the input "if false { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: Some(Block([Expr(Literal(Integer(2)))])) }'

  Scenario: If with block then-branch only (no else)
    Given the input "if true { 1 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: None }'

  Scenario: If with block then-branch only and function call (no else)
    Given the input "if false { do() }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: Block([Expr(FunctionCall(FunctionCall { name: Identifier("do"), positional_args: [], keyword_args: {} }))]), else_branch: None }'

  Scenario: If with expression branches (no blocks)
    Given the input 'if success "Ok" else ":("'
    When I parse the input
    Then the output should be 'IfExpr { condition: Ident("success"), then_branch: Literal(String("Ok")), else_branch: Some(Literal(String(":("))) }'

  Scenario: If with parenthesized condition
    Given the input "if (true) { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Parenthesized(Literal(Boolean(true))), then_branch: Block([Expr(Literal(Integer(1)))]), else_branch: Some(Block([Expr(Literal(Integer(2)))])) }'

  Scenario: An if with else evaluates to the then-branch when the condition is true
    Given the input 'if true 1 else "one"'
    When I evaluate the input
    Then the result should be "1"

  Scenario: An if with else evaluates to the else-branch when the condition is false
    Given the input 'if false 1 else "one"'
    When I evaluate the input
    Then the result should be '"one"'

  Scenario Outline: typeof reports the type of the branch that ran
    Given the input '<input>'
    When I evaluate the input
    Then the result should be "<type>"

    Examples:
      | input                         | type    |
      | typeof(if true 1)             | Integer |
      | typeof(if true 1 else "one")  | Integer |
      | typeof(if false 1 else "one") | String  |
      | typeof(if true 1 else 2)      | Integer |
      | typeof(if false 1 else 2)     | Integer |
      | typeof(if false 1 else 2.5)   | Number  |

  Scenario: A branch without braces is a plain expression
    Given the input 'if true 1 else 2'
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Integer(1)), else_branch: Some(Literal(Integer(2))) }'

  Scenario: Braces around a name make a block arm
    Given the input "if true { a } else { b }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Block([Expr(Ident("a"))]), else_branch: Some(Block([Expr(Ident("b"))])) }'

  Scenario: A pair of braces as an arm is an empty block
    Given the input "if true {}"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Block([]), else_branch: None }'

  Scenario: A pair of braces as the else arm is an empty block
    Given the input "if true 1 else {}"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Integer(1)), else_branch: Some(Block([])) }'

  Scenario: A map with a key as an arm is a map
    Given the input "if true { a: 1 }"
    When I parse the input
    Then the output should print as "if true { a: 1 }"

  Scenario: Braces and a block print and parse back
    Given the input "if true { let a = 1; a } else { 0 }"
    When I parse the input
    Then the output should print as "if true { let a = 1; a } else { 0 }"
    And the printed output should parse back to the same expression

  Scenario: A block arm with statements on separate lines
    Given the multiline input
      """
      if true {
        let a = 1
        a * 2
      } else {
        0
      }
      """
    When I parse the input
    Then the output should print as "if true { let a = 1; a * 2 } else { 0 }"

  Scenario: A brace arm evaluates to its last statement
    Given the input "if true { let a = 1; a * 2 } else { 0 }"
    When I evaluate the input
    Then the result should be "2"

  Scenario: The else arm evaluates to its last statement
    Given the input "if false { 0 } else { let b = 3; b }"
    When I evaluate the input
    Then the result should be "3"

  Scenario: An if without else and a false condition evaluates to None
    Given the input "if false { 1 }"
    When I evaluate the input
    Then the result should be "None"

  Scenario: A name declared in the then arm is gone after the if
    Given the multiline input
      """
      if true { let a = 1 }
      a
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: A name declared in the else arm is gone after the if
    Given the multiline input
      """
      if false 0 else { let b = 1 }
      b
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: b"

  Scenario: An arm hides a variable of the enclosing scope only inside the arm
    Given the multiline input
      """
      let c = true
      let r = if c { let c = false; c } else { true }
      [r, c]
      """
    When I evaluate the input
    Then the result should be "[false, true]"

  Scenario: An assignment in an arm changes a variable of the enclosing scope
    Given the multiline input
      """
      let x = 1
      if true { x = 2 }
      x
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: A pair of braces as an arm evaluates to None
    Given the input "if true {}"
    When I evaluate the input
    Then the result should be "None"

  Scenario: A pair of braces as the else arm evaluates to None
    Given the input "if false 1 else {}"
    When I evaluate the input
    Then the result should be "None"

  Scenario: Parentheses around a pair of braces make an empty map
    Given the input "if true ({})"
    When I evaluate the input
    Then the result should be "{}"

  Scenario: An empty map as an arm has the type Map
    Given the input "typeof(if true ({}))"
    When I evaluate the input
    Then the result should be "Map"

  Scenario: A map with a key as an arm evaluates to the map
    Given the input "if true { a: 1 }"
    When I evaluate the input
    Then the result should be "{a: 1}"

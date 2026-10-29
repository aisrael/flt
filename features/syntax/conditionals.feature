Feature: Conditionals

  If expressions evaluate a condition (must be boolean), then either the then-branch
  or the else-branch. The else branch is optional; when omitted and the condition
  is false, the expression evaluates to `None`.

  Scenario: If with block branches and else
    Given the input "if true { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Integer(1)), else_branch: Some(Literal(Integer(2))) }'

  Scenario: If with block branches and else (condition false)
    Given the input "if false { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: Literal(Integer(1)), else_branch: Some(Literal(Integer(2))) }'

  Scenario: If with block then-branch only (no else)
    Given the input "if true { 1 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Integer(1)), else_branch: None }'

  Scenario: If with block then-branch only and function call (no else)
    Given the input "if false { do() }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: FunctionCall(FunctionCall { name: Identifier("do"), positional_args: [], keyword_args: {} }), else_branch: None }'

  Scenario: If with expression branches (no blocks)
    Given the input 'if success "Ok" else ":("'
    When I parse the input
    Then the output should be 'IfExpr { condition: Ident("success"), then_branch: Literal(String("Ok")), else_branch: Some(Literal(String(":("))) }'

  Scenario: If with parenthesized condition
    Given the input "if (true) { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Parenthesized(Literal(Boolean(true))), then_branch: Literal(Integer(1)), else_branch: Some(Literal(Integer(2))) }'

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

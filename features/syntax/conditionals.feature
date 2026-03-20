Feature: Conditionals

  If expressions evaluate a condition (must be boolean), then either the then-branch
  or the else-branch. The else branch is optional; when omitted and the condition
  is false, the expression evaluates to unit.

  Scenario: If with block branches and else
    Given the input "if true { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), else_branch: Some(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2]))))) }'

  Scenario: If with block branches and else (condition false)
    Given the input "if false { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), else_branch: Some(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2]))))) }'

  Scenario: If with block then-branch only (no else)
    Given the input "if true { 1 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(true)), then_branch: Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), else_branch: None }'

  Scenario: If with block then-branch only and function call (no else)
    Given the input "if false { do() }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Literal(Boolean(false)), then_branch: FunctionCall(Identifier("do"), []), else_branch: None }'

  Scenario: If with expression branches (no blocks)
    Given the input 'if success "Ok" else ":("'
    When I parse the input
    Then the output should be 'IfExpr { condition: Ident("success"), then_branch: Literal(String("Ok")), else_branch: Some(Literal(String(":("))) }'

  Scenario: If with parenthesized condition
    Given the input "if (true) { 1 } else { 2 }"
    When I parse the input
    Then the output should be 'IfExpr { condition: Parenthesized(Literal(Boolean(true))), then_branch: Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), else_branch: Some(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2]))))) }'

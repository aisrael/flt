Feature: Unary expressions

  Unary operators `!`, `+`, and `-` apply to expressions. `!` is logical negation,
  `+` is unary plus, and `-` is negation.

  Scenario: Logical not on boolean
    Given the input "!true"
    When I parse the input
    Then the output should be 'UnaryExpr(Not, Literal(Boolean(true)))'

  Scenario: Logical not on identifier
    Given the input "!x"
    When I parse the input
    Then the output should be 'UnaryExpr(Not, Ident("x"))'

  Scenario: A minus touching an integer is part of the literal
    Given the input "-42"
    When I parse the input
    Then the output should be 'Literal(Integer(-42))'

  Scenario: A minus touching a number is part of the literal
    Given the input "-1.5"
    When I parse the input
    Then the output should be 'Literal(Number(Numeric(BigDecimal(sign=Minus, scale=1, digits=[15]))))'

  Scenario: A plus touching an integer is part of the literal
    Given the input "+17"
    When I parse the input
    Then the output should be 'Literal(Integer(17))'

  Scenario: A plus touching a number is part of the literal
    Given the input "+1.0"
    When I parse the input
    Then the output should be 'Literal(Number(Numeric(BigDecimal(sign=Plus, scale=1, digits=[10]))))'

  Scenario: Unary plus on identifier
    Given the input "+x"
    When I parse the input
    Then the output should be 'UnaryExpr(Plus, Ident("x"))'

  Scenario: Unary minus on identifier
    Given the input "-x"
    When I parse the input
    Then the output should be 'UnaryExpr(Minus, Ident("x"))'

  Scenario: Double negation
    Given the input "--42"
    When I parse the input
    Then the output should be 'UnaryExpr(Minus, Literal(Integer(-42)))'

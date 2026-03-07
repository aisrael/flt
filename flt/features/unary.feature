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

  Scenario: Unary minus on number
    Given the input "-42"
    When I parse the input
    Then the output should be 'UnaryExpr(Minus, Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[42]) })))'

  Scenario: Unary plus on number
    Given the input "+17"
    When I parse the input
    Then the output should be 'UnaryExpr(Plus, Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[17]) })))'

  Scenario: Unary minus on identifier
    Given the input "-x"
    When I parse the input
    Then the output should be 'UnaryExpr(Minus, Ident("x"))'

  Scenario: Double negation
    Given the input "--42"
    When I parse the input
    Then the output should be 'UnaryExpr(Minus, UnaryExpr(Minus, Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[42]) }))))'

Feature: Binary expressions

  Binary operators combine two expressions. Precedence (lowest to highest):
  `|>`, `||`, `&&`, `^^`, `|`, `^`, `&`, `+`/`-`/`<>`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`.

  Scenario: Addition
    Given the input "1 + 2"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), Add, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2])))))'

  Scenario: Subtraction
    Given the input "10 - 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[10])))), Sub, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[3])))))'

  Scenario: Multiplication
    Given the input "4 * 5"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[4])))), Mul, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[5])))))'

  Scenario: Division
    Given the input "20 / 4"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[20])))), Div, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[4])))))'

  Scenario: Multiplication has higher precedence than addition
    Given the input "1 + 2 * 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), Add, BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2])))), Mul, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[3]))))))'

  Scenario: Parentheses override precedence
    Given the input "(1 + 2) * 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Parenthesized(BinaryExpr(Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))), Add, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[2])))))), Mul, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[3])))))'

  Scenario: Logical and
    Given the input "true && false"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Boolean(true)), And, Literal(Boolean(false)))'

  Scenario: Logical or
    Given the input "true || false"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Boolean(true)), Or, Literal(Boolean(false)))'

  Scenario: Logical xor
    Given the input "true ^^ false"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Boolean(true)), Xor, Literal(Boolean(false)))'

  Scenario: String concatenation
    Given the input '"foo" <> "bar"'
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(String("foo")), Concat, Literal(String("bar")))'

  Scenario: Pipe operator
    Given the input "x |> f"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("x"), Pipe, Ident("f"))'

  Scenario: Equality
    Given the input "x == 1"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("x"), Eq, Literal(Number(Numeric(BigDecimal(sign=Plus, scale=0, digits=[1])))))'

  Scenario: Inequality
    Given the input "x != 0"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("x"), Ne, Literal(Number(Numeric(BigDecimal(sign=NoSign, scale=0, digits=[])))))'

  Scenario: Less than
    Given the input "a < b"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("a"), Lt, Ident("b"))'

  Scenario: Greater than
    Given the input "a > b"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("a"), Gt, Ident("b"))'

  Scenario: Less than or equal
    Given the input "a <= b"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("a"), Lte, Ident("b"))'

  Scenario: Greater than or equal
    Given the input "a >= b"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("a"), Gte, Ident("b"))'

Feature: Binary expressions

  Binary operators combine two expressions. Precedence (lowest to highest):
  `||`, `&&`, `^^`, `==`/`!=`, `<`/`>`/`<=`/`>=`, `|`, `^`, `&`, `|>`, `+`/`-`/`<>`, `*`/`/`/`//`.

  Scenario: Addition
    Given the input "1 + 2"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(1)), Add, Literal(Integer(2)))'

  Scenario: Subtraction
    Given the input "10 - 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(10)), Sub, Literal(Integer(3)))'

  Scenario: Multiplication
    Given the input "4 * 5"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(4)), Mul, Literal(Integer(5)))'

  Scenario: Division
    Given the input "20 / 4"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(20)), Div, Literal(Integer(4)))'

  Scenario: Floor division
    Given the input "7 // 2"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(7)), FloorDiv, Literal(Integer(2)))'

  Scenario: Floor division has the same precedence as multiplication, left to right
    Given the input "7 // 2 * 2"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Literal(Integer(7)), FloorDiv, Literal(Integer(2))), Mul, Literal(Integer(2)))'

  Scenario: Floor division has higher precedence than addition
    Given the input "1 + 7 // 2"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(1)), Add, BinaryExpr(Literal(Integer(7)), FloorDiv, Literal(Integer(2))))'

  Scenario: Floor division and division are different operators
    Given the input "8 / 2 // 3"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Literal(Integer(8)), Div, Literal(Integer(2))), FloorDiv, Literal(Integer(3)))'

  Scenario: Multiplication has higher precedence than addition
    Given the input "1 + 2 * 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Integer(1)), Add, BinaryExpr(Literal(Integer(2)), Mul, Literal(Integer(3))))'

  Scenario: Parentheses override precedence
    Given the input "(1 + 2) * 3"
    When I parse the input
    Then the output should be 'BinaryExpr(Parenthesized(BinaryExpr(Literal(Integer(1)), Add, Literal(Integer(2)))), Mul, Literal(Integer(3)))'

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
    Then the output should be 'BinaryExpr(Ident("x"), Eq, Literal(Integer(1)))'

  Scenario: Inequality
    Given the input "x != 0"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("x"), Ne, Literal(Integer(0)))'

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

  Scenario: Arithmetic binds tighter than equality
    Given the input "1 + 2 == 3"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Literal(Integer(1)), Add, Literal(Integer(2))), Eq, Literal(Integer(3)))'

  Scenario: Ordering binds tighter than equality
    Given the input "a < b == c"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Ident("a"), Lt, Ident("b")), Eq, Ident("c"))'

  Scenario: A pipe binds tighter than equality
    Given the input "x |> f == 3"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Ident("x"), Pipe, Ident("f")), Eq, Literal(Integer(3)))'

  Scenario: A pipe binds tighter than logical or
    Given the input "a || b |> f"
    When I parse the input
    Then the output should be 'BinaryExpr(Ident("a"), Or, BinaryExpr(Ident("b"), Pipe, Ident("f")))'

  Scenario: Arithmetic binds tighter than a pipe
    Given the input "1 + 1 |> f"
    When I parse the input
    Then the output should be 'BinaryExpr(BinaryExpr(Literal(Integer(1)), Add, Literal(Integer(1))), Pipe, Ident("f"))'

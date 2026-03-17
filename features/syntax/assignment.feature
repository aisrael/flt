Feature: Assignment

  Scenario: Assigning a value to a variable
    Given the input "let x = 1;"
    When I parse the input
    Then the output should be a `Statement::Let(Identifier("x"), Expr::Literal(Literal::Number(1)))`

  Scenario: Assigning a value without a let keyword
    Given the input "x = 1;"
    When I parse the input
    Then the output should be a `Statement::Let(Identifier("x"), Expr::Literal(Literal::Number(1)))`
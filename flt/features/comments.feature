Feature: Comments

  Comments use `#` and extend to the end of the line. They are ignored by the parser
  and may appear anywhere whitespace is allowed.

  Scenario: Trailing comment after expression
    Given the input "42 # this is the answer"
    When I parse the input
    Then the output should be a `Literal::Number(42)`

  Scenario: Leading comment before expression
    Given the multiline input
      """
      # leading comment
      42
      """
    When I parse the input
    Then the output should be a `Literal::Number(42)`

  Scenario: Comment between binary operands
    Given the multiline input
      """
      1 # add these
      + 2
      """
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[1]) })), Add, Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[2]) })))'

  Scenario: Comment in function call arguments
    Given the multiline input
      """
      add(1, # first arg
       2)
      """
    When I parse the input
    Then the output should be 'FunctionCall(Identifier("add"), [Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[1]) })), Literal(Number(Numeric { value: BigDecimal(sign=Plus, scale=0, digits=[2]) }))])'

  Scenario: Comment between string concatenation operands
    Given the multiline input
      """
      "foo" # before concat
      <> "bar"
      """
    When I parse the input
    Then the output should be 'BinaryExpr(Literal(String("foo")), Concat, Literal(String("bar")))'

  Scenario: Empty comment
    Given the input "true #"
    When I parse the input
    Then the output should be a `Literal::Boolean(true)`

  Scenario: Multiple consecutive comment lines
    Given the multiline input
      """
      # line one
      # line two
      # line three
      "hello"
      """
    When I parse the input
    Then the output should be 'Literal(String("hello"))'

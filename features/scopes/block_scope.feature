Feature: Block scope

  A brace block is an expression. It evaluates to its last statement (ADR 020).
  In the reference runtime, each brace block has a scope of its own (ADR 021):

  - A `let` in a block declares a variable in that block only.
  - A name is looked up in the innermost block, then the enclosing blocks, then
    the scope of the call, then the global scope.
  - `x = expr` changes the nearest binding of `x`. With no binding, it declares
    `x` in the innermost block.
  - A function call does not see the blocks of its caller.

  # Value

  Scenario: A block evaluates to its only expression
    Given the input "{ 1 }"
    When I evaluate the input
    Then the result should be "1"

  Scenario: A block evaluates to its last statement
    Given the input "{ let a = 1; a * 2 }"
    When I evaluate the input
    Then the result should be "2"

  Scenario: A block with statements on separate lines
    Given the multiline input
      """
      {
        let a = 1
        a + 1
      }
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: A block that ends with a declaration evaluates to the value
    Given the input "{ let a = 3 }"
    When I evaluate the input
    Then the result should be "3"

  Scenario: A block is an operand
    Given the input "{ 1 } + { 2 }"
    When I evaluate the input
    Then the result should be "3"

  Scenario: A block is an argument
    Given the input "typeof({ 1 })"
    When I evaluate the input
    Then the result should be "Integer"

  # Declarations

  Scenario: A name declared in a block is gone after the block
    Given the multiline input
      """
      { let a = 1 }
      a
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: A block can read a global variable
    Given the multiline input
      """
      let g = 10
      { g + 1 }
      """
    When I evaluate the input
    Then the result should be "11"

  Scenario: A let in a block hides a global variable until the block ends
    Given the multiline input
      """
      let x = 1
      let inside = {
        let x = 10
        x + 1
      }
      [inside, x]
      """
    When I evaluate the input
    Then the result should be "[11, 1]"

  Scenario: A typed let in a block checks the type
    Given the input "{ let n: Integer = 1.5 }"
    When I evaluate the input
    Then the evaluation should fail with "Variable n expects Integer, found Number"

  Scenario: A typed let in a block hides the type of an outer variable
    Given the multiline input
      """
      let n: Integer = 1
      { let n = "one" }
      n
      """
    When I evaluate the input
    Then the result should be "1"

  # Assignment

  Scenario: An assignment in a block changes a global variable
    Given the multiline input
      """
      let x = 1
      { x = 2 }
      x
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: An assignment in a block keeps to the declared type of the variable
    Given the multiline input
      """
      let n: Integer = 1
      { n = "one" }
      """
    When I evaluate the input
    Then the evaluation should fail with "Variable n expects Integer, found String"

  Scenario: An assignment to an unbound name declares it in the block
    Given the multiline input
      """
      { z = 5 }
      z
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: z"

  Scenario: An assignment to a name that a let declared in the block stays in the block
    Given the multiline input
      """
      let x = 1
      { let x = 2; x = 3 }
      x
      """
    When I evaluate the input
    Then the result should be "1"

  # Nested blocks

  Scenario: An inner block reads the names of the outer block
    Given the input "{ let a = 1; { let b = a + 1; b + 1 } }"
    When I evaluate the input
    Then the result should be "3"

  Scenario: An outer block doesn't see the names of an inner block
    Given the input "{ { let b = 1 }; b }"
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: b"

  Scenario: An inner block changes a variable of the outer block
    Given the input "{ let a = 1; { a = 2 }; a }"
    When I evaluate the input
    Then the result should be "2"

  # Functions

  Scenario: A block in a function body reads a parameter
    Given the multiline input
      """
      def inc(n: Integer) -> Integer
        { let m = n + 1; m }
      end
      inc(1)
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: A name declared in a block in a function body is gone after the block
    Given the multiline input
      """
      def f() -> Integer
        { let a = 1 }
        a
      end
      f()
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: An assignment in a block changes a parameter
    Given the multiline input
      """
      def f(n: Integer) -> Integer
        { n = n + 1 }
        n
      end
      f(1)
      """
    When I evaluate the input
    Then the result should be "2"

  Scenario: A function doesn't see the blocks of its caller
    Given the multiline input
      """
      def show() -> Integer
        a
      end
      { let a = 1; show() }
      """
    When I evaluate the input
    Then the evaluation should fail with "Unbound identifier: a"

  Scenario: A function called in a block reads global variables
    Given the multiline input
      """
      let g = 5
      def get() -> Integer
        g
      end
      { let a = 1; get() + a }
      """
    When I evaluate the input
    Then the result should be "6"

  # Bindings that a block can't make

  Scenario: A block can't bind the name of a built-in type
    Given the input "{ let Number = 1 }"
    When I evaluate the input
    Then the evaluation should fail with "Number"

  # Declarations of functions

  Scenario: A def in a block is not allowed
    Given the multiline input
      """
      {
        def one() -> Integer
          1
        end
      }
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot define one here: a function can only be declared at the top level"

  Scenario: A def in the arm of an if is not allowed
    Given the multiline input
      """
      if true {
        def one() -> Integer
          1
        end
      }
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot define one here"

  Scenario: A def in the body of a function is not allowed
    Given the multiline input
      """
      def outer() -> Integer
        def inner() -> Integer
          1
        end
        inner()
      end
      outer()
      """
    When I evaluate the input
    Then the evaluation should fail with "Cannot define inner here: a function can only be declared at the top level"

  Scenario: A def at the top level is allowed after a block
    Given the multiline input
      """
      { 1 }
      def one() -> Integer
        1
      end
      one()
      """
    When I evaluate the input
    Then the result should be "1"

Feature: None

  `None` is both a value and its own type. An `if` without `else` evaluates to
  `None` when its condition is false.
  See docs/types/none.md.

  Scenario: None evaluates to None
    Given the input "None"
    When I evaluate the input
    Then the result should be "None"

  Scenario: The type of None is None
    Given the input "typeof(None)"
    When I evaluate the input
    Then the result should be "None"

  Scenario: The None value equals the None type
    Given the input "typeof(None) == None"
    When I evaluate the input
    Then the result should be "true"

  Scenario: An if without else evaluates to the then-branch when the condition is true
    Given the input "if true 1"
    When I evaluate the input
    Then the result should be "1"

  Scenario: An if without else evaluates to None when the condition is false
    Given the input "if false 1"
    When I evaluate the input
    Then the result should be "None"

  Scenario: The type of an if without else is None when the condition is false
    Given the input "typeof(if false 1)"
    When I evaluate the input
    Then the result should be "None"

  Scenario: A variable bound to an if without else can be compared to None
    Given the multiline input
      """
      let x = if false 1
      x == None
      """
    When I evaluate the input
    Then the result should be "true"

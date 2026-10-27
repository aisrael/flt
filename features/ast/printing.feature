Feature: Printing expressions

  An expression prints as flt source that parses back to the same expression.
  See docs/literals.md#symbols.

  Scenario Outline: Symbols keep their colon
    Given the input '<input>'
    When I parse the input
    Then the output should print as '<printed>'
    And the printed output should parse back to the same expression

    Examples:
      | input                                                | printed                                              |
      | :id                                                  | :id                                                  |
      | :"hello world"                                       | :"hello world"                                       |
      | :"_tmp"                                              | :"_tmp"                                              |
      | select(:one)                                         | select(:one)                                         |
      | select(:id, :"full name")                            | select(:id, :"full name")                            |
      | group_by(:gender) \|> select(:gender, n: count(:id)) | group_by(:gender) \|> select(:gender, n: count(:id)) |

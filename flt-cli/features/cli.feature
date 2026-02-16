Feature: flt version

  Scenario: flt version
    When the command `flt version` is run
    Then it should exit with status code 0
    And the output should contain "flt version 0.0.1"

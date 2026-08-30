%{
  configs: [
    %{
      name: "default",
      files: %{
        included: ["config/", "lib/", "test/", "examples/", "integration_test/"],
        excluded: ["integration_test/_build/", "integration_test/tmp/"]
      },
      checks: %{
        extra: [
          {Credo.Check.Refactor.CyclomaticComplexity, max_complexity: 9}
        ]
      }
    }
  ]
}

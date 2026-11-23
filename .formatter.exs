[
  inputs: [
    "*.{ex,exs}",
    "{config,lib,test,examples}/**/*.{ex,exs}",
    "smoke/{mix,smoke}.exs",
    "smoke/config/*.exs"
  ],
  subdirectories: ["integration_test"]
]

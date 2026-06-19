defmodule MDExNative.ComrakTest do
  use ExUnit.Case

  @code_block_markdown """
  ```elixir
  IO.puts("Hello")
  ```
  """

  doctest MDExNative.Comrak

  test "anchorizes text" do
    assert MDExNative.Comrak.anchorize("Hello World") == "hello-world"
  end

  test "dangerous_url?" do
    assert MDExNative.Comrak.dangerous_url?("javascript:malicious")
    assert MDExNative.Comrak.dangerous_url?("Javascript:malicious")
    assert MDExNative.Comrak.dangerous_url?("data:malicious")
    assert MDExNative.Comrak.dangerous_url?("Data:malicious")
    assert MDExNative.Comrak.dangerous_url?("vbscript:malicious")
    assert MDExNative.Comrak.dangerous_url?("FILE:malicious")

    refute MDExNative.Comrak.dangerous_url?("data:image/png/x")
    refute MDExNative.Comrak.dangerous_url?("data:image/gif/x")
    refute MDExNative.Comrak.dangerous_url?("data:image/jpeg/x")
    refute MDExNative.Comrak.dangerous_url?("data:image/webp/x")
    refute MDExNative.Comrak.dangerous_url?("https://elixir-lang.org")
  end

  test "renders markdown with Comrak options" do
    html = MDExNative.Comrak.markdown_to_html("- [x] done", extension: [tasklist: true])

    assert html =~ ~s(<input type="checkbox" checked="" disabled="" /> done)
  end

  test "renders markdown with sanitize keyword options" do
    assert MDExNative.Comrak.markdown_to_html("<h1>Title</h1><p>Content</p>",
             render: [unsafe: true],
             sanitize: [rm_tags: ["h1"]]
           ) == "Title<p>Content</p>\n"
  end

  test "render functions return rendered strings" do
    assert MDExNative.Comrak.markdown_to_html("**bold**") == "<p><strong>bold</strong></p>\n"
    assert MDExNative.Comrak.markdown_to_xml("# Hello") =~ ~s(<heading level="1">)
  end

  test "renders parsed documents" do
    document = MDExNative.Comrak.parse_document("# Hello")

    assert MDExNative.Comrak.document_to_html(document) == "<h1>Hello</h1>\n"
    assert MDExNative.Comrak.document_to_xml(document) =~ ~s(<heading level="1">)
    assert MDExNative.Comrak.document_to_commonmark(document) == "# Hello\n"
  end

  test "parses a mixed markdown document and renders it from the document AST" do
    markdown = """
    # Release Notes

    Intro with **bold**, _emphasis_, [a link](https://example.com), and `inline code`.

    > A quote with nested content.
    >
    > - quoted item
    > - another quoted item

    1. first
    2. second

    - [x] shipped
    - [ ] pending

    ```elixir
    IO.puts("hello")
    ```

    | Name | Value |
    | ---- | ----- |
    | one  | 1     |
    | two  | 2     |
    """

    options = [extension: [tasklist: true, table: true]]
    document = MDExNative.Comrak.parse_document(markdown, options)

    assert document == %MDExNative.Comrak.Document{
             nodes: [
               %MDExNative.Comrak.Heading{
                 nodes: [
                   %MDExNative.Comrak.Text{
                     literal: "Release Notes",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 3}, end: {1, 15}}
                   }
                 ],
                 level: 1,
                 setext: false,
                 closed: false,
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 1}, end: {1, 15}}
               },
               %MDExNative.Comrak.Paragraph{
                 nodes: [
                   %MDExNative.Comrak.Text{
                     literal: "Intro with ",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 1}, end: {3, 11}}
                   },
                   %MDExNative.Comrak.Strong{
                     nodes: [
                       %MDExNative.Comrak.Text{
                         literal: "bold",
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 14}, end: {3, 17}}
                       }
                     ],
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 12}, end: {3, 19}}
                   },
                   %MDExNative.Comrak.Text{
                     literal: ", ",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 20}, end: {3, 21}}
                   },
                   %MDExNative.Comrak.Emph{
                     nodes: [
                       %MDExNative.Comrak.Text{
                         literal: "emphasis",
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 23}, end: {3, 30}}
                       }
                     ],
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 22}, end: {3, 31}}
                   },
                   %MDExNative.Comrak.Text{
                     literal: ", ",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 32}, end: {3, 33}}
                   },
                   %MDExNative.Comrak.Link{
                     nodes: [
                       %MDExNative.Comrak.Text{
                         literal: "a link",
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 35}, end: {3, 40}}
                       }
                     ],
                     url: "https://example.com",
                     title: "",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 34}, end: {3, 62}}
                   },
                   %MDExNative.Comrak.Text{
                     literal: ", and ",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 63}, end: {3, 68}}
                   },
                   %MDExNative.Comrak.Code{
                     num_backticks: 1,
                     literal: "inline code",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 69}, end: {3, 81}}
                   },
                   %MDExNative.Comrak.Text{
                     literal: ".",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 82}, end: {3, 82}}
                   }
                 ],
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {3, 1}, end: {3, 82}}
               },
               %MDExNative.Comrak.BlockQuote{
                 nodes: [
                   %MDExNative.Comrak.Paragraph{
                     nodes: [
                       %MDExNative.Comrak.Text{
                         literal: "A quote with nested content.",
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {5, 3}, end: {5, 30}}
                       }
                     ],
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {5, 3}, end: {5, 30}}
                   },
                   %MDExNative.Comrak.List{
                     nodes: [
                       %MDExNative.Comrak.ListItem{
                         nodes: [
                           %MDExNative.Comrak.Paragraph{
                             nodes: [
                               %MDExNative.Comrak.Text{
                                 literal: "quoted item",
                                 sourcepos: %MDExNative.Comrak.Sourcepos{
                                   start: {7, 5},
                                   end: {7, 15}
                                 }
                               }
                             ],
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {7, 5}, end: {7, 15}}
                           }
                         ],
                         list_type: :bullet,
                         marker_offset: 0,
                         padding: 2,
                         start: 1,
                         delimiter: :period,
                         bullet_char: "-",
                         tight: false,
                         is_task_list: false,
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {7, 3}, end: {7, 15}}
                       },
                       %MDExNative.Comrak.ListItem{
                         nodes: [
                           %MDExNative.Comrak.Paragraph{
                             nodes: [
                               %MDExNative.Comrak.Text{
                                 literal: "another quoted item",
                                 sourcepos: %MDExNative.Comrak.Sourcepos{
                                   start: {8, 5},
                                   end: {8, 23}
                                 }
                               }
                             ],
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {8, 5}, end: {8, 23}}
                           }
                         ],
                         list_type: :bullet,
                         marker_offset: 0,
                         padding: 2,
                         start: 1,
                         delimiter: :period,
                         bullet_char: "-",
                         tight: false,
                         is_task_list: false,
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {8, 3}, end: {8, 23}}
                       }
                     ],
                     list_type: :bullet,
                     marker_offset: 0,
                     padding: 2,
                     start: 1,
                     delimiter: :period,
                     bullet_char: "-",
                     tight: true,
                     is_task_list: false,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {7, 3}, end: {8, 23}}
                   }
                 ],
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {5, 1}, end: {8, 23}}
               },
               %MDExNative.Comrak.List{
                 nodes: [
                   %MDExNative.Comrak.ListItem{
                     nodes: [
                       %MDExNative.Comrak.Paragraph{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "first",
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {10, 4}, end: {10, 8}}
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {10, 4}, end: {10, 8}}
                       }
                     ],
                     list_type: :ordered,
                     marker_offset: 0,
                     padding: 3,
                     start: 1,
                     delimiter: :period,
                     bullet_char: "",
                     tight: false,
                     is_task_list: false,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {10, 1}, end: {10, 8}}
                   },
                   %MDExNative.Comrak.ListItem{
                     nodes: [
                       %MDExNative.Comrak.Paragraph{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "second",
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {11, 4}, end: {11, 9}}
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {11, 4}, end: {11, 9}}
                       }
                     ],
                     list_type: :ordered,
                     marker_offset: 0,
                     padding: 3,
                     start: 2,
                     delimiter: :period,
                     bullet_char: "",
                     tight: false,
                     is_task_list: false,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {11, 1}, end: {12, 0}}
                   }
                 ],
                 list_type: :ordered,
                 marker_offset: 0,
                 padding: 3,
                 start: 1,
                 delimiter: :period,
                 bullet_char: "",
                 tight: true,
                 is_task_list: false,
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {10, 1}, end: {11, 9}}
               },
               %MDExNative.Comrak.List{
                 nodes: [
                   %MDExNative.Comrak.TaskItem{
                     nodes: [
                       %MDExNative.Comrak.Paragraph{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "shipped",
                             sourcepos: %MDExNative.Comrak.Sourcepos{
                               start: {13, 7},
                               end: {13, 13}
                             }
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {13, 7}, end: {13, 13}}
                       }
                     ],
                     checked: true,
                     marker: "x",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {13, 1}, end: {13, 13}}
                   },
                   %MDExNative.Comrak.TaskItem{
                     nodes: [
                       %MDExNative.Comrak.Paragraph{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "pending",
                             sourcepos: %MDExNative.Comrak.Sourcepos{
                               start: {14, 7},
                               end: {14, 13}
                             }
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {14, 7}, end: {14, 13}}
                       }
                     ],
                     checked: false,
                     marker: "",
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {14, 1}, end: {15, 0}}
                   }
                 ],
                 list_type: :bullet,
                 marker_offset: 0,
                 padding: 2,
                 start: 1,
                 delimiter: :period,
                 bullet_char: "-",
                 tight: true,
                 is_task_list: true,
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {13, 1}, end: {14, 13}}
               },
               %MDExNative.Comrak.CodeBlock{
                 nodes: [],
                 fenced: true,
                 fence_char: "`",
                 fence_length: 3,
                 fence_offset: 0,
                 info: "elixir",
                 literal: "IO.puts(\"hello\")\n",
                 closed: true,
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {16, 1}, end: {18, 3}}
               },
               %MDExNative.Comrak.Table{
                 nodes: [
                   %MDExNative.Comrak.TableRow{
                     nodes: [
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "Name",
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {20, 3}, end: {20, 6}}
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {20, 2}, end: {20, 7}}
                       },
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "Value",
                             sourcepos: %MDExNative.Comrak.Sourcepos{
                               start: {20, 10},
                               end: {20, 14}
                             }
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {20, 9}, end: {20, 15}}
                       }
                     ],
                     header: true,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {20, 1}, end: {20, 16}}
                   },
                   %MDExNative.Comrak.TableRow{
                     nodes: [
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "one",
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {22, 3}, end: {22, 5}}
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {22, 2}, end: {22, 7}}
                       },
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "1",
                             sourcepos: %MDExNative.Comrak.Sourcepos{
                               start: {22, 10},
                               end: {22, 10}
                             }
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {22, 9}, end: {22, 15}}
                       }
                     ],
                     header: false,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {22, 1}, end: {22, 16}}
                   },
                   %MDExNative.Comrak.TableRow{
                     nodes: [
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "two",
                             sourcepos: %MDExNative.Comrak.Sourcepos{start: {23, 3}, end: {23, 5}}
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {23, 2}, end: {23, 7}}
                       },
                       %MDExNative.Comrak.TableCell{
                         nodes: [
                           %MDExNative.Comrak.Text{
                             literal: "2",
                             sourcepos: %MDExNative.Comrak.Sourcepos{
                               start: {23, 10},
                               end: {23, 10}
                             }
                           }
                         ],
                         sourcepos: %MDExNative.Comrak.Sourcepos{start: {23, 9}, end: {23, 15}}
                       }
                     ],
                     header: false,
                     sourcepos: %MDExNative.Comrak.Sourcepos{start: {23, 1}, end: {23, 16}}
                   }
                 ],
                 alignments: [:none, :none],
                 num_columns: 2,
                 num_rows: 3,
                 num_nonempty_cells: 6,
                 sourcepos: %MDExNative.Comrak.Sourcepos{start: {20, 1}, end: {23, 16}}
               }
             ],
             sourcepos: %MDExNative.Comrak.Sourcepos{start: {1, 1}, end: {23, 16}}
           }

    assert MDExNative.Comrak.document_to_html(document, options) ==
             MDExNative.Comrak.markdown_to_html(markdown, options)
  end

  test "parses deeply nested documents without crashing the NIF" do
    markdown = String.duplicate("> ", 5_000) <> "boom"
    assert %MDExNative.Comrak.Document{} = MDExNative.Comrak.parse_document(markdown)
  end

  test "renders deeply nested manually constructed documents without crashing the NIF" do
    text = %MDExNative.Comrak.Text{literal: "boom"}

    nodes = [
      Enum.reduce(1..5_000, text, fn _, child ->
        %MDExNative.Comrak.BlockQuote{nodes: [child]}
      end)
    ]

    document = %MDExNative.Comrak.Document{nodes: nodes}

    assert is_binary(MDExNative.Comrak.document_to_html(document))
  end

  test "parses code fence info with language only" do
    assert MDExNative.Comrak.parse_code_fence_info("elixir") == %{
             language: "elixir",
             metadata: "",
             attributes: %{}
           }
  end

  test "raises when lumis is requested but no syntax highlighter is compiled" do
    error =
      assert_raise RuntimeError, fn ->
        MDExNative.Comrak.markdown_to_html(@code_block_markdown,
          syntax_highlight: [
            engine: :lumis,
            opts: [
              formatter:
                {:html_inline, theme: "catppuccin_macchiato", pre_class: "code-block-example"}
            ]
          ]
        )
      end

    assert error.message =~ "Lumis is not enabled."
    assert error.message =~ "config :mdex_native, syntax_highlighter: :lumis"
  end

  test "raises when syntect is requested but no syntax highlighter is compiled" do
    error =
      assert_raise RuntimeError, fn ->
        MDExNative.Comrak.markdown_to_html(@code_block_markdown,
          syntax_highlight: [engine: :syntect]
        )
      end

    assert error.message =~ "Syntect is not enabled."
    assert error.message =~ "config :mdex_native, syntax_highlighter: :syntect"
  end

  test "does not syntax highlight when syntax_highlight is absent" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end

  test "does not syntax highlight when syntax_highlight is nil" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown, syntax_highlight: nil) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end

  test "does not syntax highlight when syntax_highlight is false" do
    assert MDExNative.Comrak.markdown_to_html(@code_block_markdown, syntax_highlight: false) ==
             "<pre><code class=\"language-elixir\">IO.puts(&quot;Hello&quot;)\n</code></pre>\n"
  end
end

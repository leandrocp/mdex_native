defmodule MDExNativeE2E.LumisRegressionTest do
  use ExUnit.Case

  if Application.compile_env(:mdex_native, :syntax_highlighter) == :lumis do
    @multi_themes {:html_multi_themes,
                   themes: [light: "catppuccin_latte", dark: "catppuccin_mocha"],
                   default_theme: "light-dark()"}

    test "multi-theme options reach the pre attributes (issue #32)" do
      html = render("```elixir\nIO.puts(:hello)\n```", formatter: @multi_themes)

      assert html =~
               "style=\"color: light-dark(#4c4f69, #cdd6f4); background-color: light-dark(#eff1f5, #1e1e2e);\""

      assert Enum.sort(pre_classes(html)) == Enum.sort(["lumis", "lumis-themes", "light", "dark"])
    end

    test "a declared parser highlights, an undeclared one renders plain" do
      declared = highlight("```elixir\nIO.puts(:hello)\n```")
      assert declared =~ "language-elixir"
      # Scoped per token, so the source is split across spans rather than literal.
      assert declared =~ "<span style=\"color: #"

      # `zig` is a language Lumis knows and this project does not depend on.
      # Not fetched, and not an error either: one undeclared fence costs
      # itself, not the document around it.
      undeclared = highlight("```zig\nconst x = 1;\n```")
      assert undeclared =~ "const x = 1;"
      refute undeclared =~ "<span style=\"color: #"
    end

    test "a fence rendered at compile time is highlighted" do
      assert MDExNativeE2E.CompileTimeRender.from_keyword() =~ "<span style=\"color: #"
    end

    test "every option Lumis.highlight/2 takes is accepted" do
      # Deprecated, but it still themes the fence, as it did through Lumis.
      assert render("```elixir\n:ok\n```", theme: "dracula") =~ "background-color: #282a36"

      # A fence decides its own language and has nothing to annotate or budget.
      assert render("```elixir\n:ok\n```",
               language: "rust",
               annotations: [],
               budget: [time_limit: 1000],
               rainbow_brackets: true
             ) =~ "language-elixir"
    end

    test "an invalid Lumis option raises ArgumentError naming it" do
      error =
        assert_raise ArgumentError, fn ->
          render("```elixir\n:ok\n```", formatter: {:html_inline, them: "dracula"})
        end

      assert error.message ==
               "invalid value for :syntax_highlight option: invalid value for :opts option: " <>
                 "invalid value for :formatter option: invalid options given to html_inline: " <>
                 "unknown option :them (did you mean :theme?), valid options are: " <>
                 "[:language, :structure, :theme, :pre_class, :pre_attrs, :code_attrs, :italic, " <>
                 ":include_highlights, :highlight_lines, :line_numbers, :header]"
    end

    test "compiled parsers go where :lumis keeps its own, or in this application without it" do
      expected =
        case :code.priv_dir(:lumis) do
          {:error, :bad_name} -> :code.priv_dir(:mdex_native)
          priv -> priv
        end

      assert MDExNative.Application.data_dir() == Path.join(List.to_string(expected), "lumis")
    end

    if Code.ensure_loaded?(Lumis) do
      # MDEx 0.14.1 and projects that copied it convert options with Lumis
      # 0.10's `Lumis.rust_options!/1` before calling mdex_native.
      test "options converted by Lumis.rust_options!/1 still render" do
        wire = [formatter: @multi_themes] |> Lumis.validate_options!() |> Lumis.rust_options!()

        for syntax_highlight <- [[engine: :lumis, opts: wire], %{engine: :lumis, opts: wire}] do
          html =
            MDExNative.Comrak.markdown_to_html("```elixir\nIO.puts(:hello)\n```",
              syntax_highlight: syntax_highlight
            )

          assert html =~ "light-dark(#4c4f69, #cdd6f4)"
        end

        assert MDExNativeE2E.CompileTimeRender.from_map() =~ "<span style=\"color: #"
      end

      # MDEx publishes flat spans where Lumis nests them, so a fence and
      # Lumis.highlight/2 agree on colour but not on element structure. These are
      # the languages whose scopes never nest, where the two must match exactly.
      @parity_samples [
        {"html", "<div class=\"a\"><script>let x = 1;</script></div>"},
        {"rust", "fn main() {\n    let v: Vec<String> = vec![];\n}"},
        {"json", "{\"a\": [1, 2, {\"b\": null}]}"}
      ]

      test "a fence renders exactly what Lumis.highlight would" do
        {name, formatter_opts} = formatter = {:html_inline, theme: "onedark"}

        for {language, source} <- @parity_samples do
          direct =
            Lumis.highlight!(source, formatter: {name, [language: language] ++ formatter_opts})

          rendered =
            "```#{language}\n#{source}\n```"
            |> render(formatter: formatter)
            |> String.trim_trailing("\n")

          assert direct == rendered, "#{language} diverged from Lumis.highlight/2"
        end
      end

      test "the parsers mdex_native reads are the ones Lumis resolves" do
        assert Lumis.Packages.installed_dirs() != []
      end
    end

    # Compiled modules go under the Lumis data directory, where `:lumis` keeps
    # its own, so a parser is compiled once rather than once per NIF.
    @tag :tmp_dir
    test "modules this NIF compiles go under the Lumis data directory", %{tmp_dir: tmp_dir} do
      compiled = Path.join([tmp_dir, "data", "compiled"])

      output =
        run_before_start("""
        Application.put_env(:lumis, :data_dir, #{inspect(Path.join(tmp_dir, "data"))})
        {:ok, _} = Application.ensure_all_started(:mdex_native)
        MDExNative.Comrak.markdown_to_html("```elixir\\nIO.puts(:hello)\\n```", syntax_highlight: [engine: :lumis])
        IO.write("compiled_in_data_dir=" <> inspect(File.dir?(#{inspect(compiled)})))
        """)

      assert output =~ "compiled_in_data_dir=true"
    end

    # A render with no fence configures the store without building it, and
    # `config/runtime.exs` runs after compilation, before :mdex_native starts.
    # The directory it sets still has to reach the store.
    @tag :tmp_dir
    test "config set after a render that built no store applies when :mdex_native starts", %{
      tmp_dir: tmp_dir
    } do
      compiled = Path.join([tmp_dir, "data", "compiled"])

      output =
        run_before_start("""
        MDExNative.Comrak.parse_document("# Hello")
        Application.put_env(:lumis, :data_dir, #{inspect(Path.join(tmp_dir, "data"))})
        {:ok, _} = Application.ensure_all_started(:mdex_native)
        MDExNative.Comrak.markdown_to_html("```elixir\\nIO.puts(:hello)\\n```", syntax_highlight: [engine: :lumis])
        IO.write("compiled_in_data_dir=" <> inspect(File.dir?(#{inspect(compiled)})))
        """)

      assert output =~ "compiled_in_data_dir=true"
    end

    # The store and its engine are built once per VM, and this suite's VM built
    # them already, so each of these needs a VM of its own.
    defp run_before_start(script) do
      {output, status} =
        System.cmd("mix", ["run", "--no-start", "--no-compile", "-e", script],
          env: [{"MIX_ENV", "test"}],
          stderr_to_stdout: true
        )

      assert status == 0, output
      output
    end

    defp highlight(markdown) do
      MDExNative.Comrak.markdown_to_html(markdown, syntax_highlight: [engine: :lumis])
    end

    defp render(markdown, opts) do
      MDExNative.Comrak.markdown_to_html(markdown,
        render: [unsafe: true],
        syntax_highlight: [engine: :lumis, opts: opts]
      )
    end
  end

  defp pre_classes(html) do
    [_before, pre] = String.split(html, "<pre class=\"", parts: 2)
    [classes | _after] = String.split(pre, "\"", parts: 2)

    String.split(classes, " ")
  end
end

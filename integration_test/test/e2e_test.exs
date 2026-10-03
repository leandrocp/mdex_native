defmodule MDExNative.Integration.E2ETest do
  use ExUnit.Case

  @mdex_repo "https://github.com/leandrocp/mdex.git"

  setup_all do
    File.rm_rf!(workspace_path())
    File.mkdir_p!(workspace_path())
    File.mkdir_p!(cargo_target_path())

    :ok
  end

  test "syntax highlighter compile-time options" do
    for e2e_case <- ~w(default lumis lumis_standalone syntect) do
      native_checkout_path = prepare_native_checkout!("native/#{e2e_case}")
      dummy_app_path = prepare_dummy_app!(e2e_case, native_checkout_path)
      env = e2e_env(e2e_case, native_checkout_path, build_path: "dummy_app/#{e2e_case}")

      run_mix!(dummy_app_path, ["deps.get"], env, label: "dummy_app/#{e2e_case}")
      run_mix!(dummy_app_path, ["compile"], env, label: "dummy_app/#{e2e_case}")
      run_mix!(dummy_app_path, ["test"], env, label: "dummy_app/#{e2e_case}")
    end
  end

  # MDEx's `main` tracks this NIF, and its latest release is what most
  # projects run: a release of this NIF must keep that one working too.
  test "mdex test suite passes against this checkout" do
    native_checkout_path = prepare_native_checkout!("native/lumis")

    for ref <- mdex_refs() do
      # A ref names a directory below; `feature/x` or `..` must not leave it.
      directory = String.replace(ref, ~r/[^A-Za-z0-9._-]|\.\./, "_")
      mdex_path = Path.join([workspace_path(), "mdex", directory])
      build_path = "mdex/#{directory}"
      File.rm_rf!(mdex_path)

      clone = ["clone", "--depth", "1", "--branch", ref, mdex_repo(), mdex_path]
      run!("git", clone, native_path(), [], label: build_path)

      env = e2e_env("lumis", native_checkout_path, build_path: build_path)
      run_mix!(mdex_path, ["deps.get"], env, label: build_path)
      run_mix!(mdex_path, ["compile"], env, label: build_path)
      run_mix!(mdex_path, ["test"], env, label: build_path)
    end
  end

  @tag :cloudflare
  test "precompiled artifact loads from Cloudflare" do
    dummy_app_path = prepare_dummy_app!("cloudflare", :hex)

    env =
      e2e_env("cloudflare", native_path(),
        build_path: "dummy_app/cloudflare",
        force_build: false
      )

    run_mix!(dummy_app_path, ["deps.get"], env, label: "dummy_app/cloudflare")
    run_mix!(dummy_app_path, ["compile"], env, label: "dummy_app/cloudflare")
    run_mix!(dummy_app_path, ["test"], env, label: "dummy_app/cloudflare")
  end

  defp prepare_native_checkout!(name) do
    destination = Path.join(workspace_path(), name)

    if File.dir?(destination) do
      destination
    else
      copy_native_checkout!(destination)
    end
  end

  defp copy_native_checkout!(destination) do
    File.mkdir_p!(destination)

    copy_project_path!("lib", destination)
    copy_project_path!("mix.exs", destination)
    copy_project_path!("README.md", destination)
    copy_project_path!("LICENSE.md", destination)
    copy_project_path!("CHANGELOG.md", destination)
    copy_project_path!("checksum-Elixir.MDExNative.Native.exs", destination)
    copy_project_path!("native/mdex_native_nif/.cargo", destination)
    copy_project_path!("native/mdex_native_nif/src", destination)
    copy_project_path!("native/mdex_native_nif/Cargo.toml", destination)
    copy_project_path!("native/mdex_native_nif/Cargo.lock", destination)
    copy_project_path!("native/mdex_native_nif/Cross.toml", destination)

    destination
  end

  defp prepare_dummy_app!(e2e_case, mdex_native_dep) do
    destination = Path.join([workspace_path(), "dummy_app", e2e_case])

    File.rm_rf!(destination)
    File.mkdir_p!(Path.dirname(destination))
    File.cp_r!(dummy_app_path(), destination)

    mix_exs = Path.join(destination, "mix.exs")

    mix_exs
    |> File.read!()
    |> String.replace(
      ~s({:mdex_native, path: "../../.."}),
      mdex_native_dependency(mdex_native_dep)
    )
    |> then(&File.write!(mix_exs, &1))

    destination
  end

  defp mdex_native_dependency(:hex), do: ~s({:mdex_native, ">= 0.0.0"})
  defp mdex_native_dependency(path), do: ~s({:mdex_native, path: #{inspect(path)}})

  defp copy_project_path!(path, destination) do
    source = Path.join(native_path(), path)

    if File.exists?(source) do
      target = Path.join(destination, path)
      File.mkdir_p!(Path.dirname(target))
      File.cp_r!(source, target)
    end
  end

  defp run_mix!(path, args, env, opts) do
    run!("mix", args, path, env, opts)
  end

  defp run!(command, args, path, env, opts, attempt \\ 1) do
    label = Keyword.get(opts, :label, Path.relative_to_cwd(path))
    command_name = Enum.join([command | args], " ")

    IO.write("e2e[#{label}]: #{Path.relative_to_cwd(path)} $ #{command_name}\n")

    started_at = System.monotonic_time()

    {output, status} =
      System.cmd(command, args,
        cd: path,
        env: env,
        stderr_to_stdout: true
      )

    elapsed_ms =
      started_at
      |> then(&(System.monotonic_time() - &1))
      |> System.convert_time_unit(:native, :millisecond)

    if status != 0 do
      if attempt < 3 do
        IO.write("e2e[#{label}]: retrying #{command_name} after failed attempt #{attempt}\n")
        Process.sleep(:timer.seconds(attempt))
        run!(command, args, path, env, opts, attempt + 1)
      else
        flunk("#{command_name} failed in #{path} after #{elapsed_ms}ms\n\n#{output}")
      end
    else
      IO.write("e2e[#{label}]: completed #{command_name} in #{elapsed_ms}ms\n")

      output
    end
  end

  defp e2e_env(e2e_case, native_checkout_path, opts) do
    env = [
      {"MDEX_NATIVE_E2E_CASE", e2e_case},
      {"MDEX_NATIVE_PATH", native_checkout_path},
      {"CARGO_TARGET_DIR", Path.join(cargo_target_path(), e2e_case)},
      {"MIX_BUILD_PATH", Path.join([workspace_path(), "_build", opts[:build_path]])},
      {"MIX_DEPS_PATH", Path.join([workspace_path(), "deps", opts[:build_path]])}
    ]

    if Keyword.get(opts, :force_build, true) do
      [{"MDEX_NATIVE_BUILD", "1"} | env]
    else
      env
    end
  end

  defp mdex_repo do
    System.get_env("MDEX_NATIVE_E2E_MDEX_REPO", @mdex_repo)
  end

  # MDEx tracks this NIF's output, so a change that moves it has to name the
  # branch that adopted it or this clones a main that predates the change.
  # `MDEX_NATIVE_E2E_MDEX_REFS` lists refs to test, `latest` naming the newest
  # release tag; `MDEX_NATIVE_E2E_MDEX_REF` still picks a single one.
  defp mdex_refs do
    refs =
      case System.get_env("MDEX_NATIVE_E2E_MDEX_REF") do
        ref when ref not in [nil, ""] -> ref
        _ -> System.get_env("MDEX_NATIVE_E2E_MDEX_REFS", "main latest")
      end

    refs
    |> String.split([" ", ","], trim: true)
    |> Enum.map(fn
      "latest" -> latest_mdex_release()
      ref -> ref
    end)
    |> Enum.uniq()
  end

  defp latest_mdex_release do
    {output, status} =
      System.cmd("git", ["ls-remote", "--tags", "--refs", mdex_repo(), "v*"],
        stderr_to_stdout: true
      )

    if status != 0, do: flunk("git ls-remote failed for #{mdex_repo()}:\n\n#{output}")

    output
    |> String.split("\n", trim: true)
    |> Enum.map(&(&1 |> String.split("refs/tags/") |> List.last()))
    |> Enum.flat_map(fn tag ->
      case Version.parse(String.trim_leading(tag, "v")) do
        {:ok, %Version{pre: []} = version} -> [{version, tag}]
        _ -> []
      end
    end)
    |> case do
      [] -> flunk("no release tag v* found in #{mdex_repo()}")
      releases -> releases |> Enum.max_by(&elem(&1, 0), Version) |> elem(1)
    end
  end

  defp native_path do
    Path.expand("../..", __DIR__)
  end

  defp dummy_app_path do
    Path.join(integration_path(), "fixtures/dummy_app")
  end

  defp integration_path do
    Path.expand("..", __DIR__)
  end

  defp tmp_path do
    Path.join(integration_path(), "tmp")
  end

  defp workspace_path do
    Path.join(tmp_path(), "workspace")
  end

  defp cargo_target_path do
    Path.join(tmp_path(), "cargo_target")
  end
end

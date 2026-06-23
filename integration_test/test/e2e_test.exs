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
    for e2e_case <- ~w(default lumis syntect) do
      native_checkout_path = prepare_native_checkout!("native/#{e2e_case}")
      dummy_app_path = prepare_dummy_app!(e2e_case, native_checkout_path)
      env = e2e_env(e2e_case, native_checkout_path, build_path: "dummy_app/#{e2e_case}")

      run_mix!(dummy_app_path, ["deps.get"], env, label: "dummy_app/#{e2e_case}")
      run_mix!(dummy_app_path, ["compile"], env, label: "dummy_app/#{e2e_case}")
      run_mix!(dummy_app_path, ["test"], env, label: "dummy_app/#{e2e_case}")
    end
  end

  test "mdex test suite passes against this checkout" do
    mdex_path = Path.join(workspace_path(), "mdex")
    native_checkout_path = prepare_native_checkout!("native/lumis")

    File.rm_rf!(mdex_path)

    run!("git", ["clone", "--depth", "1", mdex_repo(), mdex_path], native_path(), [],
      label: "mdex"
    )

    env = e2e_env("lumis", native_checkout_path, build_path: "mdex")

    run_mix!(mdex_path, ["deps.get"], env, label: "mdex")
    run_mix!(mdex_path, ["compile"], env, label: "mdex")
    run_mix!(mdex_path, ["test"], env, label: "mdex")
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

  defp prepare_dummy_app!(e2e_case, native_checkout_path) do
    destination = Path.join([workspace_path(), "dummy_app", e2e_case])

    File.rm_rf!(destination)
    File.mkdir_p!(Path.dirname(destination))
    File.cp_r!(dummy_app_path(), destination)

    mix_exs = Path.join(destination, "mix.exs")

    mix_exs
    |> File.read!()
    |> String.replace(
      ~s({:mdex_native, path: "../../.."}),
      ~s({:mdex_native, path: #{inspect(native_checkout_path)}})
    )
    |> then(&File.write!(mix_exs, &1))

    destination
  end

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
    [
      {"MDEX_NATIVE_E2E_CASE", e2e_case},
      {"MDEX_NATIVE_PATH", native_checkout_path},
      {"MDEX_NATIVE_BUILD", "1"},
      {"CARGO_TARGET_DIR", Path.join(cargo_target_path(), e2e_case)},
      {"MIX_BUILD_PATH", Path.join([workspace_path(), "_build", opts[:build_path]])},
      {"MIX_DEPS_PATH", Path.join([workspace_path(), "deps", opts[:build_path]])}
    ]
  end

  defp mdex_repo do
    System.get_env("MDEX_NATIVE_E2E_MDEX_REPO", @mdex_repo)
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

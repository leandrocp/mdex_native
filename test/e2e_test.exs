defmodule MDExNative.E2ETest do
  use ExUnit.Case

  @moduletag :e2e
  @moduletag timeout: :infinity

  @mdex_repo "https://github.com/leandrocp/mdex.git"

  test "syntax highlighter compile-time options" do
    for e2e_case <- ~w(default lumis syntect) do
      env = e2e_env(e2e_case, build_path: "dummy_app/#{e2e_case}")

      run_mix!(dummy_app_path(), ["deps.get"], env)
      run_mix!(dummy_app_path(), ["test"], env)
    end
  end

  test "mdex test suite passes against this checkout" do
    mdex_path = Path.join(tmp_path(), "mdex")

    File.rm_rf!(mdex_path)
    File.mkdir_p!(tmp_path())

    run!("git", ["clone", "--depth", "1", mdex_repo(), mdex_path], native_path(), [])

    env = e2e_env("default", build_path: "mdex")

    run_mix!(mdex_path, ["deps.get"], env)
    run_mix!(mdex_path, ["test"], env)
  end

  defp run_mix!(path, args, env) do
    run!("mix", args, path, env)
  end

  defp run!(command, args, path, env) do
    {output, status} =
      System.cmd(command, args,
        cd: path,
        env: env,
        stderr_to_stdout: true
      )

    if status != 0 do
      flunk("#{command} #{Enum.join(args, " ")} failed in #{path}\n\n#{output}")
    end

    output
  end

  defp e2e_env(e2e_case, opts) do
    [
      {"MDEX_NATIVE_E2E_CASE", e2e_case},
      {"MDEX_NATIVE_PATH", native_path()},
      {"MDEX_NATIVE_BUILD", "1"},
      {"MIX_BUILD_PATH", Path.join([tmp_path(), "_build", opts[:build_path]])},
      {"MIX_DEPS_PATH", Path.join([tmp_path(), "deps", opts[:build_path]])}
    ]
  end

  defp mdex_repo do
    System.get_env("MDEX_NATIVE_E2E_MDEX_REPO", @mdex_repo)
  end

  defp native_path do
    Path.expand("..", __DIR__)
  end

  defp dummy_app_path do
    Path.join(native_path(), "e2e/dummy_app")
  end

  defp tmp_path do
    Path.join(native_path(), "tmp/e2e")
  end
end

defmodule MDExNative.ApplicationTest do
  # Reads and writes the `:lumis` app env and `LUMIS_DATA_DIR`.
  use ExUnit.Case, async: false

  alias MDExNative.Application, as: App

  setup do
    env = System.get_env("LUMIS_DATA_DIR")
    System.delete_env("LUMIS_DATA_DIR")

    on_exit(fn ->
      Application.delete_env(:lumis, :data_dir)
      if env, do: System.put_env("LUMIS_DATA_DIR", env), else: System.delete_env("LUMIS_DATA_DIR")
    end)
  end

  describe "data_dir/0 picks the directory :lumis would" do
    test "config :lumis, :data_dir first" do
      Application.put_env(:lumis, :data_dir, "tmp/lumis-data")
      System.put_env("LUMIS_DATA_DIR", "/elsewhere")

      assert App.data_dir() == Path.expand("tmp/lumis-data")
    end

    test "then LUMIS_DATA_DIR, which the NIF reads itself" do
      System.put_env("LUMIS_DATA_DIR", "/elsewhere")

      assert App.data_dir() == nil
    end

    test "an empty LUMIS_DATA_DIR names no directory" do
      System.put_env("LUMIS_DATA_DIR", "")

      assert App.data_dir() == Path.join(List.to_string(:code.priv_dir(:mdex_native)), "lumis")
    end

    test "this application's priv when :lumis is not installed" do
      assert {:error, :bad_name} = :code.priv_dir(:lumis)
      assert App.data_dir() == Path.join(List.to_string(:code.priv_dir(:mdex_native)), "lumis")
    end
  end
end

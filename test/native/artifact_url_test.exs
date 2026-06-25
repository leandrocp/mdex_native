defmodule MDExNative.Native.ArtifactURLTest do
  use ExUnit.Case, async: true

  alias MDExNative.Native.ArtifactURL

  @version Mix.Project.config()[:version]
  @file_name "artifact.tar.gz"

  test "defaults to GitHub releases" do
    assert ArtifactURL.url(@file_name) ==
             "https://github.com/leandrocp/mdex_native/releases/download/v#{@version}/#{@file_name}"
  end

  test "can use Cloudflare R2 custom domain" do
    assert ArtifactURL.url(@file_name, :cloudflare) ==
             "https://artifacts.mdelixir.dev/releases/download/v#{@version}/#{@file_name}"
  end
end

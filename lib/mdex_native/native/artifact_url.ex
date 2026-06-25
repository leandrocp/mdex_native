defmodule MDExNative.Native.ArtifactURL do
  @moduledoc false

  @version Mix.Project.config()[:version]
  @github_base_url "https://github.com/leandrocp/mdex_native/releases/download/v#{@version}"
  @cloudflare_base_url "https://artifacts.mdelixir.dev/releases/download/v#{@version}"
  @source Application.compile_env(:mdex_native, :artifact_source, :github)

  if @source not in [:github, :cloudflare] do
    raise ArgumentError, "invalid mdex_native artifact source: #{inspect(@source)}"
  end

  def url(file_name), do: url(file_name, @source)
  def url(file_name, :github = _source), do: "#{@github_base_url}/#{file_name}"
  def url(file_name, :cloudflare = _source), do: "#{@cloudflare_base_url}/#{file_name}"
end

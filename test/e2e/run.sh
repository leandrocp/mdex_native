#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"

restore_default_nif() {
  status=$?
  set +e
  cd "$repo_root"
  MDEX_NATIVE_BUILD=1 MIX_ENV=test mix compile --force >/dev/null
  exit "$status"
}

trap restore_default_nif EXIT

cd "$repo_root/e2e/dummy_app"

for scenario in default lumis syntect; do
  echo "==> e2e: $scenario"
  rm -rf "$repo_root/.e2e/$scenario"
  MIX_ENV=test \
    MIX_BUILD_PATH="$repo_root/.e2e/$scenario/_build" \
    MIX_DEPS_PATH="$repo_root/.e2e/$scenario/deps" \
    MDEX_NATIVE_BUILD=1 \
    MDEX_NATIVE_E2E_CASE="$scenario" \
    mix do deps.get + deps.compile + test
done

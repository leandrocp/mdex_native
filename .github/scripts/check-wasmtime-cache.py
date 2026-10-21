import sys
import tomllib
from pathlib import Path


def wasmtime_version(path):
    with Path(path).open("rb") as lockfile:
        packages = tomllib.load(lockfile).get("package", [])
    matches = [p for p in packages if p.get("name") == "wasmtime"]
    if len(matches) != 1 or not matches[0].get("version"):
        raise ValueError(f"{path}: expected exactly one versioned wasmtime package")
    return matches[0]["version"]


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("Usage: check-wasmtime-cache.py MDEX_LOCK LUMIS_LOCK")
    try:
        mdex_version, lumis_version = map(wasmtime_version, sys.argv[1:])
    except (OSError, ValueError) as error:
        sys.exit(str(error))
    if mdex_version != lumis_version:
        sys.exit(
            "Compiled-cache version mismatch:\n"
            f"  mdex_native:    Wasmtime {mdex_version}\n"
            f"  targeted Lumis: Wasmtime {lumis_version}\n"
            "Align the mdex_native lockfile or select a compatible Lumis release.\n"
            "Do not tighten public dependency requirements to satisfy this check."
        )
    print(f"Wasmtime {mdex_version} in both release-pair lockfiles")

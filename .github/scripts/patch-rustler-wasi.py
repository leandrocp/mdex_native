"""Patch Rustler so the NIF builds for wasm32-wasip1-threads, as ampoule does.

Rustler 0.39 refuses targets that are neither unix nor windows. Ampoule copies
it, makes WASI take the Windows path (the enif_* API arrives at load time,
not through dlsym), and points the crate at the copy with [patch.crates-io].
These are the edits of `patch_rustler` in ampoule's runner/src/nif.rs, on the
rustler-0.39 branch of leandrocp/ampoule.

Usage: patch-rustler-wasi.py CRATE_DIR PATCH_DIR
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

EDITS = {
    "src/sys/functions.rs": [
        (
            'any(windows, all(unix, target_pointer_width = "32"))',
            'any(windows, target_os = "wasi", all(unix, target_pointer_width = "32"))',
        ),
        (
            "#[cfg(not(any(unix, windows)))]",
            '#[cfg(not(any(unix, windows, target_os = "wasi")))]',
        ),
    ],
    "src/lib.rs": [
        (
            '#[cfg(not(panic = "unwind"))]',
            '#[cfg(all(not(panic = "unwind"), not(target_os = "wasi")))]',
        ),
    ],
    "src/sys/types.rs": [
        (
            "#[cfg(unix)]\npub type ErlNifEvent = os::unix::io::RawFd;",
            "#[cfg(unix)]\npub type ErlNifEvent = os::unix::io::RawFd;\n\n"
            '#[cfg(target_os = "wasi")]\npub type ErlNifEvent = std::os::raw::c_int;',
        ),
    ],
}


def main(crate, dest):
    metadata = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        cwd=crate,
        check=True,
        capture_output=True,
        text=True,
    )
    [rustler] = [
        p for p in json.loads(metadata.stdout)["packages"] if p["name"] == "rustler"
    ]
    shutil.copytree(Path(rustler["manifest_path"]).parent, dest)

    for file, edits in EDITS.items():
        path = dest / file
        text = path.read_text()
        for old, new in edits:
            if old not in text:
                sys.exit(f"{path}: no {old!r}; Rustler's layout changed")
            text = text.replace(old, new, 1)
        path.write_text(text)

    with (crate / "Cargo.toml").open("a") as manifest:
        manifest.write(f'\n[patch.crates-io]\nrustler = {{ path = "{dest.as_posix()}" }}\n')


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class WasmtimeCacheTest(unittest.TestCase):
    def test_requires_one_identical_version_in_each_lockfile(self):
        script = Path(__file__).with_name("check-wasmtime-cache.py")
        cases = [
            (["48.0.5"], ["48.0.5"], True),
            (["48.0.6"], ["48.0.5"], False),
            ([], ["48.0.5"], False),
            (["48.0.5"], [], False),
            (["48.0.5", "48.0.6"], ["48.0.5"], False),
            (["48.0.5"], ["48.0.5", "48.0.6"], False),
        ]
        with tempfile.TemporaryDirectory() as directory:
            paths = [Path(directory) / name for name in ("mdex.lock", "lumis.lock")]
            for mdex, lumis, succeeds in cases:
                with self.subTest(mdex=mdex, lumis=lumis):
                    for path, versions in zip(paths, (mdex, lumis)):
                        path.write_text(
                            'version = 4\n[[package]]\nname = "other"\nversion = "99.0.0"\n'
                            + "".join(
                                f'[[package]]\nname = "wasmtime"\nversion = "{version}"\n'
                                for version in versions
                            )
                        )
                    result = subprocess.run(
                        [sys.executable, str(script), *map(str, paths)],
                        capture_output=True,
                        text=True,
                    )
                    self.assertEqual(result.returncode, 0 if succeeds else 1, result.stderr)
                    if succeeds:
                        self.assertIn("Wasmtime 48.0.5 in both", result.stdout)
                    elif mdex == ["48.0.6"]:
                        self.assertIn("version mismatch", result.stderr)
                        self.assertIn("48.0.6", result.stderr)
                        self.assertIn("48.0.5", result.stderr)

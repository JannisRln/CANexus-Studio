"""Integration tests for the repository-managed Rust pre-commit hook."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

HOOK = Path(__file__).resolve().parents[1] / ".githooks" / "pre-commit"


@unittest.skipUnless(os.name == "posix", "Test hook execution via POSIX shell")
class RustfmtHookTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()
        fake_cargo = self.bin_dir / "cargo"
        fake_cargo.write_text(
            """#!/bin/sh
if [ "$MOCK_CARGO_FAIL" = '1' ]; then
    exit 7
fi
case "$*" in
    'fmt --all')
        if grep -q '^unformatted$' src/lib.rs; then
            printf 'formatted\\n' > src/lib.rs
        fi
        ;;
    'fmt --all -- --check')
        if grep -q '^unformatted$' src/lib.rs; then
            exit 1
        fi
        ;;
    *)
        echo 'unexpected cargo arguments' >&2
        exit 9
        ;;
esac
""",
            encoding="utf-8",
        )
        fake_cargo.chmod(0o755)
        self.env = os.environ.copy()
        self.env["PATH"] = f"{self.bin_dir}{os.pathsep}{self.env['PATH']}"
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "hook-tests@example.invalid")
        self.git("config", "user.name", "Hook Tests")
        self.git("config", "core.hooksPath", ".githooks")
        hook_dir = self.root / ".githooks"
        hook_dir.mkdir()
        shutil.copy2(HOOK, hook_dir / "pre-commit")
        (hook_dir / "pre-commit").chmod(0o755)
        (self.root / "src").mkdir()
        (self.root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")

    def git(self, *args, env=None):
        return subprocess.run(
            ["git", *args], cwd=self.root, env=env or self.env,
            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )

    def stage_and_commit(self, message="test", env=None):
        self.git("add", "Cargo.toml", "src/lib.rs")
        return self.git("commit", "-m", message, env=env)

    def test_clean_formatted_file_commits(self):
        (self.root / "src/lib.rs").write_text("formatted\n", encoding="utf-8")
        result = self.stage_and_commit()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.git("show", "HEAD:src/lib.rs").stdout, "formatted\n")

    def test_formats_but_requires_explicit_restaging(self):
        source = self.root / "src/lib.rs"
        source.write_text("unformatted\n", encoding="utf-8")
        result = self.stage_and_commit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("git add", result.stderr)
        self.assertEqual(source.read_text(encoding="utf-8"), "formatted\n")
        self.assertEqual(self.git("show", ":src/lib.rs").stdout, "unformatted\n")
        self.git("add", "src/lib.rs")
        result = self.git("commit", "-m", "formatted")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.git("show", "HEAD:src/lib.rs").stdout, "formatted\n")

    def test_detects_partially_staged_rust_file(self):
        source = self.root / "src/lib.rs"
        source.write_text("formatted\n", encoding="utf-8")
        self.assertEqual(self.stage_and_commit("initial").returncode, 0)
        source.write_text("unformatted\n", encoding="utf-8")
        self.git("add", "src/lib.rs")
        source.write_text("formatted\n", encoding="utf-8")
        result = self.git("commit", "-m", "bad-index")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unstaged Rust changes", result.stderr)
        self.assertEqual(self.git("show", "HEAD:src/lib.rs").stdout, "formatted\n")

    def test_propagates_cargo_failure(self):
        (self.root / "src/lib.rs").write_text("formatted\n", encoding="utf-8")
        env = {**self.env, "MOCK_CARGO_FAIL": "1"}
        result = self.stage_and_commit(env=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotEqual(self.git("rev-parse", "HEAD").returncode, 0)

    def test_skips_non_rust_repositories(self):
        (self.root / "src/lib.rs").write_text("formatted\n", encoding="utf-8")
        (self.root / "Cargo.toml").unlink()
        self.git("add", "src/lib.rs")
        result = self.git("commit", "-m", "without workspace", env={**self.env, "MOCK_CARGO_FAIL": "1"})
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()

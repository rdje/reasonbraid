"""Locality and cache-integrity controls for the project command launcher."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("project_env", REPO / "scripts/project_env.py")
project_env = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(project_env)


class ProjectEnvironmentTests(unittest.TestCase):
    def setUp(self):
        scratch = project_env.local_directory(REPO, "target/project-env-tests")
        self.workspace = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.workspace.cleanup)
        self.base = Path(self.workspace.name)
        self.root = self.base / "repo"
        self.root.mkdir()

    def cache_fixture(self):
        home = self.base / "shared"
        index_name = "index.crates.io-fixture"
        index = home / "registry/index" / index_name
        archive = home / "registry/cache" / index_name / "example-1.0.0.crate"
        record = index / ".cache/ex/am/example"
        archive.parent.mkdir(parents=True)
        record.parent.mkdir(parents=True)
        archive.write_bytes(b"the locked archive")
        record.write_bytes(b"sparse index fixture")
        (index / "config.json").write_text('{"dl":"https://static.crates.io/crates"}')
        checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
        (self.root / "Cargo.lock").write_text(
            'version = 4\n[[package]]\nname = "example"\nversion = "1.0.0"\n'
            f'source = "{project_env.CRATES_IO}"\nchecksum = "{checksum}"\n'
        )
        return home, archive, record

    def test_ambient_stores_are_replaced_and_relocation_uses_new_root(self):
        ambient = {key: "/unowned-store" for key in project_env.STORES}
        ambient.update({"PATH": os.defpath, "RUSTC_WRAPPER": "/unowned-wrapper"})
        env = project_env.project_environment(self.root, ambient, self.base / "compiler")
        for key in project_env.STORES:
            self.assertTrue(Path(env[key]).is_relative_to(self.root))
            self.assertEqual(Path(env[key]).stat().st_dev, self.root.stat().st_dev)
        self.assertEqual(env["RUSTC_WRAPPER"], "")
        moved = self.base / "relocated repo"
        self.root.rename(moved)
        relocated = project_env.project_environment(moved, env, self.base / "compiler")
        for key in project_env.STORES:
            self.assertTrue(Path(relocated[key]).is_relative_to(moved))
        self.assertFalse(self.root.exists())

    def test_symlink_store_is_refused_before_writing_outside_root(self):
        outside = self.base / "outside"
        outside.mkdir()
        (self.root / ".project-data").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            project_env.project_environment(self.root, {}, self.base / "compiler")
        self.assertEqual(list(outside.iterdir()), [])
        with self.assertRaisesRegex(ValueError, "repository-relative"):
            project_env.local_directory(self.root, "../escape")
        self.assertFalse((self.base / "escape").exists())

    def test_seed_verifies_content_and_preserves_shared_source(self):
        home, archive, record = self.cache_fixture()
        originals = {p: p.read_bytes() for p in home.rglob("*") if p.is_file()}
        result = project_env.seed_cargo_cache(self.root, home)
        self.assertEqual(result["files"], 3)
        receipt = json.loads((self.root / result["receipt"]).read_text())
        self.assertEqual(receipt["bytes"], sum(len(data) for data in originals.values()))
        for item in receipt["records"]:
            self.assertFalse(Path(item["path"]).is_absolute())
            self.assertEqual(project_env.digest(self.root / item["path"]), item["sha256"])
        self.assertEqual({p: p.read_bytes() for p in originals}, originals)
        self.assertTrue(archive.exists())
        self.assertTrue(record.exists())
        self.assertEqual(project_env.seed_cargo_cache(self.root, home), result)

    def test_corrupt_archive_is_refused_before_any_cache_publication(self):
        home, archive, _ = self.cache_fixture()
        archive.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "corrupt"):
            project_env.seed_cargo_cache(self.root, home)
        self.assertFalse((self.root / ".project-data").exists())
        self.assertEqual(archive.read_bytes(), b"corrupt")

    def test_symlink_index_is_refused_without_copying_its_target(self):
        home, _, record = self.cache_fixture()
        target = self.base / "unrelated"
        target.write_bytes(b"not registry data")
        record.unlink()
        record.symlink_to(target)
        with self.assertRaisesRegex(ValueError, "symlink"):
            project_env.seed_cargo_cache(self.root, home)
        self.assertFalse((self.root / ".project-data").exists())
        self.assertEqual(target.read_bytes(), b"not registry data")


if __name__ == "__main__":
    unittest.main()

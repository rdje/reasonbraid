"""Exercise the actual acceptance checker in isolated, repository-local Git indexes."""

from pathlib import Path
import importlib.util
import os
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/check_task_acceptance.sh"
SPEC = importlib.util.spec_from_file_location("project_env", ROOT / "scripts/project_env.py")
project_env = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(project_env)
# Synthetic checker inputs, never evidence for a real project implementation.
EVIDENCE = "\n".join(
    f"- [x] **{label}** — fixture command rc=0"
    for label in ("ROOT CAUSE", "ADDRESSED", "NO REGRESSION")
) + "\n"


class TaskAcceptanceTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(ROOT, "target/doctrine_scratch/task-acceptance")
        self.assertEqual(parent.stat().st_dev, ROOT.stat().st_dev)
        self.temporary = tempfile.TemporaryDirectory(prefix="case-", dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        scratch = self.repo / "scratch"
        scratch.mkdir()
        config = self.repo / "empty.gitconfig"
        config.write_text("")
        templates = self.repo / "empty.templates"
        templates.mkdir()
        self.env = {
            key: value for key, value in os.environ.items()
            if not key.startswith("GIT_")
        }
        self.env.update({
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": str(config),
            "TMPDIR": str(scratch),
        })
        self.git("-c", f"init.templateDir={templates}", "init", "-q", ".")

    def git(self, *args):
        return subprocess.run(
            ["git", *args], cwd=self.repo, env=self.env,
            check=True, capture_output=True, text=True,
        )

    def stage(self, files):
        for name, text in files.items():
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.git("add", "--", *files)

    def check(self, expected, message=None):
        result = subprocess.run(
            ["bash", str(CHECKER)], cwd=self.repo, env=self.env,
            capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        if message:
            self.assertIn(message, result.stdout + result.stderr)
        self.assertEqual(list((self.repo / "scratch").iterdir()), [])

    def test_real_tree_with_nested_evidence_is_accepted(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/VALID.md": EVIDENCE,
            "docs/tasks/artifacts/review/INDEX.md": "Evidence index.\n",
            "docs/tasks/artifacts/review/measurement.md": "Measurement detail.\n",
        })
        self.check(0, "task-acceptance: OK")

    def test_nested_evidence_cannot_be_the_only_owner(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/artifacts/review/measurement.md": EVIDENCE,
        })
        self.check(1, "NO owning task-tree leaf")

    def test_unrelated_valid_tree_cannot_supply_missing_boxes(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/VALID.md": EVIDENCE,
            "docs/tasks/EMPTY.md": "No checklist.\n",
        })
        self.check(1, "docs/tasks/EMPTY.md has no 'ROOT CAUSE' box")

    def test_unticked_box_is_refused_despite_nested_evidence(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/UNCHECKED.md": EVIDENCE.replace("[x]", "[ ]", 1),
            "docs/tasks/artifacts/review/measurement.md": EVIDENCE,
        })
        self.check(1, "present but NOT ticked")

    def test_evidence_in_unrelated_prose_cannot_supply_a_box(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/PROSE.md": EVIDENCE.replace("fixture command rc=0", "unmeasured", 1)
            + "\nUnrelated prose: rc=0\n",
        })
        self.check(1, "ticked but carries no tool-output evidence")

    def test_template_is_not_an_owner(self):
        self.stage({"src/main.rs": "fn main() {}\n", "docs/tasks/TEMPLATE.md": EVIDENCE})
        self.check(1, "NO owning task-tree leaf")

    def test_docs_only_nested_evidence_does_not_require_code_acceptance(self):
        self.stage({"docs/tasks/artifacts/review/measurement.md": "Evidence.\n"})
        self.check(0)


if __name__ == "__main__":
    unittest.main()

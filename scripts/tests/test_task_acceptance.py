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
#
# ⚠️ These fixtures changed shape with `SIGNOFF-REPAIR.11.2.6`. The gate no longer asks
# "does this FILE contain a checklist"; it asks "does the leaf THIS COMMIT CLOSES answer
# every question". So a fixture needs a leaf heading and a `Status: done` line, and the
# old ones — three boxes and nothing else — now close no leaf at all.


def leaf(leaf_id, *, answers=True, evidence=True, ticked=True, level=3):
    """One leaf section that CLOSES in this commit."""
    tick = "[x]" if ticked else "[ ]"
    proof = " — cargo test rc=0" if evidence else " — not run"
    body = ""
    if answers:
        body = "\n".join(
            f"- {tick} **{label}**{proof}"
            for label in ("ROOT CAUSE", "ADDRESSED", "NO REGRESSION")
        ) + "\n"
    return f"{'#' * level} {leaf_id} — a fixture leaf\n{body}- Status: `done`.\n"


def open_leaf(leaf_id, *, level=3):
    """A leaf that is worked on but does NOT close here."""
    return f"{'#' * level} {leaf_id} — a fixture leaf\n- Opened: `pending`.\n- Status: `pending`.\n"


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

    def commit(self, files, message="base"):
        """Land `files` as a commit, so a later stage produces a real diff."""
        self.stage(files)
        self.git("-c", "user.email=f@x", "-c", "user.name=f", "commit", "-q", "-m", message)

    def stage(self, files):
        for name, text in files.items():
            path = self.repo / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.git("add", "--", *files)

    def check(self, expected, message=None):
        result = subprocess.run(
            ["bash", str(CHECKER)], cwd=self.repo, env=self.env,
            capture_output=True, text=True, timeout=20,
        )
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        if message:
            self.assertIn(message, result.stdout + result.stderr)
        self.assertEqual(list((self.repo / "scratch").iterdir()), [])

    def test_the_closing_leaf_with_a_full_checklist_is_accepted(self):
        self.stage({"src/main.rs": "fn main() {}\n", "docs/tasks/VALID.md": leaf("PROJ.1")})
        self.check(0, "task-acceptance: OK")

    def test_the_leaf_this_commit_closes_is_the_one_asked(self):
        """🔴 THE REGRESSION TEST FOR THE DEFECT (`SIGNOFF-REPAIR.11.2.6`).

        `PROJ.1` closed in an earlier commit and carries a complete checklist. `PROJ.2`
        closes NOW and carries none. The replaced extractor read the file's first box —
        `PROJ.1`'s — and reported green. The gate must read `PROJ.2`.
        """
        self.commit({"docs/tasks/TREE.md": leaf("PROJ.1")})
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/TREE.md": leaf("PROJ.1") + leaf("PROJ.2", answers=False),
        })
        self.check(1, "leaf PROJ.2 closes in this commit but no bullet answers")

    def test_a_parent_lane_does_not_need_its_own_checklist(self):
        """A commit closes 2.75 leaves on average: the parent closes with its child.

        Only the DEEPEST is asked, so a parent lane carrying no checklist is fine as
        long as the child that closes with it answers everything.
        """
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/TREE.md": leaf("PROJ.2", answers=False, level=3)
            + leaf("PROJ.2.1", level=4),
        })
        self.check(0, "task-acceptance: OK")

    def test_a_parent_cannot_supply_its_childs_evidence(self):
        """The inverse of the above, so the rule is not passing for a free reason."""
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/TREE.md": leaf("PROJ.2", level=3)
            + leaf("PROJ.2.1", answers=False, level=4),
        })
        self.check(1, "leaf PROJ.2.1 closes in this commit but no bullet answers")

    def test_a_commit_that_closes_no_leaf_is_not_checked(self):
        self.stage({"src/main.rs": "fn main() {}\n", "docs/tasks/TREE.md": open_leaf("PROJ.3")})
        self.check(0, "NOT CHECKED")

    def test_an_unrelated_tree_cannot_supply_the_answers(self):
        self.commit({"docs/tasks/OTHER.md": "placeholder\n"})
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/OTHER.md": leaf("OTHER.1"),
            "docs/tasks/TREE.md": leaf("PROJ.2", answers=False),
        })
        self.check(1, "leaf PROJ.2 closes in this commit but no bullet answers")

    def test_an_unticked_box_answers_nothing(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/TREE.md": leaf("PROJ.1", ticked=False),
        })
        self.check(1, "no bullet answers")

    def test_a_closing_leaf_must_cite_tool_output(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/TREE.md": leaf("PROJ.1", evidence=False),
        })
        self.check(1, "cites no tool output")

    def test_the_projects_own_spellings_are_accepted(self):
        """The label families are a seam, and the default set is wide enough for prose.

        `REPRODUCED` / `THE REPAIR` / `NO REGRESSION` is how this corpus actually writes
        the checklist — 142 / 192 / 203 uses against `ROOT CAUSE` 13 and `ADDRESSED` 14.
        """
        prose = (
            "### PROJ.4 — a leaf written in prose\n"
            "- REPRODUCED: the probe returned rc=1 before the change.\n"
            "- THE REPAIR: bind the identifier; test result: ok. 9 passed\n"
            "- NO REGRESSION: the neighbouring suite is unchanged, rc=0.\n"
            "- Status: `done`.\n"
        )
        self.stage({"src/main.rs": "fn main() {}\n", "docs/tasks/TREE.md": prose})
        self.check(0, "task-acceptance: OK")

    def test_nested_evidence_cannot_be_the_only_owner(self):
        self.stage({
            "src/main.rs": "fn main() {}\n",
            "docs/tasks/artifacts/review/measurement.md": leaf("PROJ.1"),
        })
        self.check(1, "NO owning task-tree leaf")

    def test_template_is_not_an_owner(self):
        self.stage({"src/main.rs": "fn main() {}\n", "docs/tasks/TEMPLATE.md": leaf("PROJ.1")})
        self.check(1, "NO owning task-tree leaf")

    def test_docs_only_change_is_not_governed(self):
        self.stage({"docs/tasks/artifacts/review/measurement.md": "Evidence.\n"})
        self.check(0)

    def test_the_scripts_own_self_test_passes(self):
        result = subprocess.run(
            ["bash", str(CHECKER), "--self-test"], cwd=ROOT,
            capture_output=True, text=True, timeout=20,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("SELF-TEST:", result.stdout)


if __name__ == "__main__":
    unittest.main()

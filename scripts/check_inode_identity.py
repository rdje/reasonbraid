#!/usr/bin/env python3
"""INODE-IDENTITY: an owner proves a path is still the thing it made by a HELD
handle, never by a remembered `(device, inode)` number
(`SIGNOFF-REPAIR.11.4.3.1.2.31`).

Linux hands a freed inode number to the next file or directory at once. An
owner that records `(metadata.dev(), metadata.ino())` at creation and compares
it before deleting can therefore delete a SUCCESSOR that took its path, which is
the exact deletion the comparison exists to prevent. macOS gives the successor a
different number, so the defect never shows on the development machine:

- `.23` found it in `extraction_input` (a file) when Linux CI ran;
- `.31` found it in the core test fixture (a directory) the same way;
- the census that followed found it in two more places, one of them the browser
  worker's product workspace.

The remedy is to hold the file or directory OPEN for the owner's whole life and
compare the path against the descriptor's metadata. An open descriptor pins the
inode, so no successor can be given its number. `project_storage`'s
`OwnedDirectory` is the reference shape.

The rule: a `(X.dev(), X.ino())` pair may appear only as one side of a
comparison with another such pair taken from live metadata. Binding one to a
local (`let identity = (…)`) or a struct field (`identity: (…)`) REMEMBERS it,
and is refused.

    python3 -B scripts/check_inode_identity.py              # the check
    python3 -B scripts/check_inode_identity.py --census     # every pair, classified
    python3 -B scripts/check_inode_identity.py --self-test

⚠️ WHAT THIS DOES NOT DO. It reads source text. A pair built from separate
statements (`let d = m.dev(); let i = m.ino();`), or formatted into a string, is
not seen. The string form exists today only as test witnesses of a lock the
test itself holds, which are not identities an owner deletes by.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PAIR = r"\(\s*[a-z_][a-z0-9_.()]*\.dev\(\)\s*,\s*[a-z_][a-z0-9_.()]*\.ino\(\)\s*\)"
# `let name = (…)` or a struct field `name: (…)`: the pair is REMEMBERED.
BOUND = re.compile(r"(?:\blet\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*(?::[^=]*)?=|\b[a-z_][a-z0-9_]*\s*:)\s*" + PAIR)
ANY_PAIR = re.compile(PAIR)


def tracked_rust() -> dict[str, str]:
    listed = subprocess.run(
        ["git", "ls-files", "*.rs"], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.split()
    return {rel: (ROOT / rel).read_text(encoding="utf-8") for rel in listed if (ROOT / rel).is_file()}


def classify(files: dict[str, str]) -> tuple[list[str], list[str]]:
    """(remembered pairs — breaches, compared pairs — allowed), as `file:line: text`."""
    remembered, compared = [], []
    for rel, text in sorted(files.items()):
        for number, line in enumerate(text.split("\n"), 1):
            if not ANY_PAIR.search(line):
                continue
            entry = f"{rel}:{number}: {line.strip()}"
            (remembered if BOUND.search(line) else compared).append(entry)
    return remembered, compared


def self_test() -> int:
    fixture = {
        "a.rs": "\n".join([
            "        let identity = (metadata.dev(), metadata.ino());",
            "            identity: (metadata.dev(), metadata.ino()),",
            "    let pinned: (u64, u64) = (meta.dev(), meta.ino());",
            "        if (metadata.dev(), metadata.ino()) != (held.dev(), held.ino()) {",
            "            || (metadata.dev(), metadata.ino()) != (created.dev(), created.ino())",
            "        assert_eq!((moved.dev(), moved.ino()), (original.dev(), original.ino()));",
            "            (metadata.dev(), metadata.ino()),",
            '        let token = format!("{}:{}", metadata.dev(), metadata.ino());',
            "        let device = metadata.dev();",
        ]),
    }
    remembered, compared = classify(fixture)
    assert [e.split(":")[1] for e in remembered] == ["1", "2", "3"], remembered
    assert [e.split(":")[1] for e in compared] == ["4", "5", "6", "7"], compared
    print("check_inode_identity: self-test OK (a pair bound to a local, a field and a typed local "
          "refused; a pair compared inline, in a condition, in an assertion and split across lines "
          "allowed; a formatted token and a lone device number not pairs)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    remembered, compared = classify(tracked_rust())
    if "--census" in argv:
        print(f"remembered (refused): {len(remembered)}")
        for entry in remembered:
            print(f"  {entry}")
        print(f"compared against live metadata (allowed): {len(compared)}")
        for entry in compared:
            print(f"  {entry}")
        return 0
    if remembered:
        print("INODE-IDENTITY: an owner remembers a (device, inode) number instead of holding a handle:", file=sys.stderr)
        for entry in remembered:
            print(f"    {entry}", file=sys.stderr)
        print("  Linux hands a freed inode number to the next file at once, so the remembered pair can", file=sys.stderr)
        print("  name a SUCCESSOR and the owner deletes what it never made. Hold the file or directory", file=sys.stderr)
        print("  open for the owner's life and compare against the descriptor's metadata, as", file=sys.stderr)
        print("  crates/reasonbraid-server/src/project_storage.rs's OwnedDirectory does.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

#!/usr/bin/env bash
# scripts/check_file_termination.sh — FILE-TERMINATION doctrine.
#
# Every tracked text file ends with exactly one newline: no missing terminator,
# and no blank line at the end.
#
# This is narrow on purpose. A blanket trailing-whitespace rule would be WRONG
# here: `ROADMAP.md` uses trailing double-spaces as Markdown hard line breaks, so
# `git diff --check`'s whitespace family contains a legitimate use in this
# repository. A blank line at end of file does not — it has no meaning in any
# format the tree carries, and it is the exact defect that reached a commit and
# forced the correction commit REPAIR-0062, whose own acceptance record had
# claimed the check it never ran.
#
# Whole-tree rather than staged-diff, so the CI backstop (E4) is real: a file
# that drifted while nothing was staged is still caught. The population is
# small enough to check on every commit — 642 tracked text files scanned in
# well under a second.
#
# Exceptions live in .doctrine/file_termination_exceptions.txt, verbatim with a
# reason, in the same reviewed-allowlist idiom as VISIBILITY-POLICY. A listed
# path that now conforms is ALSO a breach: the list describes what is there.
#
# Self-test: scripts/check_file_termination.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

ALLOWLIST=".doctrine/file_termination_exceptions.txt"

if [ "${1:-}" = "--self-test" ]; then
  python3 -B - "$ALLOWLIST" <<'PY'
import sys, pathlib, tempfile
fails = 0
def classify(b: bytes):
    if b"\0" in b[:8192]:
        return "binary"
    if len(b) == 0:
        return "empty"
    if not b.endswith(b"\n"):
        return "missing final newline"
    if b.endswith(b"\n\n"):
        return "blank line at end of file"
    return "ok"

cases = [
    (b"one line\n", "ok"),
    (b"two\nlines\n", "ok"),
    (b"no terminator", "missing final newline"),
    (b"trailing blank\n\n", "blank line at end of file"),
    (b"many blanks\n\n\n", "blank line at end of file"),
    (b"", "empty"),
    (b"binary\0bytes\n", "binary"),
    (b"hard break  \nkept\n", "ok"),          # Markdown hard break must NOT trip
]
for payload, expected in cases:
    got = classify(payload)
    if got != expected:
        print(f"SELF-TEST: {payload!r} classified {got!r}, expected {expected!r}", file=sys.stderr)
        fails += 1

allow = pathlib.Path(sys.argv[1])
if not allow.is_file():
    print(f"SELF-TEST: missing {allow}", file=sys.stderr); fails += 1

if fails:
    sys.exit(1)
print("FILE-TERMINATION self-test: 8 classifications verified, including a Markdown hard break and a binary file")
PY
  exit $?
fi

python3 -B - "$ALLOWLIST" <<'PY'
import subprocess, sys, pathlib

allowlist = pathlib.Path(sys.argv[1])
allowed = {}
if allowlist.is_file():
    for line in allowlist.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("\t")
        allowed[path.strip()] = reason.strip()

def classify(b: bytes):
    if b"\0" in b[:8192]:
        return None                       # binary: no text termination contract
    if len(b) == 0:
        return None                       # an empty file has nothing to terminate
    if not b.endswith(b"\n"):
        return "missing its final newline"
    if b.endswith(b"\n\n"):
        return "ends with a blank line"
    return None

names = subprocess.run(["git", "ls-files", "-z"], capture_output=True).stdout.split(b"\0")
breaches, seen = [], set()
for raw in names:
    if not raw:
        continue
    name = raw.decode("utf-8", "surrogateescape")
    path = pathlib.Path(name)
    try:
        data = path.read_bytes()
    except (FileNotFoundError, IsADirectoryError, PermissionError):
        continue                          # a submodule or a removed-but-staged path
    verdict = classify(data)
    if verdict is None:
        continue
    if name in allowed:
        seen.add(name)
        continue
    breaches.append((name, verdict))

stale = [name for name in allowed if name not in seen]

if breaches:
    print("FILE-TERMINATION: a tracked text file must end with exactly one newline.", file=sys.stderr)
    for name, verdict in breaches:
        print(f"    {name} {verdict}", file=sys.stderr)
    print("  Fix the file, or — if its bytes are generated or digest-bound — add", file=sys.stderr)
    print(f"  the path and its reason to {allowlist}.", file=sys.stderr)

for name in stale:
    print(f"FILE-TERMINATION: stale exception in {allowlist} (this file now conforms): {name}", file=sys.stderr)

sys.exit(1 if breaches or stale else 0)
PY

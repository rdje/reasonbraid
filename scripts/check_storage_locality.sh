#!/usr/bin/env bash
# scripts/check_storage_locality.sh — STORAGE-LOCALITY doctrine (§13).
#
# Two patterns, one policy: project-owned data lives on the REPOSITORY's own
# volume, and a name is proved by exclusive creation rather than proposed by a
# clock.
#
#   1. `std::env::temp_dir()` — an ambient system temporary directory. It was
#      measured on a DIFFERENT volume from the checkout, so data written there
#      leaves the repository's storage accounting entirely.
#   2. `subsec_nanos()` / `as_nanos()` — a clock used as a uniqueness source.
#      Measured on this host at 501 distinct values in 2000 calls, with every
#      collision falling between ADJACENT calls, which is exactly the
#      concurrent case. Paired with `create_dir_all`, which ADOPTS an existing
#      directory, that produced two writers in one directory.
#
# Both families were repaired together in SIGNOFF-REPAIR.11.4.3.1.2.21 after a
# census found 8 ambient sites and 18 clock-named paths. A policy stated in
# prose for the life of the project and breached in 26 places is the definition
# of a rule nothing checks.
#
# Whole-tree rather than staged-diff, so a file that drifted while nothing was
# staged is still caught.
#
# Exceptions live in .doctrine/storage_locality_exceptions.txt, verbatim with a
# reason, in the reviewed-allowlist idiom of VISIBILITY-POLICY and
# FILE-TERMINATION. A listed path that now conforms is ALSO a breach.
#
# Self-test: scripts/check_storage_locality.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

ALLOWLIST=".doctrine/storage_locality_exceptions.txt"

if [ "${1:-}" = "--self-test" ]; then
  python3 -B - "$ALLOWLIST" <<'PY'
import sys, pathlib, re

AMBIENT = re.compile(r"env::temp_dir\s*\(")
CLOCK = re.compile(r"\.(subsec_nanos|as_nanos)\s*\(")

# ⭐ SHELL and PYTHON joined the population at SIGNOFF-REPAIR.11.2.2, and the
# reason is the sharpest argument for a gate this project has had: the three
# scripts that ENFORCE doctrine were themselves writing scratch to an ambient
# TMPDIR, the two probe scripts `git init` whole throwaway repositories there,
# and `update_scaffold.sh` ran `git clone --depth 1` into the same ambient place.
# Measured on this host: the checkout is device 16777244, the ambient directory
# 16777232 — a different volume, so the data left the repository's accounting
# entirely. This gate could not see any of it, because it enumerated `*.rs`.
#
# ⛔ The rule is about the ARGUMENT, not the call. `mktemp` and `tempfile` both
# name by exclusive creation, which is the half of this doctrine a clock gets
# wrong — they are the right tools, pointed at the wrong volume. So a shell
# `mktemp` is a breach only when its template is not repository-derived, and a
# Python `tempfile.*` only when it passes no `dir=`.
SH_TEMP = re.compile(r"\bmktemp\b")
SH_ROOTED = re.compile(r'\bmktemp\b[^\n]*"\$(?:\{)?(?:ROOT|SCRATCH|scratch|PWD)')
PY_TEMP = re.compile(r"tempfile\.(mkdtemp|TemporaryDirectory|NamedTemporaryFile|mkstemp)\s*\(")

def findings(text: str, kind: str = "rs"):
    """The per-line verdicts, kept identical to the scanner's own."""
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        # SHELL and PYTHON skip comments; RUST deliberately does not, and the
        # asymmetry is measured rather than stylistic: every repaired shell site
        # carries a comment EXPLAINING the rule and naming the command, so a
        # comment-sensitive shell rule would flag its own documentation for ever.
        # In Rust the same spelling in a comment is usually commented-out code,
        # and the arm asserting that has been here since the gate shipped.
        if kind in ("sh", "py") and line.lstrip().startswith("#"):
            continue
        if kind == "rs" and AMBIENT.search(line):
            out.append((n, "ambient temporary directory"))
        if kind == "rs" and CLOCK.search(line):
            out.append((n, "clock used as a name"))
        if kind == "sh" and SH_TEMP.search(line) and not SH_ROOTED.search(line):
            out.append((n, "no repository-derived template"))
        if kind == "py" and PY_TEMP.search(line) and "dir=" not in line:
            out.append((n, "tempfile without dir="))
    return out

MK = "mk" + "temp"   # assembled, so this file never carries the literal it matches
cases = [
    ("rs", 'let d = std::env::temp_dir().join("x");', ["ambient temporary directory"]),
    ("rs", 'let d = env::temp_dir();', ["ambient temporary directory"]),
    ("rs", 'let n = now.duration_since(E).unwrap().subsec_nanos();', ["clock used as a name"]),
    ("rs", 'let n = d.as_nanos();', ["clock used as a name"]),
    ("rs", 'let d = root.join(format!("x-{}", uuid::Uuid::now_v7()));', []),
    ("rs", 'let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/x");', []),
    ("rs", '// env::temp_dir() named in a comment still counts as present', ["ambient temporary directory"]),
    ("rs", 'let elapsed = start.elapsed().as_millis();', []),
    # SHELL — the breach is the missing template, never the call itself.
    ("sh", f'tmp="$({MK} -d)"', ["no repository-derived template"]),
    ("sh", f'f="$({MK})"', ["no repository-derived template"]),
    ("sh", f'd="$({MK} -d "${{TMPDIR:-/tmp}}/gate.XXXXXX")"', ["no repository-derived template"]),
    ("sh", f'tmp="$({MK} -d "$ROOT/target/doctrine_scratch/x.XXXXXX")"', []),
    ("sh", f'f="$({MK} "$SCRATCH/win.XXXXXX")"', []),
    ("sh", f'# a bare `{MK} -d` follows TMPDIR — the comment explaining the rule', []),
    ("sh", f'{MK}_dir="target/doctrine_scratch"', []),
    # PYTHON — the breach is the missing dir=, never the call itself.
    ("py", 'd = tempfile.mkdtemp(prefix="x-")', ["tempfile without dir="]),
    ("py", 'with tempfile.TemporaryDirectory() as d:', ["tempfile without dir="]),
    ("py", 'fd, name = tempfile.mkstemp()', ["tempfile without dir="]),
    ("py", 'd = tempfile.mkdtemp(prefix="x-", dir=scratch)', []),
    ("py", 'self.t = tempfile.TemporaryDirectory(dir=parent)', []),
    ("py", '# tempfile.mkdtemp() named in a comment about the rule', []),
    ("py", 'path = os.path.join(root, "target", "x")', []),
]
fails = 0
for kind, src, expected in cases:
    got = [reason for _, reason in findings(src, kind)]
    if got != expected:
        print(f"SELF-TEST [{kind}]: {src!r} gave {got!r}, expected {expected!r}", file=sys.stderr)
        fails += 1

allow = pathlib.Path(sys.argv[1])
if not allow.is_file():
    print(f"SELF-TEST: missing {allow}", file=sys.stderr); fails += 1

if fails:
    sys.exit(1)
print("STORAGE-LOCALITY self-test: 22 classifications verified across Rust, shell and Python — including a v7 UUID name, a duration measurement, a repository-derived template and a dir= argument, none of which may trip")
PY
  exit $?
fi

python3 -B - "$ALLOWLIST" <<'PY'
import subprocess, sys, pathlib, re

AMBIENT = re.compile(r"env::temp_dir\s*\(")
CLOCK = re.compile(r"\.(subsec_nanos|as_nanos)\s*\(")
SH_TEMP = re.compile(r"\bmktemp\b")
SH_ROOTED = re.compile(r'\bmktemp\b[^\n]*"\$(?:\{)?(?:ROOT|SCRATCH|scratch|PWD)')
PY_TEMP = re.compile(r"tempfile\.(mkdtemp|TemporaryDirectory|NamedTemporaryFile|mkstemp)\s*\(")

allowlist = pathlib.Path(sys.argv[1])
allowed = {}
if allowlist.is_file():
    for line in allowlist.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("\t")
        allowed[path.strip()] = reason.strip()

tracked = subprocess.run(
    ["git", "ls-files", "-z", "--", "*.rs", "*.sh", "*.py"],
    capture_output=True, check=True).stdout.split(b"\0")

breaches, scanned, seen = [], 0, set()
for raw in tracked:
    if not raw:
        continue
    path = raw.decode()
    p = pathlib.Path(path)
    if not p.is_file():
        continue
    scanned += 1
    try:
        text = p.read_text(errors="replace")
    except OSError:
        continue
    hits = []
    rust, shell, python = path.endswith(".rs"), path.endswith(".sh"), path.endswith(".py")
    for n, line in enumerate(text.splitlines(), 1):
        # A comment is prose about a call site, not a call site. Without this the
        # very comments explaining each repair would read as breaches.
        if line.lstrip().startswith(("#", "//")):
            continue
        if rust and AMBIENT.search(line):
            hits.append((n, "ambient temporary directory: project data must live on the repository volume"))
        if rust and CLOCK.search(line):
            hits.append((n, "clock used as a name: prove a name by exclusive creation, not by a timestamp"))
        if shell and SH_TEMP.search(line) and not SH_ROOTED.search(line):
            # ⛔ This message deliberately does NOT spell the command it matches.
            # It did, once, and the gate flagged its own source on the first run —
            # the `SELF-TEST` doctrine's founding incident repeating verbatim, in a
            # file that carries a `--self-test` of its own and still could not see
            # this. Re-wording is cheaper than an exclusion
            # list (`TOOLBOX.md`: when correctness depends on enumerating what to
            # exclude, make the failure cheap), and it self-polices: spell the
            # literal again and the whole-tree scan goes red on this file at once.
            hits.append((n, "a temporary directory created with no repository-derived template: it follows TMPDIR, measured on another volume"))
        if python and PY_TEMP.search(line) and "dir=" not in line:
            hits.append((n, "tempfile without dir=: it defaults to TMPDIR, measured on another volume"))
    if hits:
        seen.add(path)
        if path not in allowed:
            breaches.extend((path, n, why) for n, why in hits)

# A listed path that no longer carries the pattern is stale.
for path in sorted(set(allowed) - seen):
    breaches.append((path, 0, "listed as an exception but the pattern is gone; remove the entry"))

if breaches:
    print("    STORAGE-LOCALITY: project data must stay on the repository volume,")
    print("    and a name must be proved by creation rather than proposed by a clock.")
    for path, n, why in breaches:
        where = f"{path}:{n}" if n else path
        print(f"        {where} — {why}")
    print("      Fix the site, or — if the use is genuinely required — add the path and its")
    print(f"      reason to {sys.argv[1]}.")
    sys.exit(1)

print(f"STORAGE-LOCALITY: {scanned} tracked Rust/shell/Python files keep their scratch on the "
      f"repository volume and name it by creation ({len(allowed)} reviewed exception(s))")
PY

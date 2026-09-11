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

def findings(text: str):
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        if AMBIENT.search(line):
            out.append((n, "ambient temporary directory"))
        if CLOCK.search(line):
            out.append((n, "clock used as a name"))
    return out

cases = [
    ('let d = std::env::temp_dir().join("x");', ["ambient temporary directory"]),
    ('let d = env::temp_dir();', ["ambient temporary directory"]),
    ('let n = now.duration_since(E).unwrap().subsec_nanos();', ["clock used as a name"]),
    ('let n = d.as_nanos();', ["clock used as a name"]),
    ('let d = root.join(format!("x-{}", uuid::Uuid::now_v7()));', []),
    ('let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/x");', []),
    ('// env::temp_dir() named in a comment still counts as present', ["ambient temporary directory"]),
    ('let elapsed = start.elapsed().as_millis();', []),
]
fails = 0
for src, expected in cases:
    got = [reason for _, reason in findings(src)]
    if got != expected:
        print(f"SELF-TEST: {src!r} gave {got!r}, expected {expected!r}", file=sys.stderr)
        fails += 1

allow = pathlib.Path(sys.argv[1])
if not allow.is_file():
    print(f"SELF-TEST: missing {allow}", file=sys.stderr); fails += 1

if fails:
    sys.exit(1)
print("STORAGE-LOCALITY self-test: 8 classifications verified, including a v7 UUID name and a duration measurement that must NOT trip")
PY
  exit $?
fi

python3 -B - "$ALLOWLIST" <<'PY'
import subprocess, sys, pathlib, re

AMBIENT = re.compile(r"env::temp_dir\s*\(")
CLOCK = re.compile(r"\.(subsec_nanos|as_nanos)\s*\(")

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
    ["git", "ls-files", "-z", "--", "*.rs"],
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
    for n, line in enumerate(text.splitlines(), 1):
        if AMBIENT.search(line):
            hits.append((n, "ambient temporary directory: project data must live on the repository volume"))
        if CLOCK.search(line):
            hits.append((n, "clock used as a name: prove a name by exclusive creation, not by a timestamp"))
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

print(f"STORAGE-LOCALITY: {scanned} tracked Rust files carry no ambient temporary directory "
      f"and no clock-derived name ({len(allowed)} reviewed exception(s))")
PY

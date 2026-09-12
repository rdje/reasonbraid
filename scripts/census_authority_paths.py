#!/usr/bin/env python3
"""The tenant authority writer/caller census, re-derivable at any commit.

`SIGNOFF-REPAIR.3.3.4.1` recorded this census as a table plus a corpus
fingerprint. A table is a measurement someone took once; this is the instrument
that took it, so a later leaf can re-run it instead of trusting the table.

    python3 -B scripts/census_authority_paths.py            # the working tree
    python3 -B scripts/census_authority_paths.py <sha>      # any past commit

The method is deliberately the original's, bounded and lexical: tracked
crates/*/src Rust files, direct calls of the named authority functions,
excluding declarations and whole-line comments. It is NOT an AST pass, and it
sees neither dynamic dispatch nor arbitrary SQL — `.3.3.4.1` states those limits
and they are unchanged.

Its own correctness check is that it reproduces the recorded baseline exactly:

    python3 -B scripts/census_authority_paths.py 1ba6184
    # 101 files, 1749975 bytes,
    # SHA-256 340c4af65db88e48496797c650bbce851bdfde47aaaefac5d93565cd38d26c2c,
    # 42 direct named-call locations

A change to the predicate that breaks that reproduction makes every later
comparison incomparable, which is the failure this file exists to prevent.
"""
import hashlib, re, subprocess, sys

NAMES = [
    "authorize", "authorize_in_tx", "authorize_tenant_admin",
    "authorize_tenant_admin_inspection", "bump_revocation_epoch",
    "create_grant_in_tx", "insert_boundary_in_tx", "revoke_boundary",
    "revoke_grant", "create_boundary", "create_grant",
    "apply_authorized_command",
]
CALL = re.compile(r"\b(" + "|".join(NAMES) + r")\s*\(")
DECL = re.compile(r"^\s*(pub(\(\w+\))?\s+)?(async\s+)?fn\s+")
COMMENT = re.compile(r"^\s*(//|/\*|\*)")

REV = sys.argv[1] if len(sys.argv) > 1 else None

def ls():
    if REV:
        out = subprocess.run(["git", "ls-tree", "-r", "--name-only", REV],
                             capture_output=True, text=True, check=True).stdout.split()
    else:
        out = subprocess.run(["git", "ls-files"],
                             capture_output=True, text=True, check=True).stdout.split()
    return sorted(f for f in out if f.endswith(".rs") and f.startswith("crates/") and "/src/" in f)

def read(path):
    if REV:
        return subprocess.run(["git", "show", f"{REV}:{path}"],
                              capture_output=True, check=True).stdout
    return open(path, "rb").read()

files = ls()

hits = []
h = hashlib.sha256()
total = 0
for path in files:
    raw = read(path)
    total += len(raw)
    h.update(path.encode()); h.update(b"\0"); h.update(raw); h.update(b"\0")
    for n, line in enumerate(raw.decode("utf-8", "replace").splitlines(), 1):
        if COMMENT.match(line) or DECL.match(line):
            continue
        m = CALL.search(line)
        if m:
            hits.append((path, n, m.group(1)))

print(f"tracked Rust source files: {len(files)}")
print(f"corpus bytes: {total}")
print(f"corpus SHA-256: {h.hexdigest()}")
print(f"direct named-call locations: {len(hits)}")
for path, n, name in hits:
    print(f"  {path}:{n}\t{name}")

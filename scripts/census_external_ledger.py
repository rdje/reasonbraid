#!/usr/bin/env python3
"""Census the ROADMAP §7.4 external dependency ledger.

`SIGNOFF-REPAIR.6.8.2`. §7.4 says the ledger records fourteen named fields for
every protocol, SDK, CLI, provider and harness, that *CI warns on expired
checks*, and that *release gates require fresh records for exposed compatibility
profiles*.

🔴 **Nothing read the file.** `grep -rln "external-ledger" scripts/ .github/
Makefile deploy/` returned nothing: the warning §7.4 advertises did not exist,
the release gate's freshness requirement had no implementation, and the ledger's
own header described both as though they ran.

    python3 -B scripts/census_external_ledger.py             # the census
    python3 -B scripts/census_external_ledger.py --check     # the gate: shape only
    python3 -B scripts/census_external_ledger.py --stale     # the §7.4 warning
    python3 -B scripts/census_external_ledger.py --triggers  # the mechanical triggers
    python3 -B scripts/census_external_ledger.py --json
    python3 -B scripts/census_external_ledger.py --self-test

⛔ **`--check` GATES SHAPE AND NEVER AGE, and the reason is `SIGNOFF-REPAIR.11.6`:
derive a number, never choose one.** §7.4 says CI *warns* on an expired check and
gives no horizon; inventing "30 days" here would be a threshold nobody derived,
enforced forever. So staleness is REPORTED with each row's age and the horizon is
left undecided — owned, not guessed.

⛔ **IT PARSES THIS FILE'S SHAPE, NOT YAML, and says so rather than implying
generality.** `pyyaml` is not a dependency of this repository and adding one to
read a file the project writes by hand would be the wrong trade. The parser
handles what the ledger is: a `entries:` list whose members start at `  - name:`
and whose fields are either a scalar, a `[]`, a block scalar (`>-`) or a
`  - ` list. ⚠️ A field spelled any other way reads as ABSENT, which fails
`--check` loudly rather than passing quietly — the safe direction.
"""

from __future__ import annotations

import datetime as dt
import json
import tomllib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = Path("docs/dependencies/external-ledger.yaml")
VENDOR = Path(".project-data/cargo/registry/src")

# ⛔ The fourteen §7.4 names, plus the two the ledger's own header documents as
# additions (`kind`, `baseline`). Quoted from the roadmap, not paraphrased.
REQUIRED = [
    "name", "owner", "source_url", "checked_at",
    "tested_versions", "protocol_versions", "license",
    "supported_features", "transports", "auth_modes",
    "known_semantic_losses", "conformance_results",
    "security_notes", "upgrade_policy", "revalidation_trigger",
]
DOCUMENTED_ADDITIONS = ["kind", "baseline"]


def entries(text: str) -> list[dict[str, object]]:
    """Split the ledger into entries and read each field.

    A field's value is one of: a scalar, an empty list `[]`, a block scalar
    (`>-`), or a `  - ` item list. Anything else is not represented, which makes
    it read as absent.
    """
    blocks = re.split(r"^  - name:", text, flags=re.M)[1:]
    out: list[dict[str, object]] = []
    for block in blocks:
        block = "  - name:" + block
        fields: dict[str, object] = {}
        lines = block.splitlines()
        for i, line in enumerate(lines):
            m = re.match(r"^\s+-?\s*([a-z_]+):\s*(.*)$", line)
            if not m:
                continue
            key, rest = m.group(1), m.group(2)
            if key not in REQUIRED + DOCUMENTED_ADDITIONS:
                continue
            rest = re.sub(r"\s+#.*$", "", rest).strip()
            if rest == "[]":
                fields[key] = []
            elif rest in (">-", "|", ">"):
                body = []
                for nxt in lines[i + 1:]:
                    if re.match(r"^\s+[a-z_]+:", nxt) or nxt.startswith("  - name:"):
                        break
                    body.append(nxt.strip())
                fields[key] = " ".join(b for b in body if b)
            elif rest:
                fields[key] = rest.strip('"')
            else:
                items = []
                for nxt in lines[i + 1:]:
                    if re.match(r"^\s{4,}- ", nxt):
                        items.append(re.sub(r"\s+#.*$", "", nxt.strip()[2:]).strip().strip('"'))
                    elif nxt.strip():
                        break
                fields[key] = items
        out.append(fields)
    return out


def shape_breaches(rows: list[dict[str, object]]) -> list[str]:
    """§7.4's SHAPE, and nothing about time."""
    out: list[str] = []
    if not rows:
        out.append("the ledger carries no entries")
    for row in rows:
        name = str(row.get("name", "<unnamed>"))
        for field in REQUIRED:
            if field not in row:
                out.append(f"`{name}`: §7.4 names `{field}` and the entry has no such field")
        checked = str(row.get("checked_at", ""))
        if checked and not re.fullmatch(r"\d{4}-\d{2}-\d{2}", checked):
            out.append(f"`{name}`: checked_at `{checked}` is not an ISO date")
    return out


def ages(rows: list[dict[str, object]], today: dt.date) -> list[tuple[str, str, int]]:
    out = []
    for row in rows:
        checked = str(row.get("checked_at", ""))
        try:
            when = dt.date.fromisoformat(checked)
        except ValueError:
            continue
        out.append((str(row.get("name", "?")), checked, (today - when).days))
    return sorted(out, key=lambda r: -r[2])


def crate_requirement(manifest_text: str, crate: str) -> str | None:
    """One crate's requirement on `crate`, in ANY of cargo's spellings.

    🔴 **A REGEX GOT THIS WRONG AND THE ERROR WAS THE DANGEROUS DIRECTION.** The
    first version matched `base64 = "0.23"` and `base64 = { version = … }` and
    missed `[dependencies.base64]` with `version` on the next line — which is how
    `rmcp 3.2.0` declares it. It therefore found NOTHING, and the caller read
    "nothing" as "one requirement remains" and announced that the block had
    LIFTED. `tomllib` handles every form cargo accepts, and the caller no longer
    has a branch that an empty result can reach.
    """
    try:
        data = tomllib.loads(manifest_text)
    except Exception:
        return None
    for table in ("dependencies", "build-dependencies"):
        dep = data.get(table, {}).get(crate)
        if dep is None:
            continue
        req = dep if isinstance(dep, str) else dep.get("version")
        if req:
            return str(req)
    return None


def base64_split(root: Path) -> dict[str, list[str]]:
    """The MCP row's MECHANICAL trigger: is the base64 requirement still split?

    Read from the VENDORED manifests rather than from `cargo metadata`, because
    the answer is a property of what the crates DECLARE and not of what the
    resolver picked — and because a gate must not resolve a dependency graph.
    """
    reqs: dict[str, list[str]] = {}
    src = root / VENDOR
    if not src.is_dir():
        return reqs
    for registry in src.iterdir():
        if not registry.is_dir():
            continue
        for manifest in registry.glob("*/Cargo.toml"):
            try:
                req = crate_requirement(manifest.read_text(encoding="utf-8"), "base64")
            except OSError:
                continue
            if req:
                major = ".".join(req.lstrip("^~=").split(".")[:2])
                reqs.setdefault(major, []).append(manifest.parent.name)
    return reqs


def report(rows, today) -> int:
    print(f"external dependency ledger — {len(rows)} entr(ies), ROADMAP §7.4")
    print()
    for name, checked, age in ages(rows, today):
        print(f"  {name[:56]:<58} checked_at {checked}  ({age} days)")
    print()
    empty = [(str(r.get("name", "?")), f) for r in rows for f in REQUIRED
             if isinstance(r.get(f), list) and not r[f]]
    unver = [str(r.get("name", "?")) for r in rows if r.get("license") == "unverified"]
    print(f"  fields still empty: {len(empty)}")
    for name, field in empty:
        print(f"      {name[:44]:<46} {field}")
    print(f"  licenses still `unverified`: {unver or 'none'}")
    print()
    bad = shape_breaches(rows)
    print("  shape:", "complete" if not bad else f"{len(bad)} breach(es)")
    for b in bad:
        print(f"      {b}")
    print()
    print("  --stale reports the ages alone; --triggers evaluates the mechanical triggers.")
    return 0


def triggers(root: Path) -> int:
    """Evaluate the triggers a machine CAN evaluate."""
    split = base64_split(root)
    print("mechanical revalidation triggers")
    print()
    print("  MCP / rmcp — the Streamable-HTTP block (SIGNOFF-REPAIR.6.6):")
    for major in sorted(split):
        who = sorted(set(split[major]))
        print(f"      base64 ^{major}: {len(who)} vendored crate(s) — {who[:6]}{' …' if len(who) > 6 else ''}")
    # ⛔ THE POSITIVE CONTROL, and it is the whole guard. An empty result means
    # the instrument saw nothing — a vendored tree that is absent, or a parser
    # that has stopped matching — and "saw nothing" is NOT "the split closed".
    # The first version of this function had exactly that branch and announced
    # that the block had LIFTED while finding zero requirements.
    if not split:
        print("      🔴 INSTRUMENT FAILURE: no base64 requirement found at all.")
        print(f"         Expected the vendored registry under {VENDOR}. This is not a")
        print("         verdict about the split — it is this check unable to answer.")
        return 2
    if len(split) == 1:
        print("      ⭐ FIRED: exactly one base64 requirement remains. The split has closed")
        print("         and both rmcp Streamable-HTTP transports become an ordinary feature")
        print("         decision again — re-open SIGNOFF-REPAIR.6.6's ruling.")
    else:
        print("      not fired: the requirement is still split, so enabling any rmcp HTTP")
        print("      transport would put two base64 majors in the tree, which deny.toml forbids.")
    return 0


def self_test() -> int:
    arms: list[tuple[str, bool]] = []
    good = (
        'entries:\n'
        '  - name: "X"\n'
        '    kind: "sdk"\n'
        '    owner: "o"\n'
        '    source_url: "u"\n'
        '    checked_at: "2026-09-20"\n'
        '    tested_versions:\n'
        '      - "v 1.0"\n'
        '    protocol_versions: []\n'
        '    license: "MIT"\n'
        '    supported_features: []\n'
        '    transports: []\n'
        '    auth_modes: []\n'
        '    known_semantic_losses: []\n'
        '    conformance_results: []\n'
        '    security_notes: []\n'
        '    upgrade_policy: "p"\n'
        '    revalidation_trigger: >-\n'
        '      a release\n'
        '    baseline: "b"\n'
    )
    rows = entries(good)
    arms.append(("a complete entry parses", len(rows) == 1))
    # 1 the SILENT case first — without it every arm below passes against a
    #   predicate that always breaches.
    arms.append(("a complete entry has no shape breach", shape_breaches(rows) == []))
    # 2 a MISSING §7.4 field is refused by name
    missing = entries(good.replace('    license: "MIT"\n', ""))
    got = shape_breaches(missing)
    arms.append(("a missing §7.4 field is refused by name", any("license" in b for b in got)))
    # 3 ⛔ AN EMPTY LIST IS NOT A MISSING FIELD. `[]` means "not yet pinned", which
    #   the ledger's own header defines; gating on it would demand invention.
    arms.append(("an empty list is present, not missing", rows[0]["protocol_versions"] == []))
    # 4 a block scalar is read as its text, not dropped
    arms.append(("a block scalar parses", rows[0]["revalidation_trigger"] == "a release"))
    # 5 an item list parses, and the inline comment is stripped
    listed = entries(good.replace('      - "v 1.0"\n', '      - "v 1.0"   # a note\n'))
    arms.append(("a list item drops its inline comment", listed[0]["tested_versions"] == ["v 1.0"]))
    # 6 a malformed date is refused — the field exists, so only the shape check sees it
    bad = entries(good.replace('"2026-09-20"', '"soon"'))
    arms.append(("a non-ISO checked_at is refused", any("ISO" in b for b in shape_breaches(bad))))
    # 7 ages are computed from the row, not from a stored number
    arms.append(("an age is derived", ages(rows, dt.date(2026, 9, 30))[0][2] == 10))
    # 8 ⛔ AGE IS NEVER A BREACH. §7.4 says CI WARNS; a horizon nobody derived
    #   must not block, and this arm is what stops one being added quietly.
    old = entries(good.replace('"2026-09-20"', '"2020-01-01"'))
    arms.append(("a very old row is NOT a shape breach", shape_breaches(old) == []))
    # 9 ⛔ THE TRIGGER'S PARSER, in every spelling cargo accepts. A regex that
    #   handled two of the three announced that the upstream block had LIFTED.
    arms.append(("an inline string requirement parses",
                 crate_requirement('[dependencies]\nbase64 = "0.22"\n', "base64") == "0.22"))
    arms.append(("an inline table requirement parses",
                 crate_requirement('[dependencies]\nbase64 = { version = "0.22", optional = true }\n', "base64") == "0.22"))
    arms.append(("a SEPARATE dependency table parses — the form that was missed",
                 crate_requirement('[dependencies.base64]\nversion = "0.23"\noptional = true\n', "base64") == "0.23"))
    # 10 and the negative side: a manifest that does not depend on it says so,
    #    so arm 9 is not passing against a function that returns something always.
    arms.append(("a manifest without the crate returns None",
                 crate_requirement('[dependencies]\nserde = "1"\n', "base64") is None))
    # 11 ⛔ the LIVE trigger still sees the split — if this ever flips, the block
    #    has genuinely lifted, and that is the trigger doing its job rather than
    #    the instrument going blind.
    arms.append(("the live base64 requirement is still split", len(base64_split(ROOT)) > 1))
    # 12 the real ledger parses, and every entry it holds is complete
    real = entries((ROOT / LEDGER).read_text(encoding="utf-8"))
    arms.append(("the real ledger parses to >=1 entry", len(real) >= 1))
    arms.append(("the real ledger is shape-complete", shape_breaches(real) == []))

    passed = sum(1 for _, ok in arms if ok)
    for label, ok in arms:
        print(f"  {'PASS' if ok else 'FAIL'}  {label}")
    print(f"census_external_ledger --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    if mode == "--triggers":
        return triggers(ROOT)
    rows = entries((ROOT / LEDGER).read_text(encoding="utf-8"))
    today = dt.date.today()
    if mode == "--json":
        print(json.dumps({"entries": rows, "ages": ages(rows, today)}, indent=2))
        return 0
    if mode == "--stale":
        for name, checked, age in ages(rows, today):
            print(f"{age:>5} days  {checked}  {name}")
        print("\n⚠️ ROADMAP §7.4 says CI WARNS on an expired check and names no horizon.")
        print("   No threshold is applied here; one would be a number nobody derived.")
        return 0
    if mode == "--check":
        found = shape_breaches(rows)
        for line in found:
            print(f"EXTERNAL-LEDGER: {line}", file=sys.stderr)
        if found:
            print(
                "  ROADMAP §7.4 names the fields every ledger entry records. An empty list\n"
                "  is a legitimate 'not yet pinned'; an ABSENT field is a record that cannot\n"
                "  be read. Add the field to docs/dependencies/external-ledger.yaml.",
                file=sys.stderr,
            )
            return 1
        return 0
    return report(rows, today)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

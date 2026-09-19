#!/usr/bin/env python3
"""Census every policy line a resolver pack ADVERTISES, and say what reads it.

`SIGNOFF-REPAIR.7.3.6.1`. A resolver pack publishes six policy fields to every
caller that reads the §12.2 registry — `egress_class`, `sandbox_level`,
`redirect_policy`, `archive_policy`, `subresource_policy`, `javascript_policy`.
A caller CHOOSING a pack reads all six. `SIGNOFF-REPAIR.7.3.5` (REPAIR-0210)
found two of them advertised as `deny` and enforced by nothing, and repaired
exactly those two; this instrument enumerates the rest so the next reader starts
from a population rather than from the advertisement.

    python3 -B scripts/census_advertised_policies.py            # the census
    python3 -B scripts/census_advertised_policies.py --check    # the gate
    python3 -B scripts/census_advertised_policies.py --readers  # the reader census
    python3 -B scripts/census_advertised_policies.py --json
    python3 -B scripts/census_advertised_policies.py --self-test

⭐ BOTH POPULATIONS ARE READ FROM THEIR PRODUCER, never from a list written
here. The gated packs (R3/R5/RX) exist only as Rust literals in
`resolvers.rs::gated_advertises` — the startup sync registers them when the gate
opens and DELETES them when it closes, so no migration seeds them. The built-in
packs (R0/R1/R2) exist only as `INSERT INTO resolver_capabilities` rows in
`migrations/`. `CLAIM_VERIFICATION.md` leg 2: derive a membership test from the
code that emits the thing, never from a description of it. A pack added to
either producer appears here on the next run; a hand-written list would not.

⛔ THE TWO POPULATIONS ARE NOT ONE POPULATION AND ARE NEVER SUMMED INTO A
SINGLE CLAIM WITHOUT SAYING SO. A Rust literal is traceable to a call graph; a
migration row is a database value that the same `register()` upsert may later
overwrite in part. They are counted separately and printed separately, and the
`origin` column says which is which.

⭐ THE READER CLASSIFICATION ERRS TOWARDS "READ", deliberately. The census's
sharpest output is the claim that four of the six fields are consumed by
nothing, and `CLAIM_VERIFICATION.md` leg 2 requires a claim about a SET to carry
its enumeration in BOTH directions. So every occurrence of a field name in
tracked Rust is classified, and the DEFAULT for an occurrence no rule explains
is `read` — an unrecognised line inflates the reader count and deflates the
claim. A scoping defect here errs in the direction that weakens the finding
(`docs/knowledge/a-scoping-defect-errs-in-one-direction.md`).

⛔ IT DOES NOT ASK WHETHER A LINE IS TRUE. Whether `javascript_policy:
"allow-bounded"` describes the worker is a judgement with evidence behind it,
and judgements live in the tracked ledger beside the leaf that earned them —
`.doctrine/advertised_policy_verdicts.tsv`. This file enumerates the lines and
refuses one that carries no verdict; it does not grade them. That split is what
lets the census be re-run against a tree the verdicts have changed.
"""

from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]

RUST_ADVERTISE = "crates/reasonbraid-server/src/resolvers.rs"
LEDGER = ".doctrine/advertised_policy_verdicts.tsv"

#: The six §12.2 policy fields, in the order the registry's INSERT lists them.
POLICY_FIELDS = (
    "egress_class",
    "sandbox_level",
    "redirect_policy",
    "archive_policy",
    "subresource_policy",
    "javascript_policy",
)

#: The verdicts the ledger may carry. A line's verdict answers ONE question:
#: what stands between the advertisement and a caller who believes it?
VERDICTS = {
    # A mechanism in the product refuses, and a control has been observed RED
    # — with the leaf or commit that observed it named in the evidence column.
    "enforced",
    # A mechanism is present and a control covers it, but NO RED observation is
    # on record. ⛔ Not a defect claim, and not a weaker `enforced`: it is the
    # named gap `CLAIM_VERIFICATION.md` §4 requires instead of a hidden one. A
    # control never seen refuse is not known to discriminate (§6), so the line
    # rests on reading the mechanism rather than on watching it hold.
    "unverified",
    # The pack has no mechanism that could violate the line: the population of
    # violating acts is empty, and the census names the evidence of absence.
    # ⛔ Not a synonym for `enforced` — nothing would notice if it stopped
    # being vacuous, which is exactly what a sibling leaf may have to fix.
    "vacuous",
    # The pack CAN violate the line and nothing refuses. A defect.
    "unenforced",
    # The advertised word has no stated meaning, so the line is neither true
    # nor false as written.
    "undefined",
    # The word describes something other than what the code does — the
    # advertisement is what is wrong, not the code.
    "misdescribed",
    # Measured, routed to a named sibling leaf, not yet adjudicated.
    "open",
}


def tracked(*patterns: str) -> list[str]:
    """The tracked files matching git pathspecs — the corpus, never a walk."""
    out = subprocess.run(
        ["git", "ls-files", "-z", *patterns],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return sorted(p for p in out.split("\0") if p)


# ── Population A: the Rust literals (the gated packs) ────────────────────────


def parse_rust(text: str) -> list[dict[str, object]]:
    """Every `ResolverAdvertise { … }` literal, with its six policy fields.

    ⛔ Brace-counted rather than regex-delimited: the literal contains a nested
    `serde_json::json!({ … })` for `latency_range_ms` and `security_evidence`,
    so a non-greedy match to the first `}` stops inside the wrong object and a
    greedy one swallows every pack into the first row.

    ⭐ THE PACK IS KEYED BY THE ID A CALLER SEES, not by the Rust constant that
    carries it. The advertise writes `resolver_id: R3_RESOLVER_ID`, the registry
    row and every caller read `r3-browser-worker`, and the ledger has to join
    the two populations on one key. An unresolvable constant keeps its own
    spelling rather than being dropped — a pack missing from the census is the
    one failure this instrument must not have.
    """
    constants = dict(
        re.findall(r'const\s+([A-Z0-9_]+):\s*&str\s*=\s*"([^"]*)"', text)
    )
    rows: list[dict[str, object]] = []
    for opening in re.finditer(r"ResolverAdvertise\s*\{", text):
        depth, i = 0, opening.end() - 1
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = text[opening.end() : i]
        pack = re.search(r"resolver_id:\s*([A-Z0-9_]+)", body)
        if not pack:
            continue
        name = constants.get(pack.group(1), pack.group(1))
        line = text.count("\n", 0, opening.start()) + 1
        for field in POLICY_FIELDS:
            hit = re.search(rf'\b{field}:\s*"([^"]*)"', body)
            if hit:
                rows.append(
                    {
                        "pack": name,
                        "field": field,
                        "value": hit.group(1),
                        "origin": "rust",
                        "at": f"{RUST_ADVERTISE}:{line + body.count(chr(10), 0, hit.start())}",
                    }
                )
    return rows


# ── Population B: the migration INSERT rows (the built-in packs) ─────────────

_INSERT = re.compile(
    r"INSERT\s+INTO\s+resolver_capabilities\s*\((?P<cols>[^)]*)\)\s*VALUES\s*\((?P<vals>.*?)\n\);",
    re.IGNORECASE | re.DOTALL,
)


def _split_values(raw: str) -> list[str]:
    """Split a VALUES list on top-level commas, respecting quotes and parens."""
    out, depth, quoted, current = [], 0, False, []
    for ch in raw:
        if quoted:
            current.append(ch)
            if ch == "'":
                quoted = False
            continue
        if ch == "'":
            quoted = True
            current.append(ch)
        elif ch == "(":
            depth += 1
            current.append(ch)
        elif ch == ")":
            depth -= 1
            current.append(ch)
        elif ch == "," and depth == 0:
            out.append("".join(current).strip())
            current = []
        else:
            current.append(ch)
    if "".join(current).strip():
        out.append("".join(current).strip())
    return out


def parse_sql(path: str, text: str) -> list[dict[str, object]]:
    """Every registry INSERT row's six policy fields, by COLUMN NAME.

    ⛔ Positional by necessity — SQL binds values to columns by order — but the
    column NAME is what selects the position, so a migration that reorders its
    column list still reports the right value. A census keyed to a fixed index
    would silently transpose two policies the day someone reorders them.
    """
    rows: list[dict[str, object]] = []
    for insert in _INSERT.finditer(text):
        columns = [c.strip() for c in insert.group("cols").replace("\n", " ").split(",")]
        values = _split_values(insert.group("vals"))
        if len(columns) != len(values):
            rows.append({"pack": "?", "field": "?", "value": "?", "origin": "sql",
                         "at": path, "error": f"{len(columns)} columns, {len(values)} values"})
            continue
        by_name = dict(zip(columns, values))
        pack = by_name.get("resolver_id", "?").strip("'")
        line = text.count("\n", 0, insert.start()) + 1
        for field in POLICY_FIELDS:
            if field in by_name:
                rows.append(
                    {
                        "pack": pack,
                        "field": field,
                        "value": by_name[field].strip().strip("'"),
                        "origin": "sql",
                        "at": f"{path}:{line}",
                    }
                )
    return rows


# ── The reader census: what consumes a policy field at all ───────────────────

#: An occurrence matching one of these is NOT a read. Everything else is, and
#: that default is the point — see the module header.
_NOT_A_READ = (
    # the SDK's own struct field declaration and its serde default attribute
    (re.compile(r"^\s*pub\s+\w+:\s"), "declaration"),
    (re.compile(r'^\s*#\[serde\('), "declaration"),
    # a Rust literal assigning the field inside an advertise (a WRITE)
    (re.compile(r'^\s*\w+:\s*"[^"]*"\.(?:to_owned|to_string)\(\)|^\s*\w+:\s*"[^"]*"\.into\(\)'), "write"),
    # the INSERT's column list and its bind, in `register()`
    (re.compile(r"^\s*\.bind\(&advertise\."), "write"),
    (re.compile(r"^\s*[\w\s,\\]*(?:_policy|_class|_level)[\w\s,\\]*$"), "write"),
    # a comment
    (re.compile(r"^\s*(?://[/!]?|\*)"), "comment"),
)


def classify_occurrence(line: str) -> str:
    for pattern, kind in _NOT_A_READ:
        if pattern.search(line):
            return kind
    return "read"


def readers(paths: list[str] | None = None) -> dict[str, list[dict[str, str]]]:
    """Every occurrence of every policy field in tracked Rust, classified."""
    paths = paths if paths is not None else tracked("*.rs")
    found: dict[str, list[dict[str, str]]] = {f: [] for f in POLICY_FIELDS}
    for path in paths:
        try:
            text = (ROOT / path).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for number, line in enumerate(text.splitlines(), start=1):
            for field in POLICY_FIELDS:
                if field in line:
                    found[field].append(
                        {"at": f"{path}:{number}", "kind": classify_occurrence(line),
                         "text": line.strip()[:120]}
                    )
    return found


# ── The verdict ledger ───────────────────────────────────────────────────────


def load_ledger(path: Path | None = None) -> dict[tuple[str, str], dict[str, str]]:
    """`pack<TAB>field<TAB>value<TAB>verdict<TAB>owner<TAB>evidence`; `#` comments.

    🔴 THE VALUE IS IN THE KEY BECAUSE THE FIRST VERSION OF THIS FILE LEFT IT
    OUT, and the instrument then carried the exact defect it was built to find.
    Keyed by `(pack, field)` alone, a verdict outlived the line that earned it:
    flipping R3's `subresource_policy` from `deny` to `allow` in the producer
    left the census GREEN, still reporting `enforced — SIGNOFF-REPAIR.7.3.5`
    for a line advertising the opposite of what that leaf repaired. Measured
    in situ, then restored byte-identical.

    ⭐ That is `CLAIM_VERIFICATION.md` §5B's derived-constant rule: a judgement
    about a value is gated on the value, or it is a comment that says *do not
    edit by hand*.
    """
    path = path if path is not None else ROOT / LEDGER
    ledger: dict[tuple[str, str], dict[str, str]] = {}
    if not path.exists():
        return ledger
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 5:
            continue
        pack, field, value, verdict, owner = (p.strip() for p in parts[:5])
        # ⛔ FAIL CLOSED ON A MALFORMED ROW, and the arm that demanded this
        # caught the loader reading a five-column row — the shape this file had
        # before the value column — with every field shifted by one: the
        # verdict became the value, the owner became the verdict, and the gate
        # passed. Dropping the row instead leaves its line UNRECONCILED, so the
        # refusal names the advertised line rather than hiding behind it.
        if verdict not in VERDICTS:
            continue
        ledger[(pack, field)] = {
            "value": value,
            "verdict": verdict,
            "owner": owner,
            "evidence": parts[5].strip() if len(parts) > 5 else "",
        }
    return ledger


def collect() -> list[dict[str, object]]:
    """Both populations, each line joined to its verdict."""
    rows = parse_rust((ROOT / RUST_ADVERTISE).read_text(encoding="utf-8"))
    for path in tracked("migrations/*.sql"):
        text = (ROOT / path).read_text(encoding="utf-8")
        if "resolver_capabilities" in text:
            rows.extend(parse_sql(path, text))
    return join(rows, load_ledger())


def join(rows: list[dict[str, object]],
         ledger: dict[tuple[str, str], dict[str, str]]) -> list[dict[str, object]]:
    """Attach each advertised line's verdict — or say why it has none."""
    for row in rows:
        entry = ledger.get((str(row["pack"]), str(row["field"])))
        if entry is None:
            row["verdict"], row["owner"], row["evidence"] = "UNRECONCILED", "", ""
        elif entry["value"] != str(row["value"]):
            # The judgement was earned for a DIFFERENT advertised value, so it
            # does not transfer. Named as its own state rather than folded into
            # UNRECONCILED: a reader needs to see what the verdict was for.
            row["verdict"] = "STALE"
            row["owner"] = entry["owner"]
            row["evidence"] = (f"the verdict {entry['verdict']!r} was earned for "
                               f"{entry['value']!r}, the producer now advertises "
                               f"{row['value']!r}")
        else:
            row["verdict"] = entry["verdict"]
            row["owner"] = entry["owner"]
            row["evidence"] = entry["evidence"]
    return rows


def report(rows: list[dict[str, object]]) -> None:
    packs = sorted({str(r["pack"]) for r in rows})
    rust = [r for r in rows if r["origin"] == "rust"]
    sql = [r for r in rows if r["origin"] == "sql"]
    print(f"advertised policy lines: {len(rows)} across {len(packs)} packs "
          f"({len(rust)} Rust literals, {len(sql)} migration rows) — NOT one population")
    width = max((len(str(r["pack"])) for r in rows), default=4)
    for pack in packs:
        origin = next(str(r["origin"]) for r in rows if r["pack"] == pack)
        print(f"\n  {pack}  ({origin})")
        for row in [r for r in rows if r["pack"] == pack]:
            mark = "??" if row["verdict"] == "UNRECONCILED" else "  "
            print(f"   {mark} {str(row['field']):<20} {str(row['value']):<18} "
                  f"{str(row['verdict']):<13} {row['owner']}")
    del width
    counts: dict[str, int] = {}
    for row in rows:
        counts[str(row["verdict"])] = counts.get(str(row["verdict"]), 0) + 1
    print("\n  verdicts: " + " · ".join(f"{v} {n}" for v, n in sorted(counts.items())))


def report_readers() -> None:
    found = readers()
    print("what consumes each advertised policy field, over tracked Rust:")
    for field in POLICY_FIELDS:
        kinds: dict[str, int] = {}
        for hit in found[field]:
            kinds[hit["kind"]] = kinds.get(hit["kind"], 0) + 1
        reads = [h for h in found[field] if h["kind"] == "read"]
        print(f"\n  {field}: {len(found[field])} occurrences — "
              + " · ".join(f"{k} {n}" for k, n in sorted(kinds.items())))
        for hit in reads:
            print(f"      read  {hit['at']}")
        if not reads:
            print("      (no read — the column is written and never consulted)")


def check(rows: list[dict[str, object]]) -> int:
    missing = [r for r in rows if r["verdict"] == "UNRECONCILED"]
    stale = [r for r in rows if r["verdict"] == "STALE"]
    bad = [r for r in rows
           if r["verdict"] not in VERDICTS and r["verdict"] not in ("UNRECONCILED", "STALE")]
    if not missing and not stale and not bad:
        print(f"ADVERTISED-POLICY: {len(rows)} advertised lines, every one carries a verdict")
        return 0
    for row in missing:
        print(f"ADVERTISED-POLICY: no verdict for {row['pack']} {row['field']} "
              f"= {row['value']!r} ({row['at']})", file=sys.stderr)
    for row in stale:
        print(f"ADVERTISED-POLICY: {row['pack']} {row['field']} ({row['at']}) — "
              f"{row['evidence']}", file=sys.stderr)
    for row in bad:
        print(f"ADVERTISED-POLICY: {row['pack']} {row['field']} carries the unknown "
              f"verdict {row['verdict']!r}", file=sys.stderr)
    print(f"""
  A pack advertises six policy lines to every caller that reads the §12.2
  registry, and a caller choosing a pack reads all of them. An advertised line
  with no verdict is a claim nobody has checked — the exact shape
  SIGNOFF-REPAIR.7.3.5 found twice and repaired. A CHANGED value is the same
  problem one step later: the judgement was earned for the old word.

  Add or re-earn the line in {LEDGER} with one of:
      {' · '.join(sorted(VERDICTS))}
  and the leaf that earned it. Do NOT write `enforced` without a control that
  has been observed RED.""", file=sys.stderr)
    return 1


def self_test() -> int:
    arms: list[tuple[str, bool]] = []

    # ── The Rust parser ──────────────────────────────────────────────────────
    sample = '''
    pub const R5_RESOLVER_ID: &str = "r5-credential-broker";
    vec![
        ResolverAdvertise {
            resolver_id: R5_RESOLVER_ID.to_owned(),
            egress_class: "listed".to_owned(),
            sandbox_level: "none".to_owned(),
            redirect_policy: "follow-classified".to_owned(),
            archive_policy: "deny".to_owned(),
            subresource_policy: "deny".to_owned(),
            javascript_policy: "deny".to_owned(),
            latency_range_ms: serde_json::json!({ "min": 200, "max": 5000 }),
            security_evidence: serde_json::json!({ "broker": "local" }),
        },
        ResolverAdvertise {
            resolver_id: RZ_RESOLVER_ID.to_owned(),
            egress_class: "any".to_owned(),
            sandbox_level: "vm_container".to_owned(),
            redirect_policy: "deny".to_owned(),
            archive_policy: "deny".to_owned(),
            subresource_policy: "deny".to_owned(),
            javascript_policy: "allow-bounded".to_owned(),
            security_evidence: serde_json::json!({ "nested": { "deeper": true } }),
        },
    ]
    '''
    parsed = parse_rust(sample)
    # 1 ⭐ TWO packs, not one. The nested `json!({ … })` is why this arm exists:
    #   a non-greedy `\\{(.*?)\\}` stops inside `latency_range_ms` and a greedy
    #   one merges both packs into the first.
    arms.append(("the Rust parser separates two packs across a nested json! literal",
                 sorted({str(r["pack"]) for r in parsed})
                 == ["RZ_RESOLVER_ID", "r5-credential-broker"]))
    # 2 every pack yields all six fields
    arms.append(("each parsed pack yields all six policy fields",
                 len(parsed) == 12))
    # 3 ⭐ the pack is keyed by the id a CALLER sees, so the two populations
    #   join on one key rather than on a Rust constant no registry row carries.
    arms.append(("a resolvable constant is reported as the id a caller reads",
                 any(r["pack"] == "r5-credential-broker" for r in parsed)))
    # 4 ⛔ NEGATIVE — an UNRESOLVABLE constant keeps its own spelling and is NOT
    #   dropped. A pack missing from the census is the one failure it must not
    #   have, and silently discarding an unparsed id is how that happens.
    arms.append(("an unresolvable constant is kept, not dropped",
                 any(r["pack"] == "RZ_RESOLVER_ID" and r["field"] == "javascript_policy"
                     and r["value"] == "allow-bounded" for r in parsed)))
    # 5 ⛔ NEGATIVE — the second pack's value is not the first pack's. A greedy
    #   brace match passes arms 1–4 and fails this one.
    arms.append(("the second pack keeps its own values",
                 {(str(r["field"]), str(r["value"])) for r in parsed if r["pack"] == "RZ_RESOLVER_ID"}
                 >= {("egress_class", "any"), ("sandbox_level", "vm_container")}))

    # ── The SQL parser ───────────────────────────────────────────────────────
    sql = """
INSERT INTO resolver_capabilities (
    resolver_id, egress_class, sandbox_level, redirect_policy,
    archive_policy, subresource_policy, javascript_policy, latency_range_ms
) VALUES (
    'r0-https-fetcher',
    'listed',
    'none',
    'follow-classified',
    'deny',
    'deny',
    'deny',
    '{"min": 100, "max": 1000}'::jsonb
);
"""
    sql_rows = parse_sql("migrations/x.sql", sql)
    arms.append(("the SQL parser binds values to columns by name",
                 {(str(r["field"]), str(r["value"])) for r in sql_rows}
                 == {("egress_class", "listed"), ("sandbox_level", "none"),
                     ("redirect_policy", "follow-classified"), ("archive_policy", "deny"),
                     ("subresource_policy", "deny"), ("javascript_policy", "deny")}))
    # 6 ⛔ NEGATIVE — REORDER the column list and the values with it. A census
    #   keyed to a fixed index reports two policies transposed and stays green.
    reordered = sql.replace(
        "resolver_id, egress_class, sandbox_level, redirect_policy,\n    archive_policy, subresource_policy, javascript_policy, latency_range_ms",
        "resolver_id, javascript_policy, sandbox_level, redirect_policy,\n    archive_policy, subresource_policy, egress_class, latency_range_ms",
    )
    reordered_rows = {(str(r["field"]), str(r["value"]))
                      for r in parse_sql("migrations/x.sql", reordered)}
    arms.append(("a reordered column list reports the reordered values",
                 ("javascript_policy", "listed") in reordered_rows
                 and ("egress_class", "deny") in reordered_rows))
    # 7 ⛔ a comma inside a quoted jsonb value must not split a value
    arms.append(("a comma inside a quoted value does not split it",
                 len(_split_values("'a', '{\"min\": 1, \"max\": 2}'::jsonb, 'b'")) == 3))

    # ── The reader classification ────────────────────────────────────────────
    arms.append(("a struct field declaration is not a read",
                 classify_occurrence("    pub redirect_policy: String,") == "declaration"))
    arms.append(("a bind in register() is not a read",
                 classify_occurrence("    .bind(&advertise.redirect_policy)") == "write"))
    arms.append(("an advertise literal is not a read",
                 classify_occurrence('            javascript_policy: "deny".to_owned(),') == "write"))
    arms.append(("a comment is not a read",
                 classify_occurrence('//! `redirect_policy: "deny"`, and since') == "comment"))
    # 12 ⭐ THE ARM THE CLAIM RESTS ON. The census's headline is that four
    #    fields are consumed by nothing; if a genuine consumption were
    #    classified as a write, that headline would be manufactured.
    arms.append(("a genuine consumption IS a read",
                 classify_occurrence("    if row.redirect_policy == \"deny\" {") == "read"))
    arms.append(("a match on the field IS a read",
                 classify_occurrence("    match advertise.javascript_policy.as_str() {") == "read"))
    # 14 ⛔ NEGATIVE — an occurrence no rule explains defaults to `read`, so an
    #    unrecognised consumption weakens the finding rather than hiding it.
    arms.append(("an unrecognised occurrence defaults to read",
                 classify_occurrence("    frobnicate(subresource_policy);") == "read"))

    # ── The ledger and the gate ──────────────────────────────────────────────
    import tempfile
    with tempfile.TemporaryDirectory(dir=ROOT / "target") as scratch:
        ledger = Path(scratch) / "ledger.tsv"
        ledger.write_text(
            "# a comment\n\nR5\tegress_class\tlisted\tvacuous\tLEAF.1\tbecause\n",
            encoding="utf-8",
        )
        loaded = load_ledger(ledger)
        arms.append(("the ledger skips comments and blank lines", len(loaded) == 1))
        arms.append(("the ledger carries the value, the verdict, the owner and the evidence",
                     loaded[("R5", "egress_class")]
                     == {"value": "listed", "verdict": "vacuous", "owner": "LEAF.1",
                         "evidence": "because"}))
        # 18 ⛔ a five-column row from before the value column is DROPPED rather
        #    than read with its fields shifted by one — which would silently
        #    turn a verdict into a value and pass the gate.
        short = Path(scratch) / "short.tsv"
        short.write_text("R5\tegress_class\tvacuous\tLEAF.1\tbecause\n", encoding="utf-8")
        arms.append(("a row without the value column is dropped, not shifted",
                     load_ledger(short) == {}))
    covered = [{"pack": "R5", "field": "egress_class", "value": "listed",
                "origin": "rust", "at": "x:1", "verdict": "vacuous", "owner": "L", "evidence": ""}]
    arms.append(("the gate passes a fully-reconciled census", check(covered) == 0))
    # 20 ⛔ NEGATIVE, observed RED: an advertised line with no verdict refuses.
    uncovered = [dict(covered[0], verdict="UNRECONCILED")]
    arms.append(("the gate REFUSES a line with no verdict", check(uncovered) == 1))
    # 21 ⛔ NEGATIVE: a verdict outside the vocabulary refuses too, or the
    #    ledger becomes free text and the gate becomes a spell-checker for `\t`.
    arms.append(("the gate REFUSES a verdict outside the vocabulary",
                 check([dict(covered[0], verdict="probably-fine")]) == 1))
    # 22 🔴 THE ARM THAT WAS MISSING, and its absence was the instrument
    #    carrying the very defect it exists to find. A verdict earned for one
    #    advertised value must not survive the value changing: R3's
    #    `subresource_policy` flipped `deny` → `allow` in the producer and the
    #    census stayed GREEN, still reporting `enforced — .7.3.5`.
    arms.append(("the gate REFUSES a verdict earned for a different value",
                 check([dict(covered[0], verdict="STALE")]) == 1))
    # 23 ⭐ and the STALE state is PRODUCED by the join, not merely accepted by
    #    the gate — arm 22 alone stays green under a join that never emits it.
    #    This is the flip that was measured in situ, replayed as a fixture.
    earned = {("r3", "subresource_policy"):
              {"value": "deny", "verdict": "enforced", "owner": ".7.3.5", "evidence": "red"}}
    flipped = join([{"pack": "r3", "field": "subresource_policy", "value": "allow",
                     "origin": "rust", "at": "x:1"}], earned)
    arms.append(("the join marks a flipped value STALE, not enforced",
                 flipped[0]["verdict"] == "STALE"))
    # 24 ⛔ and the unflipped line still earns its verdict, or arm 23 is passed
    #    by a join that marks everything STALE.
    unflipped = join([{"pack": "r3", "field": "subresource_policy", "value": "deny",
                       "origin": "rust", "at": "x:1"}], earned)
    arms.append(("the join still verdicts an unchanged value",
                 unflipped[0]["verdict"] == "enforced"))

    # ── The live corpus ──────────────────────────────────────────────────────
    # 20 ⭐ LIVE-CORPUS ARM (TOOLBOX.md: give every census one). Every arm above
    #    runs on a fixture, and a fixture cannot tell you the real producer
    #    still parses — the defect that ends a census's useful life.
    try:
        live = collect()
        live_packs = {str(r["pack"]) for r in live}
        live_ok = len(live) >= 12 and len(live_packs) >= 2 and all(
            str(r["field"]) in POLICY_FIELDS for r in live
        )
    except Exception as exc:                                       # noqa: BLE001
        live_ok = False
        print(f"census_advertised_policies: live-corpus arm raised {exc!r}")
    arms.append(("live-corpus: the real producers parse and yield policy lines", live_ok))
    # 21 ⭐ and BOTH producers contribute, or a parser could rot silently while
    #    the other carried the count.
    try:
        origins = {str(r["origin"]) for r in collect()}
    except Exception:                                              # noqa: BLE001
        origins = set()
    arms.append(("live-corpus: both populations are non-empty",
                 origins == {"rust", "sql"}))

    passed = sum(1 for _, ok in arms if ok)
    for name, ok in arms:
        print(f"census_advertised_policies: {'arm ok' if ok else 'arm FAILED'} — {name}")
    print(f"census_advertised_policies --self-test: {passed}/{len(arms)} controls pass")
    return 0 if passed == len(arms) else 1


def main(argv: list[str]) -> int:
    mode = argv[1] if len(argv) > 1 else ""
    if mode == "--self-test":
        return self_test()
    if mode == "--readers":
        report_readers()
        return 0
    rows = collect()
    if mode == "--check":
        return check(rows)
    if mode == "--json":
        print(json.dumps(rows, indent=2, sort_keys=True))
        return 0
    report(rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))

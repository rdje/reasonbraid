#!/usr/bin/env bash
# scripts/check_action_boundary.sh — ACTION-BOUNDARY doctrine.
#
# Blocker B3 is the §16.6 prompt-injection action-boundary suite. It has been
# DEFERRED for the life of the project on a measured, correct ground: model
# output cannot reach an action, so there is nothing for such a suite to attach
# to. `SIGNOFF-REPAIR.13.1` measured that rather than inheriting it.
#
# ⛔ THE PROBLEM THIS GATE SOLVES IS NOT THE DEFERRAL. IT IS THAT THE DEFERRAL'S
# REVISIT TRIGGER IS PROSE.
#
# `SIGNOFF-REPAIR.11.4.7.2` measured what that costs. The Phase-1 gate record
# deferred a fuzz baseline with the trigger "the first untrusted parser — Phase
# 4's resource packs". Phase 4 shipped, its gate record closed, `fetcher.rs` and
# `git.rs` now parse untrusted input — and `git ls-files | grep -ic fuzz`
# returns 0. The word `fuzz` appears in exactly ONE decision record: the one
# that deferred it. The trigger fired and nothing noticed, because nothing was
# built to notice.
#
# ⭐ A deferral with a trigger nobody checks is an omission with extra steps.
# This gate makes B3's trigger evaluable by a script, so the day the action
# boundary moves, the commit that moves it fails and says B3 is now DUE.
#
# WHAT HOLDS THE BOUNDARY TODAY, measured at `SIGNOFF-REPAIR.13.1` and re-read
# here rather than carried:
#
#   1. `claude.rs::EXEC_ARGS` passes `--restricted` (removes the code-running
#      tools and WebFetch) and `--tools ''` (empties the tool set).
#   2. `codex.rs::EXEC_ARGS` passes `--sandbox read-only`.
#   3. `threads::work_payload` dispatches only `kind`, `agent_role`, `subject`,
#      `objective`, `reservation`, `reservation_reason`,
#      `allow_possible_duplicate` and an optional `target_event_id` — no
#      acquired bytes, no evidence, no derivations. Acquisition terminates in
#      STORED EVIDENCE rather than in a dispatched run.
#   4. Every `Adapter` implementer DECLARES `tool_support: false`. ⭐ This is the
#      real control, and `SIGNOFF-REPAIR.13.1.1` is why it is pinned here rather
#      than left to the admission ladder: `verify_ladder` — the five-rung
#      fail-closed check that WOULD refuse a tool-declaring adapter — has no
#      production caller (six of its eight call sites are in its own test
#      module), and `AllowedCapabilities::dev()`, the crate's only ceiling,
#      permits `tool_support: true` anyway. So nothing enforces the declaration
#      at runtime, and this gate enforces it at commit time instead.
#   5. Exactly four types implement `Adapter`, and only two of them are real
#      provider CLIs. A fifth is how this gate gets bypassed, so the census is
#      pinned too.
#
# Any of those four moving means model output may now reach an action, and the
# suite B3 names stops being unattachable.
#
# ⚠️ HONEST LIMITS, stated rather than hidden:
#   - This gate does NOT prove the boundary holds. It proves the four facts the
#     boundary argument RESTS ON are unchanged. If the argument was wrong, this
#     gate is wrong with it — which is why it names `.13.1` as its source.
#   - It is textual. A flag passed through a variable rather than the pinned
#     const array would not be seen. Pinning the CONST is what makes it cheap
#     and readable; widening it to dataflow would make it a different tool.
#   - It says nothing about the acquisition packs (`fetcher.rs`, `git.rs`,
#     `browse.rs`, `extraction.rs`). Those are the UNTRUSTED-INPUT side, owned
#     by `.7.2` and by the fuzz deferral, not the model-action side.
#
# Self-test: scripts/check_action_boundary.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

python3 -B - "${1:-}" <<'PY'
import subprocess, sys, pathlib, re

SELF_TEST = (sys.argv[1] if len(sys.argv) > 1 else "") == "--self-test"

CLAUDE = "crates/reasonbraid-adapter/src/claude.rs"
CODEX = "crates/reasonbraid-adapter/src/codex.rs"
THREADS = "crates/reasonbraid-server/src/threads.rs"
ADAPTER_SRC = "crates/reasonbraid-adapter/src"

# The payload keys the dispatch is allowed to carry. A NEW key is the breach:
# it is how "acquired bytes" or "evidence" would first reach a run.
PINNED_PAYLOAD_KEYS = {
    "kind", "agent_role", "subject", "objective",
    "reservation", "reservation_reason", "allow_possible_duplicate",
    "target_event_id",
}
PINNED_ADAPTER_IMPLS = 4
# Files that construct an AdapterCapabilities and must declare no tool support.
TOOL_DECL_FILES = [
    "crates/reasonbraid-adapter/src/claude.rs",
    "crates/reasonbraid-adapter/src/codex.rs",
    "crates/reasonbraid-adapter/src/bench/scripted.rs",
    "crates/reasonbraid-adapter/src/contract.rs",
    "crates/reasonbraid-node/src/bin/rb-node.rs",
]


def exec_args(src, marker="const EXEC_ARGS"):
    """The string literals of the EXEC_ARGS const array, in order."""
    i = src.find(marker)
    if i < 0:
        return None
    j = src.find("];", i)
    if j < 0:
        return None
    return re.findall(r'"((?:[^"\\]|\\.)*)"', src[i:j])


def payload_keys(src):
    """The JSON keys `work_payload` writes: the json! block plus any
    payload["..."] assignment inside the function.

    ⛔ The body is delimited by BRACE MATCHING from the function's opening
    brace, not by scanning for the next item. The first draft of this gate used
    `src.find("\\npub ", i)` as the terminator; the next item is
    `pub(crate) async fn`, which does not match, so the slice ran to the end of a
    5,000-line file and the gate reported 26 phantom "new fields" from unrelated
    functions. A terminator that depends on how the NEXT declaration happens to be
    spelled is not a delimiter."""
    i = src.find("pub fn work_payload")
    if i < 0:
        return None
    open_i = src.find("{", i)
    if open_i < 0:
        return None
    depth, j = 0, open_i
    while j < len(src):
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    body = src[open_i:j + 1]
    keys = set(re.findall(r'"([a-z_]+)"\s*:', body))
    keys |= set(re.findall(r'payload\[\s*"([a-z_]+)"\s*\]', body))
    return keys


def tool_declarations(sources):
    """{path: [each `tool_support: <value>` it declares]}."""
    return {p: re.findall(r"tool_support:\s*(true|false)", src)
            for p, src in sources.items()}


def audit(claude_args, codex_args, keys, impls, decls=None):
    """Pure: every input is passed in, so the self-test drives the same
    function the real run does."""
    b = []
    DUE = ("— an action-bearing surface may have landed, so blocker B3 "
           "(the §16.6 prompt-injection action-boundary suite) is now DUE")

    if claude_args is None:
        b.append(f"{CLAUDE}: no EXEC_ARGS const array found {DUE}")
    else:
        if "--restricted" not in claude_args:
            b.append(f"{CLAUDE}: EXEC_ARGS no longer passes --restricted {DUE}")
        if "--tools" not in claude_args:
            b.append(f"{CLAUDE}: EXEC_ARGS no longer passes --tools {DUE}")
        else:
            k = claude_args.index("--tools")
            if k + 1 >= len(claude_args) or claude_args[k + 1] != "":
                got = claude_args[k + 1] if k + 1 < len(claude_args) else "<nothing>"
                b.append(f"{CLAUDE}: --tools is followed by {got!r}, not the "
                         f"empty tool set {DUE}")

    if codex_args is None:
        b.append(f"{CODEX}: no EXEC_ARGS const array found {DUE}")
    elif "--sandbox" not in codex_args:
        b.append(f"{CODEX}: EXEC_ARGS no longer passes --sandbox {DUE}")
    else:
        k = codex_args.index("--sandbox")
        got = codex_args[k + 1] if k + 1 < len(codex_args) else "<nothing>"
        if got != "read-only":
            b.append(f"{CODEX}: --sandbox is {got!r}, not read-only {DUE}")

    if keys is None:
        b.append(f"{THREADS}: work_payload not found {DUE}")
    else:
        new = sorted(keys - PINNED_PAYLOAD_KEYS)
        if new:
            b.append(f"{THREADS}: work_payload dispatches new field(s) "
                     f"{', '.join(new)} {DUE}. If the field carries acquired "
                     f"bytes, evidence or a derivation, model output can now "
                     f"reach a run. If it does not, widen PINNED_PAYLOAD_KEYS "
                     f"in this gate and say why in the leaf")
        gone = sorted(PINNED_PAYLOAD_KEYS - keys - {"target_event_id"})
        if gone:
            b.append(f"{THREADS}: work_payload no longer carries "
                     f"{', '.join(gone)} — this gate's pin is stale and is "
                     f"describing a function that changed under it")

    for path, values in sorted((decls or {}).items()):
        if not values:
            b.append(f"{path}: declares no tool_support at all — this gate can no "
                     f"longer see the declaration it depends on {DUE}")
        for v in values:
            if v != "false":
                b.append(f"{path}: declares tool_support: {v} {DUE}. ⛔ Nothing "
                         f"refuses this at runtime — verify_ladder has no production "
                         f"caller and the dev ceiling permits it (SIGNOFF-REPAIR.13.1.1)")

    if impls != PINNED_ADAPTER_IMPLS:
        b.append(f"{ADAPTER_SRC}: {impls} types implement Adapter, pinned at "
                 f"{PINNED_ADAPTER_IMPLS}. A new adapter is how this gate is "
                 f"bypassed — confirm its sandbox/tool posture, then re-pin {DUE}")
    return b


if SELF_TEST:
    OK_CLAUDE = ["-p", "--output-format", "stream-json", "--restricted",
                 "--tools", "", "--verbose", "--"]
    OK_CODEX = ["exec", "--json", "--skip-git-repo-check", "--ephemeral",
                "--sandbox", "read-only"]
    OK_KEYS = set(PINNED_PAYLOAD_KEYS)

    OK_DECLS = {"crates/x.rs": ["false"], "crates/y.rs": ["false", "false"]}

    cases = [
        ("today's boundary", OK_CLAUDE, OK_CODEX, OK_KEYS, 4, False),
        ("an adapter declares tool support",
         OK_CLAUDE, OK_CODEX, OK_KEYS, 4, True,
         {"crates/x.rs": ["false"], "crates/y.rs": ["true"]}),
        ("a file stops declaring tool_support at all",
         OK_CLAUDE, OK_CODEX, OK_KEYS, 4, True, {"crates/x.rs": []}),
        ("claude drops --restricted",
         [a for a in OK_CLAUDE if a != "--restricted"], OK_CODEX, OK_KEYS, 4, True),
        ("claude's --tools gains a tool",
         ["-p", "--restricted", "--tools", "Bash", "--"], OK_CODEX, OK_KEYS, 4, True),
        ("claude drops --tools entirely",
         ["-p", "--restricted", "--"], OK_CODEX, OK_KEYS, 4, True),
        ("--tools is last, with nothing after it",
         ["-p", "--restricted", "--tools"], OK_CODEX, OK_KEYS, 4, True),
        ("codex sandbox widened to workspace-write",
         OK_CLAUDE, ["exec", "--sandbox", "workspace-write"], OK_KEYS, 4, True),
        ("codex drops --sandbox", OK_CLAUDE, ["exec", "--json"], OK_KEYS, 4, True),
        ("EXEC_ARGS const removed", None, OK_CODEX, OK_KEYS, 4, True),
        ("work_payload gains an evidence field",
         OK_CLAUDE, OK_CODEX, OK_KEYS | {"acquired_evidence"}, 4, True),
        ("work_payload loses a pinned field",
         OK_CLAUDE, OK_CODEX, OK_KEYS - {"reservation"}, 4, True),
        # target_event_id is conditional, so its absence must NOT fire
        ("optional target_event_id absent",
         OK_CLAUDE, OK_CODEX, OK_KEYS - {"target_event_id"}, 4, False),
        ("a fifth Adapter implementer appears",
         OK_CLAUDE, OK_CODEX, OK_KEYS, 5, True),
        ("work_payload not found", OK_CLAUDE, OK_CODEX, None, 4, True),
    ]
    fails = 0
    for case in cases:
        label, ca, co, k, n, want = case[:6]
        decls = case[6] if len(case) > 6 else OK_DECLS
        got = bool(audit(ca, co, k, n, decls))
        if got != want:
            print(f"SELF-TEST: {label!r} -> breach={got}, expected {want}",
                  file=sys.stderr)
            fails += 1

    # The parsers must survive the real files, not just literal lists.
    for path, fn, label in ((CLAUDE, exec_args, "claude EXEC_ARGS"),
                            (CODEX, exec_args, "codex EXEC_ARGS")):
        p = pathlib.Path(path)
        if not p.is_file():
            print(f"SELF-TEST: {path} absent", file=sys.stderr); fails += 1; continue
        if not fn(p.read_text()):
            print(f"SELF-TEST: {label} parsed empty from the real file",
                  file=sys.stderr)
            fails += 1
    # ⛔ The real-file probe asserts the EXACT key set, not merely a non-empty
    # one. "Parsed something" is what let the brace-matching bug above ship: the
    # broken parser returned 30-odd keys and passed a non-empty check happily.
    tp = pathlib.Path(THREADS)
    if tp.is_file():
        got = payload_keys(tp.read_text())
        if got is None:
            print("SELF-TEST: work_payload not found in the real file",
                  file=sys.stderr)
            fails += 1
        elif got != PINNED_PAYLOAD_KEYS:
            extra = sorted(got - PINNED_PAYLOAD_KEYS)
            missing = sorted(PINNED_PAYLOAD_KEYS - got)
            print(f"SELF-TEST: work_payload parsed {len(got)} keys from the real "
                  f"file, expected {len(PINNED_PAYLOAD_KEYS)}; extra={extra} "
                  f"missing={missing}. If the function genuinely changed, this is "
                  f"the real run's job to report — but a parse that drifts is "
                  f"this probe's.", file=sys.stderr)
            fails += 1

    if fails:
        print(f"ACTION-BOUNDARY: --self-test FAILED ({fails})", file=sys.stderr)
        sys.exit(1)
    print(f"ACTION-BOUNDARY: --self-test ok ({len(cases)} audit cases, "
          f"3 real-file parses)")
    sys.exit(0)

# --- the real run ---
def read(path):
    p = pathlib.Path(path)
    return p.read_text(encoding="utf-8", errors="replace") if p.is_file() else ""

impls = len(subprocess.run(
    ["git", "grep", "-c", "-E", r"^impl Adapter for ", "--", ADAPTER_SRC],
    capture_output=True, text=True).stdout.strip().splitlines())

decls = tool_declarations({p: read(p) for p in TOOL_DECL_FILES})
breaches = audit(exec_args(read(CLAUDE)), exec_args(read(CODEX)),
                 payload_keys(read(THREADS)), impls, decls)

if breaches:
    print("ACTION-BOUNDARY: the facts B3's deferral rests on have MOVED",
          file=sys.stderr)
    for b in breaches:
        print(f"  - {b}", file=sys.stderr)
    print("  Owner: SIGNOFF-REPAIR.13.1 (blocker B3). Re-read "
          "docs/book/src/blockers.md before widening any pin here.",
          file=sys.stderr)
    sys.exit(1)

print(f"ACTION-BOUNDARY: OK — claude --restricted --tools ''; codex --sandbox "
      f"read-only; work_payload carries no acquired bytes; "
      f"{sum(len(v) for v in decls.values())} tool_support declarations, all "
      f"false; {impls} Adapter implementers. B3 remains correctly deferred.")
PY

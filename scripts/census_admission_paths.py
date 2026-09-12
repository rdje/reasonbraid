#!/usr/bin/env python3
"""The route-level admission census, re-derivable at any commit.

`SIGNOFF-REPAIR.3.3.4.1` censused the authority WRITERS and their callers by
name; `scripts/census_authority_paths.py` is that instrument. This is the other
half, which `SIGNOFF-REPAIR.3.3.4.13` needs: for every registered HTTP route,
HOW is the caller admitted, and does the route MUTATE?

    python3 -B scripts/census_admission_paths.py            # the working tree
    python3 -B scripts/census_admission_paths.py --self-test # the ground truth
    python3 -B scripts/census_admission_paths.py --json      # machine-readable

⛔ It is TRANSITIVE over locally defined functions, and that is not a refinement
— it is the difference between a true and a false answer. A first version of this
census classified `/v1/admin/grants/{grant_id}/revoke` as unadmitted because the
handler's own body only calls `run_revocation`; the admission is one level down.
Five routes were wrong that way. The closure below follows local calls to a fixed
point, so a handler that delegates is classified by what it actually reaches.

⛔ The closure stays inside `api.rs`, and the reason is a measurement rather than
a preference. An earlier version followed calls across the whole crate so that it
could find the `INSERT`/`UPDATE` statements, which now live in `authority/*.rs`
and the service modules. At that scope a lexical name-matching closure
over-approximates to uselessness: it classified 53 of 118 routes as reaching the
thread-command path and 41 as reaching a guarded transaction, because once the
closure leaves the handler file it reaches shared helpers that reach everything.
Both numbers are false. The instrument therefore does NOT try to answer "does
this route mutate" by reading code; it answers it from the route's own declared
HTTP verb, which is a fact the router states rather than one this script infers.

Its limits are the original census's, stated rather than discovered: it is
lexical, not an AST pass; it follows calls by NAME, so same-named functions in
different modules are conflated (`api.rs` wins, and the direction of that error
is over-approximation); and it sees neither dynamic dispatch nor SQL built at
runtime. It classifies ROUTES, and it cannot tell you whether a gate is the RIGHT
one — only which gate is reached.
"""
import json
import re
import subprocess
import sys

SOURCE = "crates/reasonbraid-server/src/api.rs"

FN = re.compile(r"^(?:pub(?:\(\w+\))?\s+)?(?:async\s+)?fn\s+(\w+)")
CALL = re.compile(r"\b(\w+)\s*\(")
ROUTE = re.compile(r'\.route\(\s*"([^"]+)",\s*((?:[a-z]+\(\w+\)\.?)+)', re.S)
VERB = re.compile(r"([a-z]+)\((\w+)\)")

# Ordered most specific first: a route reaching several gates is named by the
# strongest thing it reaches, because that is the gate that decides.
GATES = [
    ("guarded transaction", ("authorize_in_tx", "_in_one_transaction", "enroll_in_guard")),
    ("command path", ("apply_authorized_command", "run_thread_command")),
    ("pool tenant-admin inspection", ("authorize_tenant_admin_inspection",)),
    ("pool tenant-admin", ("authorize_tenant_admin",)),
    ("pool authorize", ("authorize_guarded",)),
    ("pool profile read gate", ("authorize_profile_read", "classify_reader")),
    ("identity only", ("resolve_principal",)),
]


def split_functions(text):
    lines = text.splitlines()
    marks = [(i, m.group(1)) for i, l in enumerate(lines) for m in [FN.match(l)] if m]
    bodies = {}
    for k, (i, name) in enumerate(marks):
        end = marks[k + 1][0] if k + 1 < len(marks) else len(lines)
        bodies[name] = "\n".join(lines[i:end])
    return bodies


def reachable(name, bodies, seen=None):
    """Every local function `name` reaches, itself included."""
    seen = seen if seen is not None else set()
    if name in seen or name not in bodies:
        return seen
    seen.add(name)
    for callee in set(CALL.findall(bodies[name])):
        if callee in bodies:
            reachable(callee, bodies, seen)
    return seen


def classify(text):
    bodies = split_functions(text)
    rows = []
    for path, spec in ROUTE.findall(text):
        for verb, handler in VERB.findall(spec):
            closure = reachable(handler, bodies)
            reached = "\n".join(bodies[n] for n in closure)
            gate = "none"
            for label, needles in GATES:
                if any(n in reached for n in needles):
                    gate = label
                    break
            rows.append(
                {
                    "route": f"{verb.upper()} {path}",
                    "handler": handler,
                    "gate": gate,
                    # From the route's OWN declared verb, not from reading code.
                    # See the module docstring for why this is not lexical.
                    "mutates": verb.lower() != "get",
                    "reaches": len(closure),
                }
            )
    return sorted(rows, key=lambda r: (r["gate"], r["route"]))


def self_test():
    """The closure is the whole point, so prove it discriminates."""
    sample = """
async fn handler_direct() { resolve_principal(); authorize_in_tx(); }
async fn handler_delegating() { helper(); }
async fn helper() { authorize_in_tx(); sqlx::query("UPDATE t SET a = 1"); }
async fn handler_open() { resolve_principal(); }
"""
    bodies = split_functions(sample)
    misses = 0
    if "authorize_in_tx" not in "\n".join(bodies[n] for n in reachable("handler_delegating", bodies)):
        print("SELF-TEST MISS: the closure did not follow a delegating handler", file=sys.stderr)
        misses += 1
    if "authorize_in_tx" in bodies["handler_delegating"]:
        print("SELF-TEST MISS: the sample does not exercise indirection", file=sys.stderr)
        misses += 1
    direct = "\n".join(bodies[n] for n in reachable("handler_direct", bodies))
    if "authorize_in_tx" not in direct:
        print("SELF-TEST MISS: a direct call was not seen", file=sys.stderr)
        misses += 1
    if "authorize_in_tx" in "\n".join(bodies[n] for n in reachable("handler_open", bodies)):
        print("SELF-TEST MISS: an unadmitted handler was credited with a gate", file=sys.stderr)
        misses += 1
    # Recursion must terminate rather than blow the stack.
    recursive = "async fn a() { b(); }\nasync fn b() { a(); }\n"
    if reachable("a", split_functions(recursive)) != {"a", "b"}:
        print("SELF-TEST MISS: the closure did not terminate on a cycle", file=sys.stderr)
        misses += 1
    if misses:
        print(f"census_admission_paths: {misses} control(s) missed; refusing", file=sys.stderr)
        return 2
    print("census_admission_paths --self-test: 5/5 controls")
    return 0


def main():
    if "--self-test" in sys.argv:
        return self_test()
    rev = next((a for a in sys.argv[1:] if not a.startswith("--")), None)
    if rev:
        text = subprocess.run(
            ["git", "show", f"{rev}:{SOURCE}"], capture_output=True, check=True
        ).stdout.decode()
    else:
        text = subprocess.run(
            ["git", "show", f":{SOURCE}"], capture_output=True, check=True
        ).stdout.decode()
    rows = classify(text)
    if "--json" in sys.argv:
        print(json.dumps(rows, indent=2))
        return 0
    print(f"registered route handlers: {len(rows)}")
    for gate in sorted({r["gate"] for r in rows}):
        group = [r for r in rows if r["gate"] == gate]
        mutating = sum(1 for r in group if r["mutates"])
        print(f"\n-- {gate}: {len(group)} routes, {mutating} of them mutate")
        for r in group:
            print(f"   {'MUT ' if r['mutates'] else '    '}{r['route']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

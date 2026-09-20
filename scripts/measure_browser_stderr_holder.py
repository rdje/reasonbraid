#!/usr/bin/env python3
"""scripts/measure_browser_stderr_holder.py — name every process holding the
R3 browser worker's stderr pipe, by KERNEL PIPE IDENTITY rather than by name.

`SIGNOFF-REPAIR.11.25` measured that the worker's stderr drain — the wait for
EOF that `a_render_refusal_survives_an_unconfirmed_cleanup` relies on as its
detector — can outlast the cleanup budget, and deliberately declined to name
what was holding the pipe. `.11.25.1.1` then tried and produced no holder,
because the instruments it used reported an absence that was never checked
against a known-present case.

This instrument exists so that answer is re-derivable rather than remembered
(`docs/CLAIM_VERIFICATION.md` leg 3: a measured number whose producer is
untracked is a "trust me" with extra steps).

WHAT IT MEASURES
    The worker holds the READ end of the browser's stderr. EOF arrives only
    when every WRITE end is closed, so the question is which processes hold
    the peer endpoint. `lsof` prints each pipe endpoint's own kernel address
    and its peer's, so the peer can be resolved to its holders exactly —
    including processes whose name says nothing about who spawned them.

THE POSITIVE CONTROL IS NOT OPTIONAL
    Every sample taken while the render is running is a sample taken when the
    browser is CERTAINLY alive and its stderr CERTAINLY piped. If no such
    sample sees the pipe and a browser process, the instrument is BLIND here
    and the run reports `blind` instead of an absence — which is exactly the
    failure `.11.25.1.1` recorded as a rule:

        a probe whose conclusion is an ABSENCE owes a positive control in the
        same run.

USAGE
    python3 -B scripts/measure_browser_stderr_holder.py --self-test
    R3_BROWSER_BIN=<chrome> python3 -B scripts/measure_browser_stderr_holder.py \
        --render-secs 30 --out target/browser-stderr-holder.json

Requires the `reasonbraid-browse` worker to be built and a browser binary on
`R3_BROWSER_BIN` — the same seam the worker itself reads. Everything it writes
is derived from the repository root at runtime (§12, §13); no absolute path is
persisted and no ambient temporary directory is used.
"""

from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import sys
import threading
import time

PIPE_TYPES = ("PIPE", "FIFO")


def repository_root() -> str:
    """The repository root, derived at runtime so moving the checkout is free."""
    root = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        cwd=os.path.dirname(os.path.abspath(__file__)),
    ).stdout.strip()
    return root or os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


# ── parsing, kept pure so the self-test can drive it without a browser ────────


def parse_pipe_rows(listing: str) -> list[dict]:
    """Every pipe endpoint in an `lsof -n -P` listing: its own address and its peer.

    macOS prints the endpoint's kernel address in DEVICE and the peer's in NAME
    as `->0x…`. Linux prints neither, so a row with no peer is kept and simply
    resolves to nothing — the caller reports `blind` rather than an absence.
    """
    rows = []
    for line in listing.splitlines():
        parts = line.split()
        if len(parts) >= 6 and parts[4] in PIPE_TYPES:
            peer = next((t[2:] for t in parts[6:] if t.startswith("->0x")), "")
            rows.append(
                {
                    "command": parts[0],
                    "pid": parts[1],
                    "fd": parts[3],
                    "self": parts[5],
                    "peer": peer,
                }
            )
    return rows


def parse_process_rows(listing: str) -> dict[str, dict]:
    """`ps -Ao pid,ppid,pgid,comm` as a table keyed by pid.

    ⛔ `comm` is the EXECUTABLE PATH and is taken as the whole remainder of the
    line, because a macOS bundle path contains spaces — `Google Chrome for
    Testing.app`. Splitting a command line on whitespace to recover the binary
    yields `…/Google` on exactly the processes this instrument exists to name,
    which is how a record can look populated and answer nothing.
    """
    table = {}
    for line in listing.splitlines()[1:]:
        parts = line.split(None, 3)
        if len(parts) == 4:
            table[parts[0]] = {
                "pid": parts[0],
                "ppid": parts[1],
                "pgid": parts[2],
                "comm": parts[3],
                "binary": os.path.basename(parts[3]),
            }
    return table


def stderr_pipe_of(worker_pid: str, pipes: list[dict], processes: dict[str, dict]) -> str:
    """The peer address of the one pipe this worker shares with a browser process.

    Identified by the PEER's holders, never by the worker's fd number: the
    worker inherits stdio pipes from whatever launched it, and which fd the
    browser's stderr lands on is not a constant.
    """
    by_self: dict[str, list[str]] = {}
    for row in pipes:
        by_self.setdefault(row["self"], []).append(row["pid"])
    for row in pipes:
        if row["pid"] == worker_pid and row["peer"]:
            for holder in by_self.get(row["peer"], []):
                command = processes.get(holder, {}).get("comm", "")
                if "chrome" in command.lower():
                    return row["peer"]
    return ""


def holders_of(pipe: str, pipes: list[dict], processes: dict[str, dict]) -> list[dict]:
    """Every process holding that pipe endpoint, with the group that owns it."""
    found = []
    for row in pipes:
        if row["self"] == pipe:
            process = processes.get(row["pid"], {})
            found.append(
                {
                    "pid": row["pid"],
                    "fd": row["fd"],
                    "ppid": process.get("ppid", "?"),
                    "pgid": process.get("pgid", "?"),
                    # The executable, recovered from `comm` rather than sliced
                    # out of a command line. A Chrome helper carries ~190 bytes
                    # of bundle path before its own name.
                    "binary": process.get("binary", row["command"]),
                    "comm": process.get("comm", "")[-120:],
                }
            )
    return found


def classify(holders: list[dict], owned_group: str) -> dict:
    """Split the holders into the owned process group and the escapees.

    The escapees are the whole finding: `stop_process` kills the browser's
    process group, so a holder outside it is structurally out of reach and the
    drain cannot complete until it closes the descriptor of its own accord.
    """
    inside = [h for h in holders if h["pgid"] == owned_group]
    outside = [h for h in holders if h["pgid"] != owned_group]
    return {
        "owned_group": owned_group,
        "holders_total": len(holders),
        "holders_in_owned_group": len(inside),
        "escapees": outside,
    }


def ours_only(before: dict[str, dict], after: dict[str, dict], needle: str) -> dict[str, dict]:
    """The processes THIS launch added, by set difference against a pre-launch snapshot.

    ⛔ A count taken after a launch is not a count of that launch's children: this
    host runs other browsers, and an earlier run of this very instrument leaves
    its own. Binding by set difference is the same correction the escapee survival
    check needed — a count that is not bound to its subject answers a question
    nobody asked (`SIGNOFF-REPAIR.11.25.1`).
    """
    return {
        pid: row
        for pid, row in after.items()
        if pid not in before and needle in row.get("comm", "")
    }


# ── the live measurement ──────────────────────────────────────────────────────


def lsof_pipes() -> list[dict]:
    return parse_pipe_rows(
        subprocess.run(["lsof", "-n", "-P"], capture_output=True, text=True).stdout
    )


def ps_table() -> dict[str, dict]:
    return parse_process_rows(
        subprocess.run(
            ["ps", "-Ao", "pid,ppid,pgid,comm"], capture_output=True, text=True
        ).stdout
    )


def stalled_origin() -> tuple[socket.socket, int]:
    """A loopback peer that ACCEPTS and never answers, so the render runs its budget."""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", 0))
    server.listen(64)
    held: list[socket.socket] = []

    def accept_forever() -> None:
        while True:
            try:
                connection, _ = server.accept()
            except OSError:
                return
            held.append(connection)

    threading.Thread(target=accept_forever, daemon=True).start()
    return server, server.getsockname()[1]


def measure(root: str, render_secs: int, interval: float) -> dict:
    worker = os.path.join(root, "target/debug/reasonbraid-browse")
    if not os.path.exists(worker):
        raise SystemExit(
            "the worker is not built: "
            "python3 -B scripts/project_env.py cargo build --locked -p reasonbraid-browse"
        )
    if not os.environ.get("R3_BROWSER_BIN"):
        raise SystemExit("set R3_BROWSER_BIN to the pinned browser, as the worker reads it")

    server, port = stalled_origin()
    url = f"http://127.0.0.1:{port}/page"
    request = {
        "url": url,
        "steps": [{"action": "navigate", "url": url}],
        "limits": {
            "max_steps": 4,
            "max_output_bytes": 1048576,
            "time_budget_secs": render_secs,
        },
    }

    started = time.time()
    child = subprocess.Popen(
        [worker],
        cwd=root,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    child.stdin.write(json.dumps(request))
    child.stdin.close()

    pipe = ""
    samples = []
    while child.poll() is None:
        pipes, processes = lsof_pipes(), ps_table()
        if not pipe:
            pipe = stderr_pipe_of(str(child.pid), pipes, processes)
        sample = {"at": round(time.time() - started, 3), "pipe": pipe}
        if pipe:
            holders = holders_of(pipe, pipes, processes)
            owned = processes.get(str(child.pid), {})
            # The owned group is the browser's own: the majority holder group,
            # which `Lifetime::spawn` created with `process_group(0)`.
            groups: dict[str, int] = {}
            for holder in holders:
                groups[holder["pgid"]] = groups.get(holder["pgid"], 0) + 1
            owned_group = max(groups, key=groups.get) if groups else owned.get("pgid", "?")
            sample.update(classify(holders, owned_group))
            sample["holders"] = holders
        samples.append(sample)
        time.sleep(interval)

    stdout, stderr = child.communicate()
    # ⭐ THE RECEIPT'S OWN CLAIM, TESTED. `cleanup_confirmed` means "no owned
    # browser process or task is KNOWN TO OUTLIVE this worker", and EOF on the
    # stderr pipe is the proxy it is decided by. Those are not the same fact: an
    # escapee that CLOSES the descriptor and keeps running satisfies the proxy
    # and refutes the claim. Sampled immediately after the worker exits, bound to
    # the pids seen holding the pipe during this render.
    #
    # ⚠ A numeric pid can be recycled, so survival is only recorded when the
    # pid is still present AND still names the same executable.
    after = ps_table()
    survivors = []
    seen = {
        escapee["pid"]: escapee
        for sample in samples
        if sample.get("escapees")
        for escapee in sample["escapees"]
    }
    for pid, escapee in seen.items():
        now = after.get(pid)
        if now and now.get("binary") == escapee.get("binary"):
            survivors.append({"pid": pid, "binary": escapee.get("binary"),
                              "pgid": now.get("pgid")})
    server.close()

    receipt = None
    for line in stderr.splitlines():
        if line.startswith("browser completion: "):
            receipt = json.loads(line[len("browser completion: ") :])
    spent = (receipt or {}).get("cleanup_elapsed_ms", {})
    budget = (receipt or {}).get("cleanup_budget_ms")
    drain = None
    if "stderr" in spent and "intercept" in spent:
        drain = spent["stderr"] - spent["intercept"]

    sighted = [s for s in samples if s.get("pipe") and s.get("holders_total", 0) > 1]
    return {
        # ⛔ The positive control's verdict comes FIRST, because every other
        # field is only readable once the instrument is known to see.
        "positive_control": "sighted" if sighted else "blind",
        "positive_control_detail": (
            f"{len(sighted)} of {len(samples)} mid-render samples resolved the pipe "
            f"to more than one holder"
            if sighted
            else "no mid-render sample resolved the worker's stderr pipe to a browser "
            "holder; this instrument cannot see the subject on this host, so it "
            "reports nothing about a later absence"
        ),
        "render_budget_secs": render_secs,
        "worker_pid": child.pid,
        "stderr_pipe": pipe,
        "cleanup_confirmed": (receipt or {}).get("cleanup_confirmed"),
        "cleanup_error": (receipt or {}).get("cleanup_error"),
        "cleanup_budget_ms": budget,
        "cleanup_elapsed_ms": spent,
        # ⚠️ RIGHT-CENSORED at the budget (`SIGNOFF-REPAIR.11.25.1.1`): a drain
        # that reaches the deadline reports the REMAINING BUDGET, never a
        # duration. The flag says which this is; the number alone cannot.
        "drain_ms": drain,
        "drain_censored": bool(
            budget is not None and spent.get("stderr") is not None and spent["stderr"] >= budget
        ),
        # ⚠ The UNION over every sighted sample, not the last one. An escapee
        # that has already released the descriptor by the final sample is still
        # an escapee that held it, and reporting only the last sample turns a
        # sampling instant into a claim about the render.
        "escapees_seen": list(
            {
                escapee["pid"]: escapee
                for sample in sighted
                for escapee in sample["escapees"]
            }.values()
        ),
        # The two facts the receipt collapses into one word, reported apart.
        "escapees_outliving_the_worker": survivors,
        "receipt_claim_holds": not (
            (receipt or {}).get("cleanup_confirmed") is True and survivors
        ),
        "render_kind": (
            json.loads(stdout).get("error", {}).get("kind")
            if stdout.strip().startswith("{")
            else None
        ),
        "samples": samples,
    }


def flag_survey(binary: str, seconds: float) -> list[dict]:
    """Does any documented launch flag stop Chrome spawning the ESCAPING handler?

    The published answer — seven configurations, two handlers every time — had no
    tracked producer until this arm existed, which is exactly the leg-3 gap
    `docs/CLAIM_VERIFICATION.md` names: a refutation nobody can re-run is a
    "trust me" with extra steps.
    """
    import shutil, tempfile  # local: only this arm needs them

    base = ["--headless", "--no-sandbox", "--disable-gpu", "--no-first-run",
            "--remote-debugging-port=0", "--disable-breakpad"]
    candidates = [
        ("(baseline, as the worker launches)", []),
        ("--disable-crash-reporter", ["--disable-crash-reporter"]),
        ("--disable-crashpad", ["--disable-crashpad"]),
        ("--no-crashpad", ["--no-crashpad"]),
        ("--disable-features=Crashpad", ["--disable-features=Crashpad"]),
        ("--crash-dumps-dir=/dev/null", ["--crash-dumps-dir=/dev/null"]),
        ("--noerrdialogs --disable-logging", ["--noerrdialogs", "--disable-logging"]),
    ]
    root = repository_root()
    results = []
    for name, extra in candidates:
        # Repository-derived, exclusively created, removed by this function (§13).
        workspace = tempfile.mkdtemp(prefix="flag-", dir=os.path.join(root, "target"))
        before = ps_table()
        child = subprocess.Popen(
            [binary] + base + extra
            + [f"--user-data-dir={workspace}/profile", f"--disk-cache-dir={workspace}/cache",
               "about:blank"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(seconds)
        ours = ours_only(before, ps_table(), "chrome_crashpad_handler")
        alive = child.poll() is None
        child.kill(); child.wait()
        time.sleep(1.0)
        shutil.rmtree(workspace, ignore_errors=True)
        results.append({"configuration": name, "browser_alive": alive,
                        "our_crashpad_handlers": len(ours),
                        "their_own_groups": sorted({r["pgid"] for r in ours.values()})})
        print(f'{name:<36} browser_alive={str(alive):<5} '
              f'our_crashpad_handlers={len(ours)}')
    return results


# ── the two-sided self-test ───────────────────────────────────────────────────

# ⛔ The worker's FIRST pipe in this fixture is its own stderr back to the
# caller, held by a NON-browser process and listed BEFORE the browser's. That
# ordering is not decoration: without it, a finder that ignores who holds the
# peer still returns the right answer by luck, and the control cannot
# discriminate the defect it exists to catch. Measured — the mutation survived
# a fixture whose first peer had no holder at all.
SIGHTED_LSOF = """\
COMMAND   PID       USER   FD   TYPE             DEVICE SIZE/OFF   NODE NAME
browse  94119 richarddje    2   PIPE 0x89f4bf21365d07fe    16384   ->0x31e00d16abd0751f
browse  94119 richarddje   11   PIPE 0xf470833860fd68af    16384   ->0xcdacbcc59fd84ba7
Python  94100 richarddje    6   PIPE 0x31e00d16abd0751f    16384   ->0x89f4bf21365d07fe
Google  94120 richarddje    2   PIPE 0xcdacbcc59fd84ba7    16384   ->0xf470833860fd68af
chrome_ 94123 richarddje    2   PIPE 0xcdacbcc59fd84ba7    16384   ->0xf470833860fd68af
Google  94130 richarddje    2   PIPE 0xcdacbcc59fd84ba7    16384   ->0xf470833860fd68af
"""

SIGHTED_PS = """\
  PID  PPID  PGID COMM
94100 94099 94100 /usr/bin/python3
94119 94100 94119 /repo/target/debug/reasonbraid-browse
94120 94119 94120 /repo/target/ci-browser/mac-arm64-0281kcb0/runtime/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing
94123     1 94122 /repo/target/ci-browser/mac-arm64-0281kcb0/runtime/chrome-mac-arm64/Google Chrome for Testing.app/Contents/Frameworks/Google Chrome for Testing Framework.framework/Versions/153.0.8010.36/Helpers/chrome_crashpad_handler
94130 94120 94120 /repo/target/ci-browser/mac-arm64-0281kcb0/runtime/chrome-mac-arm64/Google Chrome for Testing.app/Contents/Frameworks/Google Chrome for Testing Framework.framework/Versions/153.0.8010.36/Helpers/Google Chrome for Testing Helper (Renderer).app/Contents/MacOS/Google Chrome for Testing Helper (Renderer)
"""

BLIND_LSOF = """\
COMMAND   PID       USER   FD   TYPE             DEVICE SIZE/OFF   NODE NAME
browse  94119 richarddje    2   PIPE 0x89f4bf21365d07fe    16384   ->0x31e00d16abd0751f
"""


def self_test() -> int:
    failures = []

    pipes = parse_pipe_rows(SIGHTED_LSOF)
    processes = parse_process_rows(SIGHTED_PS)
    if len(pipes) != 6:
        failures.append(f"expected 6 pipe endpoints, parsed {len(pipes)}")
    if len(processes) != 5:
        failures.append(f"expected 5 processes, parsed {len(processes)}")

    # The subject is found by the peer's HOLDERS, not by the worker's fd number.
    pipe = stderr_pipe_of("94119", pipes, processes)
    if pipe != "0xcdacbcc59fd84ba7":
        failures.append(f"the browser stderr pipe resolved to {pipe!r}")

    holders = holders_of(pipe, pipes, processes)
    if len(holders) != 3:
        failures.append(f"expected 3 holders of the pipe, found {len(holders)}")

    verdict = classify(holders, "94120")
    if verdict["holders_in_owned_group"] != 2:
        failures.append(f"expected 2 holders inside the owned group, found "
                        f"{verdict['holders_in_owned_group']}")
    escapees = [h["pid"] for h in verdict["escapees"]]
    if escapees != ["94123"]:
        failures.append(f"expected the crashpad handler as the only escapee, found {escapees}")
    if verdict["escapees"] and verdict["escapees"][0]["ppid"] != "1":
        failures.append("the escapee's reparenting to init was not carried through")
    if verdict["escapees"] and verdict["escapees"][0]["binary"] != "chrome_crashpad_handler":
        failures.append(
            "the escapee's own binary name was not recovered: "
            f"{verdict['escapees'][0]['binary']!r}"
        )

    # ⛐ A macOS bundle executable's own name CONTAINS SPACES, and it is the
    # renderer — an in-group holder — that proves the recovery is whole rather
    # than sliced at the first blank.
    renderer = next((h for h in holders if h["pid"] == "94130"), None)
    if renderer is None or renderer["binary"] != "Google Chrome for Testing Helper (Renderer)":
        failures.append(
            "a bundle executable whose name contains spaces was not recovered whole: "
            f"{renderer and renderer['binary']!r}"
        )

    # ⛔ THE FLAG SURVEY'S OWN BINDING, driven here rather than against a browser:
    # a process present BEFORE the launch is never ours, however it is named, and a
    # process added by the launch is ours only if it is the handler we asked about.
    before = {"10": {"pgid": "10", "comm": "chrome_crashpad_handler"},
              "11": {"pgid": "11", "comm": "/usr/bin/other"}}
    after = dict(before)
    after["12"] = {"pgid": "12", "comm": "/r/Helpers/chrome_crashpad_handler"}
    after["13"] = {"pgid": "13", "comm": "/r/Google Chrome for Testing"}
    ours = ours_only(before, after, "chrome_crashpad_handler")
    if sorted(ours) != ["12"]:
        failures.append(
            "the flag survey counted a process it did not start, or missed one it did: "
            f"{sorted(ours)}"
        )

    # ⛔ THE RED SIDE, which is the whole point of the family this instrument
    # belongs to: given a listing in which the subject is NOT resolvable, the
    # finder must return nothing rather than guess — so a caller reports
    # `blind` instead of publishing an absence (`SIGNOFF-REPAIR.11.25.1.1`).
    blind = stderr_pipe_of("94119", parse_pipe_rows(BLIND_LSOF), processes)
    if blind != "":
        failures.append(f"a listing with no browser holder resolved a pipe: {blind!r}")

    # A holder whose pgid the process table does not carry must not be silently
    # counted as inside the owned group.
    unknown = classify([{"pid": "999", "fd": "2", "ppid": "?", "pgid": "?", "command": "x"}],
                       "94120")
    if unknown["holders_in_owned_group"] != 0 or len(unknown["escapees"]) != 1:
        failures.append("a holder with an unknown group was not treated as an escapee")

    for failure in failures:
        print(f"measure_browser_stderr_holder: {failure}", file=sys.stderr)
    print(
        "measure_browser_stderr_holder --self-test: "
        f"{'FAILED' if failures else 'ok'} ({len(failures)} failure(s))"
    )
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run the two-sided self-test")
    parser.add_argument("--render-secs", type=int, default=30)
    parser.add_argument("--interval", type=float, default=0.5)
    parser.add_argument(
        "--out",
        default="target/browser-stderr-holder.json",
        help="repository-relative path for the evidence record",
    )
    parser.add_argument("--runs", type=int, default=1)
    parser.add_argument("--flag-survey", action="store_true",
                        help="ask whether any launch flag stops the escaping handler")
    arguments = parser.parse_args()

    if arguments.self_test:
        return self_test()

    root = repository_root()
    binary = os.environ.get("R3_BROWSER_BIN", "")
    if arguments.flag_survey:
        if not binary:
            raise SystemExit("set R3_BROWSER_BIN to the pinned browser")
        survey = flag_survey(binary, 5.0)
        out = os.path.join(root, arguments.out)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        with open(out, "w", encoding="utf-8") as handle:
            json.dump(survey, handle, indent=1)
        print(f"wrote {arguments.out}")
        return 0
    records = [
        measure(root, arguments.render_secs, arguments.interval) for _ in range(arguments.runs)
    ]
    out = os.path.join(root, arguments.out)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with open(out, "w", encoding="utf-8") as handle:
        json.dump(records, handle, indent=1)

    for record in records:
        print(
            json.dumps(
                {
                    key: record[key]
                    for key in (
                        "positive_control",
                        "render_budget_secs",
                        "cleanup_confirmed",
                        "drain_ms",
                        "drain_censored",
                    )
                }
            )
        )
        if not record["receipt_claim_holds"]:
            print(
                "  \u26d4 cleanup_confirmed: true, and "
                f"{len(record['escapees_outliving_the_worker'])} browser-spawned "
                "process(es) outlived the worker"
            )
        for escapee in record["escapees_seen"]:
            print(
                f"  escapee pid={escapee['pid']} ppid={escapee['ppid']} "
                f"pgid={escapee['pgid']} fd={escapee['fd']} {escapee['binary']}"
            )
    print(f"wrote {arguments.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

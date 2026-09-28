#!/usr/bin/env python3
"""scripts/showcase.py — one entry point to try ReasonBraid live (`SHOWCASE.1`).

The director asked (2026-09-28) for a CLI and a web page to use the product as it
is being built and give live feedback, because a book is static. This is that
entry point. It is a LOCAL tool: everything binds to 127.0.0.1, it adds no route
to the server, and it drives the product only through its existing API and CLI.

    make showcase                               # the same as `up`
    python3 -B scripts/showcase.py up           # live system + the page
    python3 -B scripts/showcase.py status       # the progress summary, no server
    python3 -B scripts/showcase.py --self-test

`up` builds the binaries, starts a disposable PostgreSQL cluster under
`target/showcase/`, starts `rb-server`, enrols you (`you`) and two agent roles,
and keeps two `rb-node` agents running with the scripted stand-in adapter, so a
question you ask gets answered. Then it serves the page on
http://127.0.0.1:4320/ and prints the console URL and a CLI sheet. Ctrl-C stops
everything and removes the cluster.

⚠️ The agents are the deterministic fake adapter, as in the two-host demo:
`rb-node` constructs no real model adapter yet. Everything around them is the
real system: identity, authorization, ordering, budgets and the audit trail.

The progress panel is DERIVED, never typed: open leaves from
`scripts/census_open_leaves.py`, recent repairs from git, the last broad run from
its own log. Feedback typed on the page is appended to
`target/showcase/feedback.jsonl`, which the working session reads and routes to
the task tree.
"""

from __future__ import annotations

import json
import os
import re
import signal
import shutil
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHOWCASE = ROOT / "target" / "showcase"
# The director's notes land here. A test run points `SHOWCASE_FEEDBACK` elsewhere,
# so a check of the feedback box never writes into the director's file.
FEEDBACK = Path(os.environ.get("SHOWCASE_FEEDBACK") or SHOWCASE / "feedback.jsonl")
SERVER_PORT = int(os.environ.get("SHOWCASE_SERVER_PORT", "4310"))
PAGE_PORT = int(os.environ.get("SHOWCASE_PAGE_PORT", "4320"))
PG_PORT = int(os.environ.get("SHOWCASE_PG_PORT", "55442"))
AGENTS = ("agent-a", "agent-b")
# The claim each agent's one-time token is bound to, and the one its node presents.
HOST_CLAIM = "showcase-host"
ANSWERS = {
    "agent-a": "AGENT-A (scripted stand-in): I read the question and answer it independently; a real model would answer here.",
    "agent-b": "AGENT-B (scripted stand-in): a second, independent answer, so the panel has two views to compare.",
}


# ── the derived progress summary (pure where it can be) ─────────────────────


def broad_run_summary(log_text: str) -> dict:
    """Totals from a broad-run log: binaries, passed, failed, and the final rc."""
    results = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed", log_text)
    rc = re.findall(r"broad rc=(\d+)", log_text)
    return {
        "binaries": len(results),
        "passed": sum(int(p) for p, _ in results),
        "failed": sum(int(f) for _, f in results),
        "rc": int(rc[-1]) if rc else None,
        "demonstration": "ALL acceptance checks passed" in log_text,
    }


def census_summary(census: dict) -> dict:
    """Open leaves by kind and by lowest blocking class, from the census JSON."""
    by_kind = {kind: len(ids) for kind, ids in census.get("by_kind", {}).items()}
    by_class = {k: ids for k, ids in census.get("blocking_by_lowest_class", {}).items()}
    return {"open": census.get("open", 0), "leaves": census.get("leaves", 0),
            "by_kind": by_kind, "blocking_by_class": by_class}


def latest_broad_run() -> tuple[str | None, dict | None]:
    logs = sorted(ROOT.glob("target/*/broad.log"), key=lambda p: p.stat().st_mtime)
    if not logs:
        return None, None
    text = logs[-1].read_text(errors="replace")
    return str(logs[-1].relative_to(ROOT)), broad_run_summary(text)


def recent_repairs(limit: int = 12) -> list[dict]:
    out = subprocess.run(["git", "log", f"-{limit}", "--format=%h%x09%cs%x09%s"],
                         cwd=ROOT, capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        sha, date, subject = (line.split("\t", 2) + ["", ""])[:3]
        leaf = re.search(r"\(leaf ([A-Z-]+[0-9.]+)\)", subject)
        rows.append({"sha": sha, "date": date, "subject": subject,
                     "leaf": leaf.group(1) if leaf else None})
    return rows


def evidence() -> list[dict]:
    """Before/after logs a leaf left under target/r11_*: the RED and GREEN runs."""
    rows = []
    for d in sorted(ROOT.glob("target/r11_*"), key=lambda p: p.stat().st_mtime, reverse=True)[:12]:
        before = sorted(p.name for p in d.glob("red*.log"))
        after = sorted(p.name for p in d.glob("green*.log"))
        if before or after:
            rows.append({"leaf": ".11." + d.name[4:].replace("_", "."), "dir": d.name,
                         "before": before, "after": after})
    return rows


def progress() -> dict:
    census = subprocess.run([sys.executable, "-B", "scripts/census_open_leaves.py", "--json"],
                            cwd=ROOT, capture_output=True, text=True)
    summary = census_summary(json.loads(census.stdout)) if census.returncode == 0 else {}
    log_path, broad = latest_broad_run()
    return {"census": summary, "repairs": recent_repairs(), "broad_run": broad,
            "broad_run_log": log_path, "evidence": evidence()}


def print_status() -> int:
    data = progress()
    c = data["census"]
    print(f"open leaves: {c.get('open')} of {c.get('leaves')}  by kind: {c.get('by_kind')}")
    for klass, ids in sorted(c.get("blocking_by_class", {}).items()):
        print(f"  blocking, class {klass}: {len(ids)} — {', '.join(ids)}")
    b = data["broad_run"]
    if b:
        print(f"last broad run ({data['broad_run_log']}): rc={b['rc']}, {b['passed']} passed, "
              f"{b['failed']} failed across {b['binaries']} binaries, demonstration "
              f"{'passed' if b['demonstration'] else 'not seen'}")
    print("recent repairs:")
    for r in data["repairs"]:
        print(f"  {r['date']} {r['sha']} {r['subject'][:110]}")
    return 0


# ── the live system ─────────────────────────────────────────────────────────


class Live:
    """The disposable cluster, the server, the two agents, and the CLI identity."""

    def __init__(self) -> None:
        stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        self.work = SHOWCASE / f"run-{stamp}"
        self.cli_state = self.work / "cli"
        self.pg_bin = Path(os.environ.get("PG_BIN") or self._brew_pg())
        self.server_base = f"http://127.0.0.1:{SERVER_PORT}"
        self.children: list[subprocess.Popen] = []
        self.agents: dict[str, tuple[subprocess.Popen, Path]] = {}
        self.names: dict[str, str] = {}  # role id -> the name it was enrolled under
        self.tenant = ""

    @staticmethod
    def _brew_pg() -> str:
        prefix = subprocess.run(["brew", "--prefix", "postgresql@16"],
                                capture_output=True, text=True).stdout.strip()
        return f"{prefix}/bin"

    def rb(self, *args: str, as_json: bool = True) -> dict | str:
        cmd = [str(ROOT / "target/debug/rb"), "--server", self.server_base, *args]
        if as_json:
            cmd.append("--json")
        env = dict(os.environ, REASONBRAID_CLI_STATE=str(self.cli_state))
        out = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True)
        if out.returncode != 0:
            raise RuntimeError(f"rb {' '.join(args)}: {out.stderr.strip() or out.stdout.strip()}")
        return json.loads(out.stdout) if as_json else out.stdout

    def start(self, build: bool) -> None:
        binaries = [ROOT / "target/debug" / name for name in ("rb-server", "rb-node", "rb")]
        # ⛔ One cargo at a time: a second build waits on the first one's build
        # lock, and two stacked builds can stall each other. If a build is
        # already running (the working session's tests, most likely), this one
        # is skipped and the binaries already built are used, so starting the
        # showcase never disturbs work in progress.
        busy = subprocess.run(["pgrep", "-x", "cargo"], capture_output=True).returncode == 0
        if build and busy:
            if not all(b.exists() for b in binaries):
                raise RuntimeError("a cargo build is running and the binaries are not built yet; "
                                   "try again when it finishes")
            print("showcase: a cargo build is already running, so this one is skipped and the "
                  "binaries already built are used", flush=True)
        elif build:
            print("showcase: building the binaries (cargo, through project_env)…", flush=True)
            subprocess.run([sys.executable, "-B", "scripts/project_env.py", "cargo", "build",
                            "--workspace", "--bins", "--locked"], cwd=ROOT, check=True)
        missing = [b.name for b in binaries if not b.exists()]
        if missing:
            raise RuntimeError(f"missing binaries {missing}: run without --no-build first")
        self.work.mkdir(parents=True)
        self.cli_state.mkdir()
        data, sock = self.work / "pg", self.work / "sock"
        sock.mkdir()
        subprocess.run([str(self.pg_bin / "initdb"), "-D", str(data), "-A", "trust", "-U", "postgres"],
                       check=True, capture_output=True)
        subprocess.run([str(self.pg_bin / "pg_ctl"), "-D", str(data), "-l", str(self.work / "pg.log"),
                        "-o", f"-p {PG_PORT} -k {sock} -c listen_addresses=127.0.0.1", "-w", "start"],
                       check=True, capture_output=True)
        subprocess.run([str(self.pg_bin / "createdb"), "-h", "127.0.0.1", "-p", str(PG_PORT),
                        "-U", "postgres", "reasonbraid_showcase"], check=True)
        env = dict(os.environ, DATABASE_URL=f"postgres://postgres@127.0.0.1:{PG_PORT}/reasonbraid_showcase?sslmode=disable")
        server_log = open(self.work / "server.log", "w")
        self.children.append(subprocess.Popen(
            [str(ROOT / "target/debug/rb-server"), "--host", "127.0.0.1", "--port", str(SERVER_PORT)],
            cwd=ROOT, env=env, stdout=server_log, stderr=subprocess.STDOUT))
        ready = f"rb-server listening on {self.server_base} "
        for _ in range(300):
            if ready in (self.work / "server.log").read_text(errors="replace"):
                break
            if self.children[0].poll() is not None:
                raise RuntimeError("rb-server exited; see " + str(self.work / "server.log"))
            time.sleep(0.2)
        else:
            raise RuntimeError("rb-server did not come up; see " + str(self.work / "server.log"))
        me = self.rb("enroll", "human", "you")
        self.tenant = me["tenant_id"]
        for agent in AGENTS:
            role = self.rb("enroll", "role", agent, "--tenant", self.tenant)["principal_id"]
            self.names[role] = agent
            token = self.rb("node", "issue-token", "--node", role, "--host-claim", HOST_CLAIM,
                            "--as", "you", "--tenant", self.tenant)
            node_dir = self.work / agent
            node_dir.mkdir()
            script = json.dumps([{"step": "emit_chunk", "chunk": ANSWERS[agent]}, {"step": "complete"}])
            log = open(node_dir / "node.log", "w")
            self.children.append(subprocess.Popen(
                [str(ROOT / "target/debug/rb-node"), "--journal", str(node_dir / "node.db"),
                 "--server", self.server_base, "--node-id", role, "--fake-script", script,
                 "--poll-ms", "300", "--enroll-token", token["token_id"],
                 "--enroll-nonce", token["nonce"], "--node-secret", f"showcase-{agent}-secret",
                 "--host-claim", HOST_CLAIM,
                 "--provider", "fake", "--model", "scripted", "--harness", "fake"],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT))
            self.agents[agent] = (self.children[-1], node_dir / "node.log")
        # ⛔ An agent that cannot enrol exits at once, and the first version of
        # this script noticed nothing: the page asked, and no answer ever came
        # (a token bound to one host claim, the node presenting another). Each
        # agent must say it enrolled, or the start fails with its own log.
        for agent, (child, log_path) in self.agents.items():
            for _ in range(150):
                if "enrolled on host claim" in log_path.read_text(errors="replace"):
                    break
                if child.poll() is not None:
                    raise RuntimeError(f"{agent} exited: {self.agent_tail(agent)}")
                time.sleep(0.2)
            else:
                raise RuntimeError(f"{agent} did not enrol: {self.agent_tail(agent)}")

    def agent_tail(self, agent: str) -> str:
        lines = self.agents[agent][1].read_text(errors="replace").strip().splitlines()
        return lines[-1][:300] if lines else "(no output)"

    def agent_status(self) -> dict:
        return {agent: ("running" if child.poll() is None else f"stopped: {self.agent_tail(agent)}")
                for agent, (child, _) in self.agents.items()}

    def ask(self, question: str) -> dict:
        created = self.rb("thread", "create", "--subject", question[:200],
                          "--objective", "answer the question independently", "--as", "you")
        thread = created["thread_id"]
        for agent in AGENTS:
            self.rb("thread", "invite", "--thread", thread, "--agent", agent, "--as", "you", as_json=False)
            self.rb("thread", "accept", "--thread", thread, "--as", agent, as_json=False)
        return {"thread_id": thread}

    def thread(self, thread_id: str) -> dict:
        inspected = self.rb("inspect", "thread", thread_id, "--as", "you", "--tenant", self.tenant)
        events = inspected.get("events", {}).get("events", [])
        answers = [{"author": self.names.get(e.get("body", {}).get("author"), e.get("body", {}).get("author")),
                    "content": e.get("body", {}).get("content"), "at": e.get("committed_at")}
                   for e in events if e.get("event_type") == "thread.contribution_submitted"]
        state = inspected.get("thread", {}).get("state")
        if isinstance(state, dict):  # the projection nests the lifecycle name
            state = state.get("state")
        return {"thread_id": thread_id, "state": state,
                "events": len(events), "answers": answers}

    def threads(self) -> list:
        listed = self.rb("inspect", "threads", "--as", "you")
        return listed.get("threads", listed) if isinstance(listed, dict) else listed

    def cli_sheet(self) -> str:
        rb = f"target/debug/rb --server {self.server_base}"
        state = self.cli_state.relative_to(ROOT)
        return "\n".join([
            f"export REASONBRAID_CLI_STATE={state}",
            f"{rb} thread create --subject 'your question' --objective 'answer it' --as you",
            f"{rb} thread invite --thread <thread_id> --agent agent-a --as you",
            f"{rb} thread accept --thread <thread_id> --as agent-a",
            f"{rb} inspect thread <thread_id> --as you --tenant {self.tenant}",
            f"{rb} inspect threads --as you",
        ])

    def stop(self) -> None:
        for child in reversed(self.children):
            if child.poll() is None:
                child.terminate()
        for child in self.children:
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                child.kill()
        data = self.work / "pg"
        stopped = subprocess.run([str(self.pg_bin / "pg_ctl"), "-D", str(data), "-m", "fast", "stop"],
                                 capture_output=True).returncode == 0
        running = subprocess.run([str(self.pg_bin / "pg_ctl"), "-D", str(data), "status"],
                                 capture_output=True).returncode == 0
        if stopped or not running:
            shutil.rmtree(self.work)
            print(f"showcase: stopped, and {self.work.relative_to(ROOT)} removed")
        else:
            print(f"showcase: PostgreSQL did not stop; {self.work.relative_to(ROOT)} is left in place", file=sys.stderr)


# ── the page ────────────────────────────────────────────────────────────────


PAGE = r"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>ReasonBraid showcase</title><style>
:root{--bg:#fbfaf7;--fg:#1d1d1b;--muted:#6b6a64;--card:#fff;--line:#e4e1d8;--accent:#2f5d50;--warn:#9a5b13}
@media (prefers-color-scheme:dark){:root{--bg:#161614;--fg:#ecebe6;--muted:#a3a19a;--card:#1f1f1c;--line:#34332e;--accent:#8cc7b3;--warn:#e0a458}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.5 -apple-system,system-ui,sans-serif}
main{max-width:1080px;margin:0 auto;padding:24px 16px}h1{font-size:22px;margin:0 0 4px}h2{font-size:16px;margin:0 0 10px}
.muted{color:var(--muted)}.grid{display:grid;grid-template-columns:1fr 1fr;gap:16px}@media(max-width:820px){.grid{grid-template-columns:1fr}}
.card{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:16px;margin-bottom:16px}
textarea{width:100%;min-height:70px;background:transparent;color:var(--fg);border:1px solid var(--line);border-radius:8px;padding:8px;font:inherit}
button{background:var(--accent);color:var(--bg);border:0;border-radius:8px;padding:8px 14px;font:inherit;cursor:pointer;margin-top:8px}
.answer{border-left:3px solid var(--accent);padding:6px 10px;margin:8px 0}.small{font-size:13px}code,pre{font:12px/1.45 ui-monospace,Menlo,monospace}
pre{white-space:pre-wrap;background:transparent;border:1px solid var(--line);border-radius:8px;padding:8px;overflow:auto}
table{width:100%;border-collapse:collapse}td{padding:3px 6px;border-top:1px solid var(--line);vertical-align:top}.warn{color:var(--warn)}a{color:var(--accent)}
</style></head><body><main>
<h1>ReasonBraid showcase</h1>
<p class="muted small">Local only. The agents are scripted stand-ins; identity, authorization, ordering, budgets and the audit trail are the real system. <span id="live"></span></p>
<div class="grid"><div>
<div class="card"><h2>Ask the network</h2>
<textarea id="q" placeholder="Ask a question. Two agents are invited and answer independently."></textarea>
<button onclick="ask()">Ask</button><div id="asked"></div></div>
<div class="card"><h2>Your threads</h2><div id="threads" class="small muted">none yet</div></div>
<div class="card"><h2>Tell me what you think</h2>
<textarea id="fb" placeholder="What worked, what confused you, what you expected instead."></textarea>
<button onclick="feedback()">Send feedback</button><div id="fbok" class="small muted"></div></div>
</div><div>
<div class="card"><h2>Progress</h2><div id="progress" class="small">loading…</div></div>
<div class="card"><h2>Command line</h2><pre id="sheet"></pre>
<p class="small muted">The read-only console for the same data: <a id="console" href="#">open</a>.</p></div>
</div></div>
<script>
const $=id=>document.getElementById(id);let watched=[];
function esc(s){return String(s??'').replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]))}
async function j(url,opt){const r=await fetch(url,opt);const b=await r.json();if(!r.ok)throw new Error(b.error||r.status);return b}
async function ask(){const q=$('q').value.trim();if(!q)return;$('asked').innerHTML='<p class="small muted">asking…</p>';
 try{const t=await j('/api/ask',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({question:q})});
 watched.unshift({id:t.thread_id,q});$('q').value='';render()}catch(e){$('asked').innerHTML='<p class="warn small">'+esc(e.message)+'</p>'}}
async function render(){let h='';for(const w of watched){try{const t=await j('/api/thread/'+w.id);
 h+='<div class="card"><b>'+esc(w.q)+'</b><div class="small muted">'+esc(t.thread_id)+' · '+esc(t.state)+' · '+t.events+' events</div>';
 h+=t.answers.length?t.answers.map(a=>'<div class="answer"><div class="small muted">'+esc(a.author)+(a.at?' · '+esc(a.at.slice(11,19))+' UTC':'')+'</div>'+esc(a.content)+'</div>').join(''):'<p class="small muted">waiting for the agents…</p>';h+='</div>'}catch(e){h+='<p class="warn small">'+esc(e.message)+'</p>'}}
 $('asked').innerHTML=h;$('threads').innerHTML=watched.length?watched.map(w=>'<div>'+esc(w.id)+'</div>').join(''):'none yet'}
async function feedback(){const t=$('fb').value.trim();if(!t)return;await j('/api/feedback',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({text:t,watched:watched.map(w=>w.id)})});$('fb').value='';$('fbok').textContent='saved to target/showcase/feedback.jsonl — the working session reads it'}
async function progress(){const p=await j('/api/progress');const c=p.census;let h='<p><b>'+c.open+'</b> open of '+c.leaves+' leaves</p><table>';
 for(const[k,ids]of Object.entries(c.blocking_by_class||{}))h+='<tr><td>blocking, class '+k+'</td><td>'+ids.length+'</td></tr>';
 for(const[k,n]of Object.entries(c.by_kind||{}))if(k!=='blocking')h+='<tr><td>'+esc(k)+'</td><td>'+n+'</td></tr>';h+='</table>';
 const b=p.broad_run;if(b)h+='<p>Last full test run: <b>'+b.passed+' passed, '+b.failed+' failed</b> (rc '+b.rc+')'+(b.demonstration?', demonstration passed':'')+'</p>';
 h+='<p><b>Recent repairs</b></p>'+p.repairs.map(r=>'<div class="small">'+esc(r.date)+' <code>'+esc(r.sha)+'</code> '+esc(r.subject.replace(/^REASONBRAID-[A-Z]+-\d+ /,''))+'</div>').join('');
 if(p.evidence.length)h+='<p><b>Before and after</b></p>'+p.evidence.map(e=>'<div class="small">'+esc(e.leaf)+': '+e.before.map(f=>'<a href="/evidence/'+e.dir+'/'+f+'">before</a>').join(' ')+' '+e.after.map(f=>'<a href="/evidence/'+e.dir+'/'+f+'">after</a>').join(' ')+'</div>').join('');
 $('progress').innerHTML=h}
async function meta(){const m=await j('/api/meta');$('sheet').textContent=m.cli;$('console').href=m.console;$('live').innerHTML='Live: server '+esc(m.console)+'; agents '+Object.entries(m.agents).map(([a,s])=>esc(a)+' <span class="'+(s==='running'?'':'warn')+'">'+esc(s)+'</span>').join(', ')+'.'}
meta();progress();setInterval(render,2000);setInterval(progress,30000);setInterval(meta,10000);
</script></main></body></html>"""


def serve(live: Live) -> None:
    class Handler(BaseHTTPRequestHandler):
        def _send(self, code: int, body: bytes, kind: str) -> None:
            self.send_response(code)
            self.send_header("content-type", kind)
            self.send_header("cache-control", "no-store")
            self.end_headers()
            self.wfile.write(body)

        def _json(self, code: int, value) -> None:
            self._send(code, json.dumps(value).encode(), "application/json")

        def log_message(self, *_):  # quiet: the terminal is for the operator
            return

        def do_GET(self):
            try:
                if self.path == "/":
                    return self._send(200, PAGE.encode(), "text/html; charset=utf-8")
                if self.path == "/api/meta":
                    return self._json(200, {"console": live.server_base + "/", "agents": live.agent_status(),
                                            "cli": live.cli_sheet(), "tenant": live.tenant})
                if self.path == "/api/progress":
                    return self._json(200, progress())
                if self.path.startswith("/api/thread/"):
                    thread = self.path.rsplit("/", 1)[1]
                    if not re.fullmatch(r"thr_[0-9a-f-]{36}", thread):
                        return self._json(400, {"error": "not a thread id"})
                    return self._json(200, live.thread(thread))
                if self.path.startswith("/evidence/"):
                    m = re.fullmatch(r"/evidence/(r11_[0-9_]+)/((?:red|green)[A-Za-z0-9_]*\.log)", self.path)
                    if not m:
                        return self._json(404, {"error": "no such evidence"})
                    path = ROOT / "target" / m.group(1) / m.group(2)
                    if not path.is_file():
                        return self._json(404, {"error": "no such evidence"})
                    return self._send(200, path.read_bytes(), "text/plain; charset=utf-8")
                return self._json(404, {"error": "not found"})
            except Exception as error:  # the page shows the reason
                return self._json(500, {"error": str(error)})

        def do_POST(self):
            try:
                length = int(self.headers.get("content-length", "0"))
                if length > 20_000:
                    return self._json(413, {"error": "too long"})
                body = json.loads(self.rfile.read(length) or b"{}")
                if self.path == "/api/ask":
                    question = str(body.get("question", "")).strip()
                    if not question:
                        return self._json(400, {"error": "ask a question"})
                    return self._json(200, live.ask(question))
                if self.path == "/api/feedback":
                    text = str(body.get("text", "")).strip()
                    if not text:
                        return self._json(400, {"error": "empty feedback"})
                    FEEDBACK.parent.mkdir(parents=True, exist_ok=True)
                    with FEEDBACK.open("a") as out:
                        out.write(json.dumps({"at": datetime.now(timezone.utc).isoformat(),
                                              "text": text, "threads": body.get("watched", [])}) + "\n")
                    return self._json(200, {"saved": True})
                return self._json(404, {"error": "not found"})
            except Exception as error:
                return self._json(500, {"error": str(error)})

    httpd = ThreadingHTTPServer(("127.0.0.1", PAGE_PORT), Handler)
    print("\n== ReasonBraid showcase ==")
    print(f"  page:     http://127.0.0.1:{PAGE_PORT}/   (ask, watch the answers, progress, feedback)")
    print(f"  console:  {live.server_base}/   (read-only views of the same data)")
    print(f"  CLI:\n    " + live.cli_sheet().replace("\n", "\n    "))
    print("  Ctrl-C stops the agents, the server and the cluster.\n", flush=True)
    httpd.serve_forever()


# ── self-test (pure, no processes) ──────────────────────────────────────────


def self_test() -> int:
    log = ("test result: ok. 3 passed; 0 failed; 0 ignored\n"
           "test result: FAILED. 5 passed; 1 failed; 0 ignored\n"
           "[12:00] ALL acceptance checks passed\nbroad rc=0\n")
    assert broad_run_summary(log) == {"binaries": 2, "passed": 8, "failed": 1, "rc": 0,
                                      "demonstration": True}, broad_run_summary(log)
    assert broad_run_summary("nothing")["rc"] is None
    census = {"open": 5, "leaves": 9, "by_kind": {"blocking": ["A.1", "A.2"], "deferred": ["A.3"]},
              "blocking_by_lowest_class": {"3": ["A.1", "A.2"]}}
    assert census_summary(census) == {"open": 5, "leaves": 9, "by_kind": {"blocking": 2, "deferred": 1},
                                      "blocking_by_class": {"3": ["A.1", "A.2"]}}
    for path, ok in [("/evidence/r11_52/red.log", True), ("/evidence/r11_52/green2.log", True),
                     ("/evidence/../MEMORY.md", False), ("/evidence/r11_52/../../x.log", False),
                     ("/evidence/r11_52/mutant_R1.log", False)]:
        m = re.fullmatch(r"/evidence/(r11_[0-9_]+)/((?:red|green)[A-Za-z0-9_]*\.log)", path)
        assert bool(m) == ok, path
    print("showcase: self-test OK (broad-run totals, census summary, evidence paths confined)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    if argv[:1] == ["status"]:
        return print_status()
    live = Live()
    # Ctrl-C, a closed terminal and a plain `kill` all tear the system down.
    # ⚠️ SIGINT is installed explicitly: a process started in the background by a
    # non-interactive shell inherits SIGINT as IGNORED, and Python then raises no
    # KeyboardInterrupt, so the first check of this script sent Ctrl-C and
    # nothing stopped.
    signal.signal(signal.SIGINT, signal.default_int_handler)
    for sig in (signal.SIGTERM, signal.SIGHUP):
        signal.signal(sig, lambda *_: (_ for _ in ()).throw(KeyboardInterrupt()))
    try:
        live.start(build="--no-build" not in argv)
        serve(live)
    except KeyboardInterrupt:
        pass
    finally:
        live.stop()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

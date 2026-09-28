# SHOWCASE: one local entry point to use ReasonBraid live and send feedback

## Metadata

- Tree ID: `SHOWCASE`
- Status: `active`
- Roadmap lane: none — a director-requested tool over the SHIPPED system, off the frozen v0.4.1 roadmap. It adds no route, no binary and no product surface; it drives the product through its existing API and CLI.
- Created: `2026-09-28`
- Owner: repo-local workflow

## Goal

The director, 2026-09-28: *"Please create a single entry CLI and Web page for me to get a peek into what's going on, so that I can test live in addition to the mdbook. A book is static. a CLI and Web page would help me feel the progress much stronger."* And: *"That way, I can interact with final product as it is being build and provide you with some live user feedback."*

## Non-Goals

- A production web client for humans. That is `PARTICIPATION`'s requirement, graded there; the product's own console stays read-only by design (`web-ui.md`).
- A real model behind the agents. `rb-node` constructs only the fake adapter (the bug bar's claimed profile); wiring a real model is a separate decision.
- A new server surface. Everything the page does is an existing route or CLI verb.

## Leaves

### SHOWCASE.1 — `make showcase`: a live system with answering agents, a local page, a status CLI, and a feedback file

- Status: `done` — `REASONBRAID-SHOWCASE-0001`; opened 2026-09-28 by the director's request above.
- ⚖️ Bar (`REASONBRAID-DOC-0162`): not a corrective finding; the director's request, done before the remaining class 3 leaves at the director's priority.
- Owns: `scripts/showcase.py` (`up`, `status`, `--self-test`) and `make showcase`. `up` builds the binaries, starts a disposable cluster under `target/showcase/`, starts `rb-server`, enrols `you` and two agent roles, keeps two `rb-node` agents running with the scripted stand-in adapter, and serves a loopback page (default `:4320`) with an ask box (thread create, invite, accept through the CLI), live answers, a derived progress panel (open leaves from `census_open_leaves.py`, recent repairs from git, the last broad run from its log, before/after logs), the CLI sheet, and a feedback box that appends to `target/showcase/feedback.jsonl`. Ctrl-C stops everything and removes the cluster. Book: a chapter, linked from the summary.
- Acceptance: a real run end to end — the page answers, a question asked on it gets both agents' answers, the progress panel renders, feedback lands in the file, and Ctrl-C leaves no process and no cluster behind (`scripts/check_no_background_jobs.sh`).
- [x] **REPRODUCE / ISSUE** — the need, measured: the only live entry points were `make dev`, which starts an EMPTY system and a read-only console, and `make demo`, which runs every suite first (about an hour) and then a fixed script. Neither lets the director ask something and see it answered. The first live run of this tool then found its own defect: `curl http://127.0.0.1:4320/api/thread/<id>` showed `answers: []` after a minute, because each agent's token was bound to host claim `showcase-host` while `rb-node` presents `dev-host` by default; the agents exited at enrollment (*"the token is bound to a different host claim"*) and the page noticed nothing.
- [x] **ADDRESSED** — `scripts/showcase.py` (`up`, `status`, `--self-test`) and `make showcase`, as the Owns line says. The nodes now present the token's own claim, and the start waits until each agent logs that it enrolled, failing with the agent's log otherwise; the page shows each agent's state. Measured end to end (`target/showcase_check/`): the page `200`, `/api/meta` → both agents `running`, `/api/progress` → 64 open leaves and the last broad run's `578 passed`, a question asked on the page → **both agents' answers within about 2 s**, shown as `agent-a` and `agent-b` with their commit times; `/api/feedback` → `{"saved": true}` into the test file set by `SHOWCASE_FEEDBACK`, with the director's file untouched; the console `200`; an evidence log `200` and a traversal path `404`.
- ⚠️ **Two stop checks failed before one passed, and both were the check's fault or the environment's, then the tool's.** The first SIGINT went to the shell wrapper, not the Python process, which kept running. The second went to a process started in the background by a non-interactive shell, which inherits SIGINT as IGNORED, so Python raised nothing; SIGTERM, which the script handles, stopped it cleanly. The script now installs its own SIGINT handler, and the third run stopped on SIGINT: `showcase: stopped, and target/showcase/run-… removed`, no `rb-server`, `rb-node` or showcase `postgres` left (`pgrep -x`), ports 4310, 4320 and 55442 released.
- [x] **NO REGRESSION** — no product code changed; the tool drives existing routes and CLI verbs. `python3 -B scripts/showcase.py --self-test` → `self-test OK`; `bash scripts/check_doctrines.sh` on the staged commit, with `SELF-TEST` running the new self-test.
- [x] **LOCKSTEP** — the book gains *Try it live: the showcase* (`docs/book/src/showcase.md`), linked after the web console; `make help` lists the target. `ROADMAP.md` unchanged: this is a tool over the shipped system, not a planned feature.

### SHOWCASE.2 — Route the director's showcase feedback into the task trees

- Status: `pending` — opened 2026-09-28 with `SHOWCASE.1`, standing.
- ⚖️ Bar (`REASONBRAID-DOC-0162`): each note is classified by the bar when it is routed; this leaf is the routing, not a finding.
- Owns: at the start of each working turn, new lines in `target/showcase/feedback.jsonl` are read, and each becomes a leaf in the tree that owns its surface (or a note on an existing one), with the note quoted. A note is never answered only in chat.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1a | `SHOWCASE.1` | `done` | ✅ `make showcase`: a live system with two answering agents, a local page, a status CLI and a feedback file, run end to end. |
| 1 | `SHOWCASE.2` | `pending` | standing: the director's notes, routed as they arrive |

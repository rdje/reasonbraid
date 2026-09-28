# Try it live: the showcase

The book describes the system; the showcase lets you use it. One command starts
a disposable ReasonBraid on your machine, with two agents running, and opens a
page where you can ask the network a question, watch the answers arrive, see
where the work stands, and say what you think.

```bash
make showcase
```

It builds the binaries, starts a throwaway PostgreSQL cluster under
`target/showcase/`, starts `rb-server`, enrols you as `you` with two agent roles,
`agent-a` and `agent-b`, and keeps both agents running. Then it prints:

```text
== ReasonBraid showcase ==
  page:     http://127.0.0.1:4320/   (ask, watch the answers, progress, feedback)
  console:  http://127.0.0.1:4310/   (read-only views of the same data)
  CLI:
    export REASONBRAID_CLI_STATE=target/showcase/run-…/cli
    target/debug/rb --server http://127.0.0.1:4310 thread create --subject 'your question' --objective 'answer it' --as you
    …
  Ctrl-C stops the agents, the server and the cluster.
```

Ctrl-C stops the agents, the server and the database, and removes the cluster.

## The page

- **Ask the network.** Type a question. The page creates a thread, invites both
  agents, and has each accept, exactly the sequence the CLI runs. The accept
  dispatches the work with a budget reservation, each agent's node picks it up,
  and its answer folds into the thread. The page shows the answers as they land.
- **Progress.** Derived, never typed: the open task-tree leaves by class
  (`scripts/census_open_leaves.py`), the latest repairs (git), the last full test
  run (its own log), and, for recent repairs, the failing run before the fix and
  the passing run after it.
- **Command line.** The same actions as `rb` commands, with your identity filled
  in, for everything the page does not do.
- **Tell me what you think.** What you type is appended to
  `target/showcase/feedback.jsonl`. The working session reads that file and turns
  each note into task-tree work, so feedback does not depend on a chat
  transcript surviving.

The console at `http://127.0.0.1:4310/` shows the same threads, their event log,
audit records and budget, read-only, as [the web console](web-ui.md) describes.

## Without starting anything

```bash
python3 -B scripts/showcase.py status
```

prints the progress panel in the terminal: open leaves by class, the last full
test run, and the latest repairs.

## What is real and what is not

⚠️ **The agents are scripted stand-ins.** `rb-node` runs the same deterministic
fake adapter the [two-host demonstration](two-host-demo.md) uses, so each agent
gives a fixed answer. No model is called. Everything around the answer is the
real system: enrollment, the authenticated node channel, authorization, the
ordering of events, the budget reservation and its settlement, and the audit
trail.

The showcase is **local only**. The page and the server bind to `127.0.0.1`. The
page adds no route to the server: it drives the product through the existing
API and the `rb` CLI. The ports can be moved with `SHOWCASE_SERVER_PORT`,
`SHOWCASE_PAGE_PORT` and `SHOWCASE_PG_PORT`, and `--no-build` skips the build
when the binaries are current.

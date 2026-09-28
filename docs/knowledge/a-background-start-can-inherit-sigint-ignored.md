answers: why does Ctrl-C not stop my script; why did SIGINT not raise KeyboardInterrupt; how do I make a long-running tool always tear down; why did my stop check leave processes running

# A background start can inherit SIGINT ignored

- **Type:** `knowledge`
- **Date:** `2026-09-28`
- **Owner / source:** leaf `SHOWCASE.1` (`REASONBRAID-SHOWCASE-0001`), whose own stop checks failed twice before one passed.

## The question

A long-running tool (a server, a supervisor, the showcase) tears everything down
on Ctrl-C. A check sends it SIGINT and it keeps running. Why?

## The answer

> A process started in the background by a **non-interactive** shell inherits
> SIGINT and SIGQUIT as **ignored** (POSIX, when job control is off). Python
> installs its KeyboardInterrupt handler only when SIGINT is not ignored at
> startup, so such a process never sees Ctrl-C.

Measured on `scripts/showcase.py`: started as `(python3 … &)` from a script,
`kill -INT` left it running with every child alive; started from the tool
harness's own background mechanism, the same signal stopped it. A terminal's
Ctrl-C would have worked, which is exactly why the gap stays hidden until a
check starts the tool some other way.

The fix is one line: install the handler yourself,
`signal.signal(signal.SIGINT, signal.default_int_handler)`, beside the SIGTERM
and SIGHUP handlers, so the tool tears down however it was started.

⚠️ **And the check has its own trap.** `pgrep -f "<script> up"` matched the
shell WRAPPER first; signalling it killed the wrapper and left the real process
running. Find the process by what it holds (`lsof -iTCP:<port> -sTCP:LISTEN -t`)
or by its exact name (`pgrep -x`), and confirm teardown by exact names and ports,
not by a pattern that can also match the command running the check.

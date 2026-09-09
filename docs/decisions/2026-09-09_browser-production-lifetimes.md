---
answers:
  - Who owns the browser process when launch or rendering is cancelled?
  - Where does each browser invocation keep its profile and diagnostics?
  - What does successful worker cleanup establish and what remains unqualified?
---
# Own browser lifetime outside the render future

- Owner: `SIGNOFF-REPAIR.11.4.3.1.5.2`; REPAIR-0039.
- Evidence: docs/tasks/artifacts/signoff_review/browser-production-lifetimes.md.

Launch Chrome as an explicitly owned Tokio child, in a group established at spawn,
before awaiting endpoint discovery, connection or rendering. Chromiumoxide remains
the CDP client. Its pinned default launch flags and handler settings are preserved;
the worker adds explicit loopback debugging and private storage. Parse only the
owned child's bounded loopback browser endpoint, drain stderr throughout execution
and retain at most 64 KiB. Keep the CDP, network and stderr task handles.

Rendering, including startup, has the requested time budget (minimum one second);
startup also retains a twenty-second ceiling. Every cooperative outcome then enters
one additional ten-second process/task shutdown budget. Try CDP close, consume
child exit and inspect group absence, with bounded TERM/KILL escalation as needed.
Consume task completion/cancellation. Neither close acknowledgement, Drop, a
requested signal nor EPERM establishes cleanup. Retire a confirmed process-group
identity before later storage work; do not signal that old number on a storage error.

Each invocation exclusively creates a private UUID directory beneath
`.project-data/browser`, with separate profile/cache/tmp/config/data/state children.
Discover the current repository at runtime from current-directory ancestors, as
rb-site does. No checkout-specific root is compiled into production storage and no
OS/home temporary fallback is allowed. Enabling R3 currently requires launching
from within the repository; other packaged server features keep their existing
independence. Installed Chrome is a required read-only tool dependency.

Record ownership before the first await; record completion on stderr and retain
private owner/completion/bounded-stderr files on failure. Delete successful storage
only after process/task confirmation, original directory identity and same-volume
census. Chromium's singleton links are unlinked without traversal. Workspace Drop
retains data. This assumes an operator-controlled repository and same-user storage;
it is not an adversarial same-UID mutation or mounted-filesystem isolation proof.

The ten-second bound covers asynchronous process/task shutdown. Stdin admission,
filesystem system calls and response delivery do not acquire a hard real-time bound
from it. An OS kill, worker crash or intentionally detached descendant still needs
parent/container enforcement, owned by `.7.3.1` and `.7.3.2`. The current server kills
at the render deadline and can interrupt cleanup; worker controls cannot close that
integration gap. Aggregate retained-storage quotas and broader browser resource,
network, sandbox and evidence policies also remain explicitly tracked.

## Ambient output paths

The launcher also binds CHROME_LOG_FILE, BREAKPAD_DUMP_LOCATION and
CHROME_USER_DATA_DIR to its private invocation paths, and removes SSLKEYLOGFILE
and QLOGDIR. The configured executable control receives deliberately conflicting
values and requires all five corrections before emitting its bounded diagnostic
payload. This qualifies the launch environment, not a deliberate real-browser crash.
Chromium documents the logging override in its
[logging guide](https://www.chromium.org/for-testers/enable-logging/); the
[crash reporter implementation](https://chromium.googlesource.com/chromium/src/+/HEAD/chrome/app/chrome_crash_reporter_client.cc)
uses the alternate dump-directory environment variable. Published upstream source
supports the mechanism; native execution and the local pinned client have separate
evidence. Chrome-created logs/crash data are private but not given an aggregate
quota by the 64 KiB stderr bound; that remains .7.3.2.

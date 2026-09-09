# Production browser lifetime evidence

Owner: `SIGNOFF-REPAIR.11.4.3.1.5.2`; REPAIR-0039. Baseline commit `9829b44`.
Raw evidence: `target/browser-production-controls`. The installed Chrome 152.0.7977.83
is a read-only OS-volume tool; all probe, worker and Cargo stores are repository-local.

## Baseline and root cause

The supervised baseline uses an exclusive fixture and local HTTP origin. Worker
74852 exits zero after rendering. The immediate census identifies Chrome renderer
75395, state R, parent PID 1, still in owned group 30880 and naming the shared
`tmp/chromiumoxide-runner` profile. The enclosing supervisor then confirms whole-group
cleanup, rc=0; its origin is closed and its thread joined. This is a concrete
post-worker renderer observation, not a claim that it would live indefinitely.

The exact worker SHA-256 is
`7d76ce0be431d266a9edd7809b4da6efdd89e6f9d63235cec1957c3439b055ff`;
main.rs SHA-256 is
`8aae4effcdf1f0a955ca1fd05e75009ca909983c88c5c00c326ed7f1265ae221`.
See baseline.json and baseline-d1y3whgw request/worker/log/census files. This binary
was rebuilt by the earlier test workflow; it is not the older inventory binary.

Pinned chromiumoxide 0.9.1 creates `temp_dir()/chromiumoxide-runner` when no profile
is supplied. Its async launch owns a child internally, so cancelling launch has no
explicit consumed shutdown path in the caller. Production main previously closed
CDP only on success, never waited for child exit, aborted its handler without joining
and discarded the network task handle. Those missing ownership barriers explain why
the old worker's return was not process completion.

## Replacement and falsification

The new lifetime/storage modules keep the child and all three task handles outside
the cancellable operation, use a unique private repository-derived workspace and
run explicit bounded shutdown on every cooperative outcome. Endpoint discovery
requires the owned child's loopback browser WebSocket URL; stderr is drained with
64 KiB retained and a 4 KiB line parser. Successful storage removal follows process
and task confirmation, identity verification and a no-follow same-volume census.
Failed/unconfirmed workspaces retain private receipts and bounded diagnostics.

The first integration run passes ten controls in 7.03s, rc=0, including real render,
step error/output refusal and cancelled startup. Every production worker returns
without test-side worker-group stopping, and independent browser-group observations
establish absence. Existing intentional stalled/overflow harness controls still
exercise supervisor intervention. Its initial executable startup delay preceded the
runner's test output; no test-body timing claim is derived from that delay.

Final qualification uses frozen identities in qualified-identities.json. The built
worker SHA-256 is d57a6a2a2f91230708915f87e749c16af28619f82bfc252f8306b4045617a015.
Five unit controls pass in 0.03s; thirteen integration controls pass in 7.03s, zero
failed/ignored/skipped. Strict all-target/all-feature focused clippy passes in 6.97s.
All commands return zero and all process results are consumed. The injected handler
panic is an expected negative control: its join is consumed, cleanup refuses and
storage remains until the control verifies/removes its known process-free fixture.

The real controls prove rendering/network/title/digest compatibility, step error,
output refusal, navigation deadline after an observed origin hit, and two overlapping
browser groups with distinct live profiles beneath the same runtime root. Instrumented
executables prove cancelled startup, failed startup with 100,000 stderr bytes capped
to exactly 65,536, and replacement of ambient output paths. Independent storage
controls prove private same-volume creation, collision preservation, retained Drop,
replacement refusal and no traversal through singleton-style links.

Thirteen terminal worker receipts and eight browser completions confirm cleanup;
every production invocation returns without test-side worker-group stopping. Native
checks independently find all twenty-one groups absent, no matching live Chrome
profiles and no successful/unit fixture residue. Exactly nine older failed fixtures
remain: the original .5.1 case and eight zero-output workers from this leaf's failed
run. Source/binary identities match before/after; Cargo.lock, server browse.rs and
README are unchanged. Static launch-flag comparison preserves all thirty-one prior
non-profile flags and adds only explicit loopback binding; profile/cache arguments
are derived separately. Workspace format, diff, book and twelve rendered markers
pass. See qualified-verification.json, scope-verification.json and book-verification.json.
The first aggregate-receipt parser assertion caught one libtest status line interleaved
inside the budget receipt; verify_final.py records/removes only that exact display
fragment for parsing and retains the raw log. No result or receipt field was invented.

## Preserved startup failure

final-tests.log/.exit preserves an earlier expanded run: four unit controls passed,
then six integration controls passed and seven failed (eight production workers,
because overlap uses two). All eight workers exceeded their existing harness deadlines
with zero stdout/stderr, including budget admission which never starts Chrome. Their
groups were consumed; every failed fixture remains. The observer could not establish
live overlapping browsers. This is an actual failed run, not browser qualification.

A unit-only rebuild overlapped that in-flight run. Subsequent native inspection found
no remaining worker/executable to sample; the unit executable completed naturally.
The exact wait mechanism for these eight processes was not captured, and no failed-run
binary hash was captured before possible rebuild. Do not label it as a proved loader
or Chrome defect. Separate pre-browser diagnostic probes consume exit-zero responses
within their one-second observer windows, with unchanged before/after binary hashes
and no live process needing sampling. A no-rebuild thirteen-control retry passes;
after the final output-environment correction, the complete five/thirteen-control
qualification above passes on frozen final identities. The original failure and its
limitations are additionally owned by .11.2 alongside existing independently sampled
native-loader waits. No timeout, signature, attribute or OS security setting was relaxed.

## Limits and durable routing

The runtime root is discovered from current-directory ancestors containing Cargo.toml
and migrations; the R3-enabled deployment must run inside the repository. The existing
packaged server otherwise retains its independent runtime. Process/task shutdown has
one additional ten-second budget; filesystem calls and wire admission/delivery are
not a hard real-time guarantee. Failed storage is retained without an aggregate quota
yet. Ordinary OS/toolchain reads remain explicit locality exceptions.

Server browse.rs still kills at the render deadline, waits before draining stdout and
owns no descendant group. `.7.3.1` owns its complete transport/termination repair;
`.7.3.2` owns retained-data quotas and detached-process/container qualification.
Abrupt kills, deliberate process-group escape, adversarial same-UID filesystem
mutation, browser sandbox/network/evidence hardening and non-native runtime platforms
are not qualified by this worker leaf. Stale owner PIDs are evidence, never automatic
permission to signal a current process. Full CI, remote results and push remain pending.

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

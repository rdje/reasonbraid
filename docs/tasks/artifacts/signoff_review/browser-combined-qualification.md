# Combined browser qualification

Owner: `SIGNOFF-REPAIR.11.4.3.1.5.3`; REPAIR-0040. Production baseline `0cd0592`.
Raw evidence: `target/browser-combined-controls`. Chrome 152.0.7977.83 and installed
compiler/OS tools remain read-only dependencies; all fixtures, outputs and stores
are repository-derived on its volume.

## Preserved implementation and coverage census

Production main/lifetime/storage source and worker binary are byte-identical to the
qualified REPAIR-0039 state. Worker SHA-256:
`d57a6a2a2f91230708915f87e749c16af28619f82bfc252f8306b4045617a015`.
Only browser_roundtrip.rs changes among Rust files. Cargo manifests/lock, server
browse.rs, support helper, README and CI workflow remain unchanged.

The executable lists fifteen integration controls. The earlier five unit controls
retain their separate source-bound REPAIR-0039 results; they were not rerun here.
The native integration source has no ignored controls. Absent Chrome is still
explicitly unqualified; configured invalid binaries fail. CI requires an executable
Chrome and builds/checks both worker binaries before workspace tests. Source routing
is not actual Ubuntu/GitHub execution evidence.

| Boundary | Concrete evidence |
| --- | --- |
| Render/network/title/digest and admission | Preserved controls in the final fifteen-control executable |
| Step/output/startup/navigation errors and deadlines | Real or explicitly instrumented worker controls; all production outcomes independently inspected |
| Profile concurrency | Two real browsers held at the origin gate, distinct live groups/profiles, delayed second launch |
| Runtime-root relocation | Two real renders using the same binary across a same-volume rename; device/inode/witness preserved |
| Linked storage parent | Real worker returns browser_storage_failed, emits no browser ownership and leaves linked target unchanged |
| Worker/origin/stream lifetime | Existing bounded-output/group/descendant controls plus exact listener close receipt |
| Private storage and handler failure | Five unit controls from REPAIR-0039 against unchanged production source |
| Parent kill/pipe protocol and detached processes | Still .7.3.1/.2; not closed by this matrix |

## Relocation and linked-parent controls

The initial selected runs pass one control each: relocation 1.94s, linked parent
0.46s; strict focused lint passes in 13.91s, all rc=0. Relocation moves the exact
runtime-root directory between two real worker invocations. The directory remains
on device 16777244, inode 75777380 in that initial run; the old path disappears,
a witness remains unchanged and both successful private invocation directories are
removed. Each worker/browser group is independently consumed. The linked-parent
control uses an owned local sentinel, not an external or shared target.

## Concurrency correction

Source review found the previous overlap control held the first browser using
fifteen 200 ms settle beats. An uneven second startup could miss that window even
with correct isolation. Replace the timing assumption with an explicit local-origin
gate. Both requests remain pending until the observer verifies two live groups and
two distinct profiles. The second command deliberately begins four seconds after
the first request arrives. Release the gate on observation success/failure and
origin shutdown so a failed assertion cannot strand the serving task.

## Listener-identity failure and native falsification

The first complete run after that correction passes fourteen controls and fails
origin_shutdown_closes_the_listener: a connection to the old address succeeds after
Origin.finish was consumed. Its exact address/peer was not retained. Preserve
gated-tests.log and exit 101; do not label its peer as a proved reused listener.

The separate reused_port.py control closes its first listener (fileno -1), binds
an owned successor to the same port 62215, receives the payload "owned successor"
and closes all four sockets. Numeric descriptor 3 is reused too. This proves the
old port-reachability assertion can fail despite correct original shutdown.
reused-port.json records the independent client/peer identities and closed state.

Pinned axum 0.8.9 serve/mod.rs drops the listener before awaiting all connection
receivers. The test wrapper now drops its exact socket before sending a dedicated
oneshot receipt; Origin.finish requires both the consumed task and receipt. The
live-listener control rejects a premature receipt. Accept/error behavior still
delegates to axum's own TcpListener implementation. No timeout or shutdown condition
is weakened.

## Final results and limits

The final integration executable passes fifteen controls in 8.55s, zero
failed/ignored/skipped, rc=0. Strict all-feature focused clippy passes in 10.57s,
rc=0. All build, list, selected/full-test, lint and native-control results are
consumed. An attempted startup inspection found no still-waiting empty-output
executable to sample; it adds no diagnosis to the earlier .11.2 startup evidence.

Sixteen terminal worker receipts and ten browser completions confirm cleanup; no
production invocation requires test-side worker-group stopping. The original
intentional stalled/overflow harness controls still require supervisor intervention.
Native verification independently finds all twenty-six final groups absent and
all successful fixtures removed. The exact same nine prior failed fixtures remain;
no new failed origin fixture exists because that control owns no directory. Final
production/control source and binary identities match their recorded values.
Workspace format, git diff, make book and nine rendered-marker checks pass, rc=0.
The parser consumes the original final log directly; no display normalization is
needed for this run. See verification.json, identities.json and census.json.

This closes .11.4.3.1.5 and its three children as a full-checkpoint prerequisite.
It does not close server hard termination/pipe handling, aggregate retained storage,
detached-process/container enforcement, broader browser isolation/evidence policy,
uncaptured host startup waits or Linux/remote qualification. Those owners stay open.
Compiler-artifact disposition .11.4.3.1.6 precedes full local/remote execution .2;
no push or fresh full CI is claimed.

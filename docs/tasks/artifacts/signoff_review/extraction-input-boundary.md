# Production extraction input boundary

Owner: `SIGNOFF-REPAIR.7.3.3.1`; REPAIR-0058. Source:
`be22d43fade68dfb087887aef4dddab575533d4f`. Raw evidence:
`target/extraction-input-controls`.

## Preservation and exact reproduction seam

Preserve api.rs, extraction.rs, the server manifest, Cargo.lock and the current
actual extraction worker before any compilation. Freeze all 342 tracked
non-Markdown source identities. The 479-byte API span at api.rs:1985–1993 uses
PID/subsec_nanos and std::fs::write. Git binds it to e2b43ea1, PHASE-4.4.3; the
fixture predecessor belongs to ede2e2a8, PHASE-4.4.2. No server source is modified.

The ignored standalone probe embeds that span byte-for-byte, giving it the same
document.bytes input through a minimal wrapper. Thirty-two threads begin behind a
barrier; join all writers before observing their files, preserving a controlled
possible overlap without a mocked clock. After finding a mismatch, call the
unchanged copied extraction.rs run_extraction with the preserved actual worker
for each affected caller. Keep every conflicting input rather than invoking the
API's subsequent deletion. This isolates name/write and actual worker-read
semantics; it does not execute the HTTP handler, fetch network data or mutate a
database. Do not describe it as a server HTTP end-to-end test.

The first isolated build fails with E0433 because the diagnostic manifest omits
chrono, used by the copied module's receipt type (71.742954 seconds, exit 101).
No baseline runtime starts. Preserve that manifest, lock and log; add exact
chrono 0.4.45 with serde, regenerate only the isolated lock and verify all
52 dependency identities/checksums match the workspace. Dependency feature-union
identity is not claimed. Corrected build passes in 10.332957 seconds.

## Native-clock interference and actual worker output

The runtime stops at round zero with the explicit reproduced-defect exit 10 in
0.942501 seconds. All 32 writers finish. Six duplicated paths produce eight
wrong-owner observations and leave 24 unique files; every file is retained with
size/hash identities. The unchanged worker returns the overwritten file's actual
content and parent digest for all eight affected callers.

| Owners sharing one path | Caller receiving another owner's bytes | Worker text owner |
| --- | --- | --- |
| 0, 5 | 0 | 5 |
| 2, 4 | 4 | 2 |
| 6, 7 | 7 | 6 |
| 8, 9, 10, 13 | 8, 9, 13 | 10 |
| 19, 21 | 19 | 21 |
| 23, 24 | 24 | 23 |

The independent Python reconciliation reconstructs all intended Atom inputs,
checks their SHA-256 identities, reads every retained file, and verifies each
reported parent/chunk digest and chunk text. The observed mismatch is between the
caller's intended bytes and the worker's actual input, not a fabricated hash or
changed parser. The actual worker group is consumed. Adjacent unchanged-worker
positive controls use retained unique-owner 1 and shared-path winner 5 without
rewriting inputs. Both pass in 0.339678 seconds, exit zero, with exact own-source
parent digest and expected chunk text/digest. The earlier positive-control build
passes in 4.670757 seconds; its dispatch then stops before runtime because the
planned receipt name already belongs to immutable input-case data. Preserve this
observer failure and use the distinct positive-run receipt; no input is overwritten
and no worker ran during the failed dispatch.

## Consequences and bounded repair ownership

After run_extraction returns, the API removes its path unconditionally. On success
it persists acquired document.bytes but submits response chunks without comparing
response.parent_digest to those acquired bytes. The controlled worker results
show how distinct inputs can reach this boundary; actual incorrect database
persistence remains untested and is owned by .7.3.3.3, not claimed as reproduced.

The worker function can also return from stdin/try_wait errors without consuming
its Child, and the timeout discards kill/wait errors. This is source-level evidence
that return/timeout is not a sufficient cleanup witness; error-path runtime
reproduction and correction belong to .7.3.3.2. Qualify that prerequisite first,
then integrate exclusive same-volume input ownership, digest binding and precise
cleanup/retention in .7.3.3.3. The indexed decision records the selected contract.

The existing profiles R2 test exercises ranking, loopback-fetch refusal, preserved
reference and stricter sandbox refusal. It never reaches successful input storage;
it cannot certify concurrent document attribution. Preserve its existing checks
and add the needed actual-boundary coverage during integration.

Broader synchronous stdin, wait-before-stdout-drain, unbounded reply read,
descendant pipe ownership and aggregate retained-storage limits have concrete
owner .7.3.4. Direct-child completion and unique paths must not close those larger
isolation boundaries. This diagnostic commit changes no production/test code or
manifest; full checkpoint and public push remain pending.


## Platform interruption and private support draft

On 2026-09-10 the director reports approximately three near-consecutive OpenAI
content-withheld notices, then two further notices, and requests the information
needed for Support. The recorded user-report times are 11:43:29.427 UTC and
11:53:05.542 UTC (13:43 and 13:53 CEST, Europe/Paris); these are not confirmed
moderation-event times. Local metadata identifies Codex CLI 0.153.4, model
gpt-6-astra, macOS arm64. The exact classifier trigger is unknown. Successful local
tool results remain valid; expected diagnostic exit 10 and the separate corrected
receipt-name assertion must not be relabeled as platform failures.

The mode-0600 draft at target/extraction-input-controls/openai-support-report.md
contains the user's exact notice, environment, current session identity, seven
nearby response IDs explicitly labeled as unconfirmed correlation candidates,
redacted authorized scope, observed sequence and missing account/request details.
Its support-report-receipt.json binds the local bytes. Account email/plan,
applicable organization/workspace/project identity, authentication route and any
Trusted Access approval remain for the director to add privately; no credential
contents were inspected. The report and identifiers stay ignored, outside public
Git history. This tracked record preserves the incident and report location.
No message was sent and no account/model/safety/access setting changed.

Official reporting guidance: https://help.openai.com/en/articles/20001326 and
https://help.openai.com/en/articles/6614161-how-can-i-contact-support. The notice
alone does not establish a policy violation. Exact blocked-request IDs and
withheld response contents are unavailable; the session ID is not a request ID.


## Final verification

Independent verify.py returns zero: all 342 tracked non-Markdown sources are
unchanged; five original identities and 24 retained input files match size/hash;
the exact 479-byte production span and 52 locked dependency identities match.
All eight recorded command groups are absent. The book build returns zero in
0.393835 seconds; eight rendered markers pass. README remains unchanged and
qualification categories are identical. Private report bytes, same-volume storage,
0600 mode and ignored status pass; public tracked diff excludes correlation IDs.
No production change, full CI, public push or Support submission occurs here.

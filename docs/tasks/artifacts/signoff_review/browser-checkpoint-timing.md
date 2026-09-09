# Browser timing witnesses at the full checkpoint

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.4`; REPAIR-0045. Date: 2026-09-10.
Source checkpoint: `7e0109735ebdac487b791997af058a171f85b0e4`.
Raw evidence: target/checkpoint-ci/full-7e01097 and target/browser-checkpoint-controls.

## Original result and boundary

The source-7e01097 checkpoint passes format, strict all-target/all-feature Clippy,
workspace binaries, fifty Python controls (including four native PostgreSQL runner
controls), book, thirteen doctrines, cargo-deny and the pinned history scan.
The history scan checks 314 commits and reports zero findings under the qualified
two-fingerprint policy. Workspace test compilation passes; execution stops at
browser_roundtrip with thirteen passed/two failed in 12.39s, command exit 101.
The live PostgreSQL collection and demonstration do not start. This is a failed
full checkpoint, not a broad runtime pass. The driver and native job census are
consumed, with no remaining project background job.

The original navigation worker returns time_budget_exceeded for its four-second
render and confirms browser cleanup. Its origin-hit assertion fails. The overlap
control times out awaiting two navigations, but only the first fixture has a worker
receipt; its render succeeds after the observer releases the gate. The second
fixture has no worker logs/receipt. The old assertion order discards the second
command's error before displaying it, so the original exact second-launch error
and host scheduling cause are not recoverable from that log. Preserve this limit.

The production render budget covers launch and navigation together. Startup itself
can take up to twenty seconds. A four-second budget therefore cannot guarantee a
navigation-stage witness. In the overlap test, an independent five-second wait can
return before starting worker two; a twelve-second observer then masks that error.
The twenty-second render budget also competes with two sequential startups and a
four-second launch delay. These are test qualification assumptions, not established
production shutdown failures.

## Native falsification

The original test executable is preserved and hash-verified before replacement:
SHA-256 99e63e209ac77578448138c1c00fbca08fbde605799b0609c7ed7755197e3222.
The checkpoint production worker remains unchanged:
SHA-256 82db3f0ae8871ace5d1f221ccdecabd212356bc8a11dcbe625da5868b5a40a53.
Original extraction-worker bytes also remain equal to the checkpoint receipt.

A task-owned executable delays Chrome startup by six seconds, recording separate
start/exec observations before exec of the installed browser. Both unchanged test
functions reproduce exit 101. Navigation expires before the wrapper reaches exec;
its process group is consumed. Overlap reaches real Chrome after the delay, but
worker two is never launched; the first render completes after the twelve-second
observer failure releases its response. Durations are 6.079s and 12.396s including
supervision. This establishes a controlled failing mechanism and matching witness
failure, without inventing the original host's missing phase timings.

## Repair contract

Only the integration-test source changes. An explicit gated endpoint records actual
navigation arrivals through a watch channel; responses stay pending until release.
The navigation-deadline control admits a thirty-second render (twenty-second
production startup allowance plus ten seconds for protocol/page setup), requires
an actual arrival and time_budget_exceeded, and keeps the response unreleased until
the worker returns. It verifies independent browser-group absence and the production
completion receipt. The separate one-second cancelled-launch control remains.

Overlap still deliberately delays worker two four seconds after worker one's
navigation. Two thirty-second navigation windows plus that delay bound observation
at sixty-four seconds; ten seconds for post-release completion yields a seventy-four
second render budget. Supervision adds the ten-second production cleanup allowance
and ten seconds for worker startup. Two distinct live groups/profiles, no completion
receipts before release, successful responses and consumed shutdown remain required.
Both command errors and the observer outcome are recorded before propagating a panic.
No skip or removed concurrency/deadline assertion substitutes for qualification.

A socket-level control independently proves one arrival cannot satisfy a two-request
witness, that no response byte is sent before release, and that explicit release
produces the expected HTTP response. The exact original listener close receipt and
consumed server task remain the shutdown predicate.

## Further production boundary exposed

The first corrected delayed-start navigation observes a request but returns
browser_cleanup_unconfirmed: stderr completion did not finish in the ten-second
cleanup budget. Its fixture is retained. A second run captures native process and
pipe identities and reproduces the same refusal in 40.046s. At ownership +31/34/37
seconds the browser group is gone, while detached chrome_crashpad_handler and four
GoogleUpdater processes still hold the exact stderr write endpoint paired with the
worker's read descriptor. Raw observations preserve both pipe directions, process
identity and timestamps. This establishes the escaped-writer cause of missing EOF.
It does not establish the full set of updater filesystem mutations or authorize
killing/deleting shared browser/update state. A dedicated non-updating runtime and
the existing detached/container owner require further qualification.

The candidate test repair remains unqualified, and the broader checkpoint remains
failed. Preserve both new production refusals; do not reinterpret them as successful
cleanup or replace the pending-navigation witness with a shorter workload.

## Selected dedicated-runtime qualification

Official metadata selects Chrome for Testing 153.0.8010.36 for mac-arm64.
The official HTTPS archive is 191,016,009 bytes, SHA-256
1f701ef60757c63c6ccf98afaf28291dd0c8d1457d3d738e81fd62201c230ad0;
its published GCS MD5 also matches. Extraction verifies paths, bounded sizes and
internal symlink chains. An initial direct-target check rejects a legitimate
framework link through Versions/Current before extraction; its result is preserved.
The corrected verifier rejects cycles/escapes and resolves to actual archive members.
All 346 regular extracted files (375,683,377 bytes) match archive payload hashes;
five links remain internal. The archive's total uncompressed size includes those
141 bytes of link text (375,683,518 bytes across 675 entries).

The additional Developer ID expectation fails: the archive contains no resource
seal, and native codesign display identifies an ad-hoc linker signature with no
team identifier. Preserve that failure. The exact upstream release's
chrome/installer/installers.gni disables the macOS installer for is_chrome_for_testing;
this is a testing distribution, not a verified signed desktop application. No
signature, quarantine or OS setting is changed. Trust/provenance here is the official
HTTPS metadata/archive, published integrity value, recorded SHA-256 and equal
extracted bytes. The native --version check passes exactly. Browser executable
SHA-256: bfe18f25f912e28e567164f823efbcbf0a87c5292bd66566b6cce7861086d789.

| Run | Executed controls | Result | Supervised elapsed |
| --- | --- | --- | --- |
| Six-second delayed real navigation | 1 | pass, rc=0 | 30.168s |
| Six-second delayed Chrome at each overlapping launch | 1 | pass, rc=0 | 18.227s |
| Complete integration executable, required dedicated browser | 16 | pass, rc=0; no ignore/skip | 30.189s |
| All-target/all-feature strict focused Clippy | selected browse crate | pass, rc=0 | 53.009s |

The two selected controls repeat members of the sixteen-test collection; these are
eighteen invocations, not eighteen distinct tests. The navigation receipt proves an
actual request, an unreleased response and production-confirmed cleanup. Overlap
records two distinct simultaneously live groups/profiles after the delayed launch.
The socket-level missing-second and unreleased-response controls pass in the full
collection. The original desktop-runtime refusal remains a real configuration limit;
the dedicated runtime also changes browser version, so no version-independent or
universal process-containment claim follows from this comparison.

Independent native verification finds nineteen worker and thirteen browser groups
plus three test groups absent. Successful fixtures are absent; all nine historical
and eight newly failed fixtures remain (seventeen total). Original checkpoint-failure
file hashes remain equal. Exactly one tracked non-Markdown file differs from the
checkpoint manifest: browser_roundtrip.rs. The production worker and extraction
binary retain their exact checkpoint hashes. All results are consumed.

Pinned installation and local/CI binding are the concrete next leaf .11.4.3.1.2.5;
current CI still selects installed desktop Chrome. The configured trusted-fixture
result does not qualify untrusted browsing or the unresolved parent/aggregate/
detached-container owners. The full checkpoint stays incomplete; PostgreSQL/demo and
remote CI remain pending. Book/document verification is recorded in the owning leaf.

Primary distribution references:
- https://developer.chrome.com/docs/automation-and-testing/chrome-for-testing
- https://github.com/GoogleChromeLabs/chrome-for-testing
- https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.36/chrome/installer/installers.gni


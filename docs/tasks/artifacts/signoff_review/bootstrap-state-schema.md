# Versioned CLI bootstrap recovery state

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.1. Product baseline: f2c29c3.
Status: schema/publication support qualified by twenty-four selected controls, strict all-target CLI lint and book checks.
The actual CLI still sends no bootstrap key in this prerequisite leaf.

## Matched format baseline

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test bootstrap_state -- --nocapture --test-threads=1`
returns rc=101, 0/1 passed (build 7.67s; execution 0.00s). Both pending-only and
completed-only version-two records refuse to load/round-trip, while the exact
owned raw bytes remain unchanged. This is an absent format capability, not a
claim that rejecting an unsupported schema was a prior decoder defect.
All record-* fixtures are removed; result consumed before product edits.

## Selected schema and lifecycle

StateFile version two adds bootstrap metadata with required nullable pending and
completed fields. Version zero remains empty legacy state, and version one
retains the original principal/thread format without a bootstrap field. Public
Rust callers gain one optional StateFile field; existing local constructors use
Default update syntax. Four explicit public data records describe recovery,
request, completed request/outcome and the full outcome. They carry data, not
credentials or authenticated proof of a server's authority.

A request retains canonical RFC UUIDv7 request identity, a canonical absolute
HTTP(S) server base (maximum 4096 bytes, no credentials/query/fragment), exact
name and the required nullable original action list. Human actions remain
ignored server input, retained here for stable resends. Outcome fields are all
required and bind key, human kind/name, canonical human/tenant identity and the
actual deterministic boundary/grant source strings. Unknown/duplicate fields,
wrong shapes, conflicting bindings and malformed identities refuse. The whole
snapshot keeps its existing 8 MiB encoded/read bound.

Recovery retains one unresolved request and the most recent completed request/
outcome. A completed receipt is historical; later legitimate changes to the
local name mapping need not erase it. Normal fresh invocation and explicit
recovery are distinct intents. The next CLI child adds --resume-bootstrap so a
caller can recover after pending cleanup or lost CLI output without guessing
whether a person consumed stdout. A URL binds the configured endpoint, not
cryptographic server/database continuity; that trust boundary is not widened.

The private Writer gains borrowed persist, reused by consuming publish, so each
intermediate snapshot can synchronize without releasing the guard. Ordinary
writers refuse existing pending metadata before HTTP, even in this intermediate
schema commit. Against the actual currently published snapshot, persistence must
not drop recovery metadata or replace an unresolved request. A completed receipt
changes through the matching pending request, and pending cleanup requires both
its completed outcome and matching principal mapping in the published snapshot.
A valid first snapshot may restore recorded metadata into a legacy/empty store;
this local data validation is not server authentication.

The next flow publishes pending before HTTP, publishes principal/outcome while
pending remains, then clears pending while retaining the completed receipt.
The schema leaf qualifies those storage transitions, not their future HTTP use.
Existing qualified clients reject version two rather than ignore recovery state.

## Qualified scope

New controls cover strict malformed records, byte preservation, transition
refusal and completion requirements, a borrowed guard across multiple snapshots,
five injected intermediate-publication failures, and actual no-dispatch refusal
of an unrelated pending bootstrap by both CLI writers. Prior library/storage/
writer compatibility remains selected. All final results are consumed and unique fixtures are absent before committing
REPAIR-0030. The final escalated `python3 -B scripts/project_env.py bash scripts/check_no_background_jobs.sh`
census returns `handoff: OK`, rc=0; its result is consumed.

All fixture data/logs remain under target/cli-bootstrap-controls (or the already
owned storage/writer fixture roots) on the repository volume. Installed toolchain
and necessary OS metadata are read-only dependencies; no off-volume data or
external service mutation. The decoder samples are fixed owned input, not live
credentials.

Baseline log SHA-256: `14b5c6a262f78a6508c08cf5c29a283f855ee5024bb3e2ef59b67cf452d3a54d` (1084 bytes).

## Owned pre-main wait investigation

The candidate passes all ten library controls and the first three schema controls
(including 51 malformed-record cases) before the publication binary stalls.
Observed PID 80578, Cargo parent 80052, state S, age 4m50s and CPU 0.00s. A
one-second sample contains 763 _dyld_start stacks and a 96 KiB footprint. This
sample does not place execution inside application or test code.

Read-only xattr display shows only com.apple.provenance, not a quarantine
attribute. Signature display reports Mach-O arm64, ad-hoc/linker-signed, CDHash
ae7163dc541e3ffeb6f70274ddbb15111c1eeae8; display alone is not signature
verification. Exact binary bytes: 6,458,064; SHA-256
0bfca288aa761580b35b98f3b7cccb2aae89ac1873f69254d3bad1d9a3e04a1a.
A ten-minute OS log query restricted to this executable name returns only the
query's own log record, so it supplies no causal admission diagnosis. All these
observation results are consumed. Evidence: publication-start.sample and
publication-start-observations.json under the owned artifact root. No security
setting, attribute or signature is changed; no native-trust or OS root-cause
claim is made. The existing .11.2 owner retains underlying host investigation.

The exact identity-checked pre-main process is bounded to ten minutes before
termination and explicit result consumption. A retry must preserve this failed
startup evidence and verify unchanged executable/source, not hide the delay or
weaken the desired controls. This is a bounded verification recovery experiment.

## Final commands and outcomes

The initial candidate returns rc=101 only because the exact pre-main publication
process is terminated after its recorded ten-minute bound (observed 10m40s).
Its ten library controls pass (0.91s), as do its three initial schema controls
(0.54s); build 17.49s. Cargo reports SIGTERM for the publication process, not a
failed application assertion. The orchestration result is consumed.

The unchanged publication executable is rerun directly through project_env.py
with a 180s bound. All four controls pass rc=0 in 0.17s, with identical before/
after SHA-256 0bfca288aa761580b35b98f3b7cccb2aae89ac1873f69254d3bad1d9a3e04a1a.
Result consumed; no signature/attribute/source change is involved. This shows a
successful retry of that binary, not the underlying cause of the stalled launch.

`python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test bootstrap_state --test state_writers -- --nocapture --test-threads=1`
returns rc=0, final four schema and six writer controls (build 6.06s; execution
0.81s/11.10s). The final schema matrix preserves 53 malformed inputs. Legacy
wire shape, canonical historical IDs/endpoints, simultaneous new pending plus
older completion and original action retention pass. Both real CLI writers
refuse a different pending request with zero HTTP requests and unchanged bytes.
All prior overlap/crash/explicit-principal controls pass. Result consumed.

Together, 10 library + 4 schema + 4 publication + 6 writer controls pass on the
implemented product source. They are not represented as one uninterrupted green
run. Only schema test coverage changed after the initial candidate; the original
publication binary is hash-qualified above. No live PostgreSQL or full CI is run
for this storage-schema prerequisite.

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
returns rc=0 in 19.50s; result consumed. Formatting/diff/book checks pass, including
fourteen rendered schema markers, the chapter link and two parsed JSON examples
with request/outcome/source bindings and independent illustrative IDs. A first
text-marker check expected different wording for required nullable fields; it
was corrected to the actual explicit requirement, without weakening that rule.
All record-*, writer-*, e2e-*, state-*, unit-* and sync-probe-* fixtures are absent.
README remains 52 lines/2054 bytes; LIVE_STATUS category values unchanged. No push.

Retained evidence SHA-256 (target/cli-bootstrap-controls):

- schema-baseline.log: `14b5c6a262f78a6508c08cf5c29a283f855ee5024bb3e2ef59b67cf452d3a54d` (1084 bytes).
- schema-baseline.exit: `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` (4 bytes).
- schema-candidate.log: `db2272aac33d8161fed74d8a3b922bb18f5bda6b723d3e60c9ff50efbea9fbfa` (2331 bytes).
- schema-candidate.exit: `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` (4 bytes).
- schema-lint.log: `bc1bd00c4d2e1d1876c6b05e4fbc4a26af686f8b261d9f3a6840bf2a85a872c3` (176 bytes).
- schema-lint.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- schema-final.log: `df34adb21af845fba485cc3f616a5cc2fc910ef7c31a418f7fc9f5edfb6c85f7` (3469 bytes).
- schema-final.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- publication-start.sample: `00503aaacd487b20e4673047c10337aaf435b9ae2ce87fd8b9baa80cc587c407` (1021 bytes).
- publication-start-observations.json: `37d0fee9984324da2d45ae49a3ca68b0ef641425a44fdbaf3cff219ea67e1bfd` (2239 bytes).
- publication-start-deadline.json: `cb61074c743891cfa280938cb199d673f09e262839e4e19d554a722c26106c24` (264 bytes).
- publication-retry.log: `7126272bfe0e69c5ac3cb93368d8a40579330bd1de7d9f1c0308318a063989ef` (618 bytes).
- publication-retry.json: `d17a282f635f1c6995c93b6483f9a70d7e36fa22f59e6729e9e9ffb3aa446dc8` (128 bytes).

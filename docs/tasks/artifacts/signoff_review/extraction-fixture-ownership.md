# Extraction fixture ownership

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.12`; REPAIR-0057. Predecessor:
`ec8df086019284261d9912a270a8693a3fbb921b`. Raw evidence:
`target/pdf-extraction-controls`; failed full checkpoint:
`target/checkpoint-ci/full-ec8df08`.

## Failed checkpoint and unchanged comparisons

The workspace command returns 101 after 669.997550 seconds. The extractor unit
suite passes five and fails two: the text PDF returns zero chunks instead of one
(crates/reasonbraid-extract/src/main.rs:598), and the JavaScript-bearing PDF returns success instead of its named
refusal (line 646). This is not a zero-character observation. Twenty-three result
blocks complete, with 177 nominal passes, two failures, one ignore and five
DATABASE_URL early returns; PostgreSQL and demo commands never start. Eight other
gates pass: format, strict full lint, workspace binary build, 67 Python controls,
book, thirteen doctrines, pinned cargo-deny and pinned Gitleaks. The failed browser
wrapper mac-arm64-0281kcb0 remains preserved. All twelve writer controls and the
new inherited-lock regression pass in this same workspace run.

The failed-checkpoint verifier confirms all 341 committed source identities and
forty absent recorded process groups. Its current-source exception is exactly the
temporary extractor observer, checked against the committed original separately.
The expected complete workspace total of 620 was an inventory, not this run's
observed pass count. Keep this checkpoint failed and incomplete.

Original source, manifest, lockfile and both extractor binaries are copied and
hashed before compilation. Main.rs is 28,686 bytes, SHA-256
7a3b44b395165ee66627abf74ee412e181f80bc39b7678c3dcf6bd2b9c1fd324.
Its helper and dependencies match the preceding complete workspace source.
The original input residue census is empty; deleted original inputs cannot be
reconstructed as evidence.

| Exact original executable comparison | Result | Command seconds |
| --- | --- | --- |
| Default concurrent | seven pass | 0.369079 |
| Isolated text PDF | one pass | 0.021004 |
| Isolated JS refusal | one pass | 0.022092 |
| Serial | seven pass | 0.044475 |
| Bounded concurrent repetition, iteration five | six pass, one fail | 0.046990 |

The repeated failure is the positive ZIP fixture receiving nested_archive for
outer.zip. Only the separate refusal fixture constructs that entry. Five preceding
repeats pass. Source/test/worker identities remain unchanged and every group is
absent. Passing reruns do not invalidate either failure.

## Mechanism observed with the unchanged helper

The shared helper uses PID plus subsec_nanos, then std::fs::write without exclusive
creation. Temporary instrumentation records test/path/intended byte digest. Its
build passes in 473.507896 seconds. The first observer stops at iteration 17 on
JSONDecodeError because libtest stdout interleaves within stderr JSON; all eighteen
test bodies pass, but the observer is failed, not valid full trace evidence. An
exec-only launcher separates the streams without changing the executable. All
100 instrumented seven-test runs pass without a witnessed duplicate. Preserve
those logs; instrumentation changes scheduling and does not disprove collision.
Restore and hash-check original main.rs before permanent correction.

A standalone probe embeds the original helper byte-for-byte. Up to 256 rounds
would launch 32 labelled writers behind a barrier; no file is removed until all
writers return and their actual bytes are compared. With the native clock it
stops in round zero: exit 10, 0.368310 seconds, group 31528 absent. Three paths
are shared by writer pairs 3/30, 6/7 and 23/22, and the earlier owners observe the
other payloads. All 29 unique resulting files remain under .project-data/tmp with
size/hash identities in collision-files.json. This independently establishes
actual clock-name reuse and overwriting. No clock mock, test serialization or
parser change is involved. Exact paths from the original two PDF failures were
not captured, so their historical path attribution remains unavailable.

## Bounded repair and controls

A test-only Input helper now serves unit and both stdio fixtures. It discovers
the current repository from cwd, checks nonlinked same-volume storage parents,
creates private files exclusively under target/extract-tests and retries occupied
process/counter candidates within a finite bound. create_new establishes ownership;
the proposed filename alone does not. Existing files and links are untouched.
Inputs remain alive through assertions. Successful cleanup checks the path's file
identity against its still-open owner; panic retains evidence and replacement
refuses cleanup. This is a controlled test workspace, not a hostile concurrent
same-user directory-mutation boundary.

Three durable controls verify 32 simultaneous independent payloads and cleanup,
existing-file/link/traversal refusal, and panic/replacement retention. The seven
original parser/refusal tests and both actual worker stdio tests retain their
assertions and default concurrency. Parser production, protocol, manifests and
lockfile are unchanged. Selected qualification results and independent locality/
source/process checks are recorded in the owning task leaf before closure.

```bash
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-extract --all-targets -- --nocapture
python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-extract --all-targets --all-features -- -D warnings
```

## Historical scope and production follow-ups

Git binds the original helper to ede2e2a8, PHASE-4.4.2; the task's historical
nine-test claim now carries this ownership qualification. The production R2
input path in server api.rs uses the same clock/truncating-write pattern and
unconditional best-effort removal. That source-level cross-request risk has
immediate next owner SIGNOFF-REPAIR.7.3.3, including runtime reproduction and the
adjacent spawner fixture, before full-checkpoint resumption. Fixture repair does
not qualify production input attribution, worker pipes, resource limits, JS
traversal or container isolation.

The same-mechanism census also owns Git acquisition scratch under .7.2.1 and
remaining journal/stub fixture names under .11.2.1. The unrelated authority
transaction nanosecond precision check is excluded. No production collision is
claimed merely from that census. Full checkpoint, authorized public push and
actual remote CI remain pending.

Final qualification: all twelve distinct tests pass with no skips/ignores in
271.946638 seconds (ten unit, two real stdio); strict lint passes in 20.944596 and
format in 0.544521. The first selected pass is a repeat, making 24 executions of
twelve distinct tests. Three actual-helper cases confirm an occupied first
candidate survives, the same executable works after moving its discovered root,
and a linked parent refuses before writing outside. All process outcomes are
consumed. Source reconstruction verifies unchanged parser/all original assertions
and 339 other existing non-Markdown files. Five original identity copies and all
29 collision files remain; successful fixture storage is empty. Eight rendered
book markers, unchanged README 52 lines/2017 bytes and unchanged qualification
categories pass independent checks. The final task receipt records process census
and book/commit closure.

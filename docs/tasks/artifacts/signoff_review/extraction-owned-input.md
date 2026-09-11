# Exclusively owned extraction inputs

Owner: `SIGNOFF-REPAIR.7.3.3.3.1`; REPAIR-0063. Predecessor: `4dffa40`
(REPAIR-0062, clean tree). Raw evidence:
`target/extraction-owned-input-controls`.

## What was still wrong at this commit

`.7.3.3.1` reproduced the production defect: the R2 span builds its input name
from the process id and a nanosecond field in an ambient temporary directory and
truncates whatever is there, which produced six colliding paths and eight worker
responses describing another caller's document. That reproduction stands; it is
not repeated here.

A census of the current tree confirms the defective span is unchanged at this
commit — `crates/reasonbraid-server/src/api.rs` still calls
`std::env::temp_dir()`, still names the file `r2-input-<pid>-<subsec_nanos>`,
still writes it with the truncating `std::fs::write`, and still removes it
unconditionally after every result. This child builds the owner that replaces
it; `.7.3.3.3.2` performs that wiring.

## What this child establishes

`crates/reasonbraid-server/src/extraction_input.rs` — `OwnedInput`:

- creates one private file per request under `<root>/.project-data/extraction`,
  with the root discovered at RUNTIME from the current directory so moving the
  checkout needs no edit, and with NO temporary-directory or home fallback;
- creates it with `create_new` and mode 0600, trying at most 64 v7-UUID
  candidates. An occupied candidate is SKIPPED — never opened, truncated or
  adopted;
- proves every parent directory is owned by this process, on the same device,
  and not group- or other-writable. The ownership reference is the uid of the
  file it just created, since a new file carries the creating process's
  effective uid — so the check needs no dependency outside the standard library;
- records the created file's (device, inode) and exposes the digest of the bytes
  written, in the worker's own `sha256:<hex>` form, so a caller can bind a
  response to the source it supplied;
- removes the input only when no reader can still hold it AND the file is still
  the exact one created — same device, same inode, one link. Anything else is
  retained with its repository-relative path named.

The release predicate is `WorkerCompletion::reader_finished`, added beside
`is_consumed` in `.7.3.3.2.2`'s evidence type. `NeverStarted` releases: no child
was created, so nothing can be reading. Only `Unconfirmed` retains. Gating on
`is_consumed` instead would litter the private store on every absent binary and
every failed spawn while proving nothing extra.

The runtime root predicate is deliberately stricter than the browser worker's
`Cargo.toml` + `migrations` pair: `crates/reasonbraid-node` satisfies that pair,
so a process whose working directory sat there would place private storage
inside the crate. `rust-toolchain.toml` exists only at the real root. The browser
worker's own predicate is NOT changed here; it is routed to
`SIGNOFF-REPAIR.7.3.2`, which owns that worker's storage boundary.

## Controls

Eleven pass — nine in the module, where the single-candidate seam lives, and two
integration controls through the real worker:

| Control | Proves |
| --- | --- |
| `the_digest_is_the_worker_s_own_form_over_the_written_bytes` | the exposed digest is the worker's `sha256:<hex>` form over exactly the written bytes |
| `an_unconfirmed_reader_retains_the_input_and_names_its_evidence` | an unconfirmed reader retains the file, names the completion evidence, and never emits an absolute path |
| `a_never_started_worker_releases_the_input_it_never_read` | the one failure that releases, because no reader ever existed |
| `the_store_resolves_to_the_repository_root_not_the_crate` | ancestor discovery reaches the repository root from the crate directory |
| `an_occupied_candidate_name_is_never_opened_or_truncated` | the occupant's bytes and (device, inode) survive a candidate collision |
| `a_symlinked_candidate_name_is_refused_and_its_target_untouched` | nothing is written through a link; the target is unchanged |
| `a_replaced_input_is_retained_and_its_successor_survives` | a changed identity refuses removal and the successor is never deleted |
| `dropping_an_unreleased_owner_retains_the_input` | an owner dropped without release retains the file |
| `simultaneous_creators_never_share_a_path_or_a_document` | 32 concurrent creators hold 32 distinct paths, each with exactly its own bytes and digest |
| `the_real_worker_describes_the_owned_input_by_its_own_digest` | the ACTUAL worker's `parent_digest` equals the owned input's digest, and release follows a finished reader |
| `a_response_describing_other_bytes_disagrees_with_the_owned_input` | a response for other bytes is detectable, and the input is still owned when it is |

## One control failed, and its own retention identified it

The post-format run returned `FAILED. 1 passed; 1 failed` for
`--test extraction_input`. The failing test name was not captured before the next
invocation, and 40 consecutive direct runs of the compiled binary then returned
`test result: ok` — a narrow window, not a reproducible schedule.

Two independent observations close it rather than a rerun:

1. A deliberate widening probe (`race-probe.rs`, preserved) holds
   `R2_WORKER_BIN` for 1500 ms while an unsynchronized caller of the same shape
   runs. That caller receives the stub's
   `sha256:0000000000000000000000000000000000000000000000000000000000000000`
   against its own `sha256:92e3…` — the exact assertion that failed.
2. The failed run RETAINED its input at
   `.project-data/extraction/input-01a08f5b-add5-71f2-9414-b4591b5c7b66`,
   187 bytes, `sha256:1b7e2f3b…` — the real-worker control's own Atom feed. The
   input held the caller's own bytes, so the response was wrong, not the input.
   Only the real-worker control asserts digest EQUALITY, so the stub control
   cannot produce that retention.

Root cause: `the_real_worker_describes_the_owned_input_by_its_own_digest` read
ambient worker selection without holding the file's `worker_selection()` lock,
so its sibling's process-wide `R2_WORKER_BIN` override could hand it another
control's worker. Both controls now hold that lock across selection AND the
spawner call; 40 repeated runs of the serialized binary return zero failures.

The defect is the same class this whole family is about — shared ambient state
standing in for owned state — and it appeared in the controls written to repair
it. The retained input is left in place as evidence.

## Verified

| Check | Command | Result |
| --- | --- | --- |
| 9 module controls | `cargo test -p reasonbraid-server --lib extraction_input` | 9 passed, 0.23s |
| 2 integration controls | `cargo test -p reasonbraid-server --test extraction_input` | 2 passed |
| serialization stability | 40 repeated runs of the compiled control binary | 0 failures |
| 90 server library tests | `env -u DATABASE_URL … --lib` | 90 passed, 9.70s |
| 16 completion controls | `cargo test -p reasonbraid-server --test extraction_completion` | 16 passed, 5.15s |
| 12 adjacent extractor tests | `cargo test -p reasonbraid-extract --all-targets` | 12 passed |
| strict lint | `cargo clippy -p reasonbraid-server --all-targets --locked --offline -- -D warnings` | rc=0, 1m22s |
| format | `cargo fmt --all -- --check` | rc=0 |

## Honest limits

- No production caller uses this owner yet. `api.rs` is unchanged and still
  carries the defective span; `.7.3.3.3.2` wires it, binds the response digest
  and gates cleanup, and the adjacent in-module spawner test still writes its own
  input under `std::env::temp_dir()` until that wiring.
- The digest binding is PROVED to be observable here; the refusal that consumes
  it before persistence is `.7.3.3.3.2`'s.
- Ownership is checked against this process's own effective uid and the parents
  as they are at creation. It is not a defence against an administrator, nor
  against a hostile process that can already write inside `.project-data`.
- Synchronous pipes, total deadlines, descendant containment and aggregate
  retained storage remain `SIGNOFF-REPAIR.7.3.4`. No HTTP handler, database
  write, full CI run or public push is claimed.

---

## The wiring (`SIGNOFF-REPAIR.7.3.3.3.2`, REPAIR-0064)

At `b9eb9e8`, `git grep -n "temp_dir" -- crates/reasonbraid-server/src/api.rs
crates/reasonbraid-server/src/extraction.rs` returned two live uses: the R2
handler's input span and the adjacent in-module spawner fixture. The handler
removed its path unconditionally after every result and compared nothing about
the response, so a receipt describing other bytes would have been persisted as a
snapshot and its derivations. After the wiring the same command returns nothing.

`extract_acquired_bytes` performs the whole boundary in one call — owned private
input, spawner call, digest binding, release gated on the reader being finished —
and the handler's twenty-five-line span becomes that call.
`ExtractionError::SourceMismatch { expected, received }` is a named variant; the
enum stays exhaustively matched, so the handler could not compile without mapping
it, and it maps to the wire kind `extraction_source_mismatch`. The refusal
therefore happens before any snapshot, derivation or receipt write.

| Control | Result |
| --- | --- |
| `the_bound_extraction_returns_a_receipt_for_the_supplied_bytes` | the real worker's receipt is the digest of the supplied bytes |
| `a_named_refusal_survives_the_bound_extraction` | `feed_unreadable` survives verbatim |
| `a_response_for_other_bytes_is_refused_before_it_can_be_persisted` | a stub returning `sha256:0000…` produces `SourceMismatch` naming both digests |
| `a_worker_failure_keeps_its_classification_through_the_boundary` | a nonzero exit stays `RequestFailed` |
| `concurrent_bound_extractions_each_describe_their_own_document` | eight simultaneous callers each receive their own document's digest and text |

Seven integration controls pass in 0.66s; 90 server library tests, 16 completion
controls, 12 adjacent extractor tests, strict server lint (1m25s) and workspace
format pass. The live adjacent `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles`
returns `31 passed; 0 failed` in 34.29s, including the R2 resolver control, and
its disposable cluster reports `stopped and removed`.

### The live coverage gap this exposed, with its census

No live test drives a SUCCESSFUL R2 acquisition through to a snapshot and a
derivation. The census: `git grep -n "R2_RESOLVER_ID\|r2-extract-worker\|
extraction_version" -- 'crates/**/tests/*.rs'` returns exactly two hits — a
resolver-ranking assertion and a hand-submitted derivation — and
`git grep -n "acquisition\"\]" -- 'crates/**/tests/*.rs'` returns none. The live
R2 test refuses at the loopback SSRF gate before reaching the extraction leg, and
an outbound Internet fetch is not an acceptable test dependency.

That join is exactly where the misattribution would have become a persisted wrong
record, and its absence is a plausible reason the input defect survived a full
historical phase closure. The extraction leg itself is now covered against the
exact function the handler calls. The missing join has the concrete owner
`SIGNOFF-REPAIR.7.3.3.4`, which must supply a policy-allowed local origin rather
than weaken the production destination rules.

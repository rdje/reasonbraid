# Dedicated browser prerequisite

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.5`; REPAIR-0046. Raw records:
`target/ci-browser-controls`. Predecessor: e5d7eec. Production Rust, worker
executables and Cargo manifests/lockfile are unchanged by this leaf.

## Exact archive selection

Chrome for Testing **153.0.8010.36**, official versioned objects beneath
`https://storage.googleapis.com/chrome-for-testing-public/153.0.8010.36/`.
Each platform uses `<platform>/chrome-<platform>.zip`. Selection metadata and
the initial macOS ARM64 archive are preserved under
`target/browser-checkpoint-controls`. Official project/distribution context:
[Chrome for Testing](https://developer.chrome.com/docs/automation-and-testing/chrome-for-testing)
and [versioned release API](https://github.com/GoogleChromeLabs/chrome-for-testing).

| Platform | Archive bytes | SHA-256 | Published object MD5 |
| --- | --- | --- | --- |
| mac-arm64 | 191016009 | 1f701ef60757c63c6ccf98afaf28291dd0c8d1457d3d738e81fd62201c230ad0 | W69kLZi52eVlvWjrR4r0Mw== |
| mac-x64 | 201449122 | cddfd83fadf88808fb036f44282488b1cf3f1b50efacce470225ae41f14b0302 | vhfyaiytC4cIqRMF++e6wQ== |
| linux64 | 195711476 | 167a098c4fdec156b58a9f678c90a84f9072d789f9c6e7b35496a6987b8b7ef8 | jsuWg1AeWW4w5xH9sD+gug== |
| linux-arm64 | 195918275 | dfc4955719c5d494c8507990506d2d5bed174c31bf89266aa2dc5593c6607e8b | CPeXIXIXHoqfMMu6ibW9SQ== |

Source pins use SHA-256 plus exact length; runtime setup does not download its
trust record or accept an environment override. Archive payloads match published
object MD5 metadata. Both macOS packages have 675 entries, 346 regular files and
five internal framework links; both Linux packages have 308 entries, 303 regular
files and no links. Independent extraction re-derives every regular-file hash and
link target and verifies the repository volume. Temporary verification extractions
are removed; archives and evidence remain. This is all-platform archive validation,
not native execution on every platform. Exact counts/hashes:
`target/ci-browser-controls/archive-layout-verification.json`.

## Implementation contract

`scripts/ci_browser.py` requires a supported platform, creates an exclusive private
installation directory, verifies original archive bytes before ZIP parsing and
validates the entire layout before writing. Internal link chains are bounded;
cycles, root escape, duplicate members, missing targets and writes beneath a file
or link refuse. Regular extraction streams enforce declared sizes and ZIP CRCs;
the executable is regular, nonempty and executable. Only owner permissions survive.
Archive/expanded/per-file bounds are 300 MiB/1 GiB/512 MiB, with 20,000 entries.

Download/version/command phases use the shared consumed process supervisor with
300-second/30-second/one-hour defaults. The command accepts an explicit 1–7200-second
deadline. Receipt fields identify root-relative installation/executable paths,
archive and executable hashes, exact version and child phase PIDs/results.
Browser selection and diagnostic log overrides are cleared. Successful calls remove
only their own archive/runtime; failures retain them. Receipts/version logs remain.
The Make test/check path and Rust workflow require this wrapper after locked worker
builds. CI uploads only receipt/version metadata; job logs hold command output.

The official macOS build has an ad-hoc linker signature. HTTPS provenance, pinned
archive integrity and version are not Developer ID authentication. No re-signing,
quarantine/OS change or desktop browser/updater mutation occurs. This prerequisite
qualifies trusted fixtures; production hostile-content, detached containment and
aggregate retention remain `.7.3.2`.

## Preserved diagnostic failures and repairs

The first three-archive diagnostic used executor threads with run_command, whose
cleanup changes signal handlers and therefore requires the main interpreter thread.
The driver fails; downloader exit codes were not captured and are not claimed.
Independent native inspection proves all three recorded groups absent; separate
published-object hash/length and full extraction checks establish complete archive
payloads. Subsequent supervised commands run on the main thread. Preserve
`acquire.py`, `initial-acquisition-refusal.json` and each original receipt/log.

Thirteen initial launcher controls pass. The next expanded captured run encounters
three fixture interpreters before their first entry witness. Its test harness uses
subprocess.run's abrupt outer timeout, killing each launcher before it can consume
its independently grouped downloader. Native ps identifies three exact orphan
fixtures; identity-checked TERM cleanup proves each group absent. A bounded sample
of PID 98334 shows 85 samples at `_dyld_start`, without binary-image information.
That localizes the wait before Python entry; its deeper host cause remains `.11.2`.
Preserve all four affected fixtures and `focused-controls.log`; the real browser
step was unstarted. The harness now uses cooperative run_command shutdown, with
an explicit stalled-downloader outer-timeout control.

The corrected collection then has fourteen passes/one failure: a stalled version
probe records PermissionError during shutdown. Three matched native children exit
between process polling and group inspection; ps shows each as a zombie and the
old helper raises EPERM. After explicit reaping each group disappears. The repaired
shared supervisor reaps/reobserves within its existing deadlines and never accepts
a denied observation as absence. Three matched native post-repair controls pass;
each zombie is reaped with exit zero and its group is absent. Persistent probe and
signal denial still fails. Original and matched receipts:
`native-shutdown-race-before.json` / `native-shutdown-race-after.json`.

## Verification

- All 67 Python controls pass in 21.834 seconds, exit zero, including fifteen new
  browser controls, two shared-shutdown controls, scanner/environment controls and
  all four native PostgreSQL lifecycle controls. Exact output:
  `target/ci-browser-controls/python-regression.log`.
- All four archive payload/layout checks pass; their temporary extractions are
  absent, raw archives retained. Matched native shutdown controls change from
  three PermissionError results to three consumed successful reaps.
- Fresh native wrapper completes in 89.966 seconds, exit zero: official download,
  exact version and all sixteen integration tests pass (test body 30.22 seconds,
  no ignored/filtered tests). Receipt: target/ci-browser/mac-arm64-9et1az_i/browser.json.
  Native version/executable hash matches the predecessor's qualified runtime.
  Raw log: target/ci-browser-controls/native-browser.log.
- Final independent verifier returns zero: sixteen worker and ten browser receipts,
  plus four setup/supervisor groups, are all absent. Successful fixtures and the
  setup payload are absent; seventeen historical browser failures and all five
  launcher-control failures remain. Production browse/extract binary hashes are
  unchanged. Live PostgreSQL control workspaces are absent.
- YAML parsing, Rust-job shell syntax, Make dry-run routing and two deliberate
  prerequisite omissions pass. Book build, seven rendered markers and diff checks
  pass. README stays 52 lines and shrinks to 2,017 bytes. Exact results:
  target/ci-browser-controls/final-verification.json.

Full workspace, full PostgreSQL/demo and remote CI qualification remain parent
`.11.4.3.1.2`; no release or production qualification category advances here.

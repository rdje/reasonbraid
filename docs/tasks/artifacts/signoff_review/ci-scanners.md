# Pinned CI scanner installation and execution driver

- Owner: `SIGNOFF-REPAIR.11.4.3.1.3.2`; REPAIR-0035.
- Baseline: 674c3b1; scripts/ci_scanners.py and scripts/tests/test_ci_scanners.py are the implementation/control surfaces.
- Scope: installer integrity, platform catalog, command routing, exact version and process/file lifetime. Full dependency/secret gates, workflow wiring and remote execution remain separate owners.

## Source finding and release identity

The existing workflow references cargo-deny-action v1. Read-only git ls-remote
identified annotated tag ef301417264190a1eb9f26fcf171642070085c5b; peeling the tag
and independently reading its GitHub API object identify commit
3f4a782664881cf5725d0ffd23969fcce89fd868. Initial raw URLs using the tag-object SHA
returned 404 because it is not a commit; exact committed source was then retrieved.
The [immutable Dockerfile](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny-action/3f4a782664881cf5725d0ffd23969fcce89fd868/Dockerfile)
pins Rust 1.71.0 and cargo-deny 0.14.21. Its image/installer stores do not establish
our repository-local toolchain contract. Local policy targets cargo-deny 0.20 and
the repository pin is Rust 1.98.0. This establishes the source/version mismatch;
no remote workflow failure was run or inferred. The old Gitleaks workflow streams
a downloaded archive into tar without a pinned digest check.

Use official prebuilt cargo-deny 0.20.2 and Gitleaks 8.30.1 releases. Pin archive
name, byte count and SHA-256 in source; never obtain the expected digest from the
network at execution time. Release metadata was retrieved from the official
[cargo-deny release API](https://api.github.com/repos/EmbarkStudios/cargo-deny/releases/tags/0.20.2)
and [Gitleaks release API](https://api.github.com/repos/gitleaks/gitleaks/releases/tags/v8.30.1).
All eight archives were downloaded and independently hashed, matched against API
metadata and published checksum files, then read by the production bounded decoder.
All expected regular executable members and size limits pass, rc=0. This catalog
probe executes no binaries. Actual native probes are recorded separately below.

| Archive | Bytes | SHA-256 |
| --- | --- | --- |
| cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz | 4936832 | 9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f |
| gitleaks_8.30.1_linux_x64.tar.gz | 8230402 | 551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb |
| cargo-deny-0.20.2-aarch64-unknown-linux-musl.tar.gz | 4631618 | 995c82be0defc7a025cae49a2aa2644ce8245c9a3318fc4103907c6a285e8c7d |
| gitleaks_8.30.1_linux_arm64.tar.gz | 7601421 | e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080 |
| cargo-deny-0.20.2-x86_64-apple-darwin.tar.gz | 4731454 | 248da7f581724e470071990c088ffc55c811981715f4cbdb258621fb79f8b7a6 |
| gitleaks_8.30.1_darwin_x64.tar.gz | 8359235 | dfe101a4db2255fc85120ac7f3d25e4342c3c20cf749f2c20a18081af1952709 |
| cargo-deny-0.20.2-aarch64-apple-darwin.tar.gz | 4517865 | fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3 |
| gitleaks_8.30.1_darwin_arm64.tar.gz | 7897593 | b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5 |

Raw release/API/checksum/action source observations and catalog-verification.json
remain under target/ci-workflow-controls/scanners. The source pins and this record
preserve exact recovery identities if generated data is later retired. Necessary
installed Python/curl/OS/selected compiler inputs remain read-only; curl 8.22.0
was available locally. No global tool installation or shared cache was modified.

## Driver contract

Each invocation validates project stores and creates an exclusive directory under
target/ci-scanners. It uses the shared CI environment to clear inherited gate
controls. Curl receives --disable first, so default curlrc files cannot change the
download; SSLKEYLOGFILE and QLOGDIR are removed to prevent inherited diagnostic
output destinations. The local presence census found neither those variables nor
CURL_HOME/CURL_SSL_BACKEND/CURL_CA_BUNDLE/SSL_CERT_FILE/SSL_CERT_DIR set. This
observation is not a guarantee about a later caller; the driver enforces its own
controls. Explicit other tool paths still require their locality contract.

Downloads require HTTPS through redirects, a ten-second connection limit,
120-second curl limit and a 125-second supervisor bound covering native/DNS/TLS
waits. The archive must match its exact pinned length and hash before decoding.
Bounds are 16 MiB compressed, 96 MiB expanded tar, 64 members and 64 MiB for the
executable. Refuse linked/special, absolute/traversing, duplicate executable,
missing executable and oversized entries. Read only the expected regular member;
never extract archive paths. Publish its bytes exclusively with mode 0700 and
check the written digest before execution.

The exact --version output must match within thirty seconds. Cargo-deny emits
"cargo-deny 0.20.2"; Gitleaks emits "gitleaks version 8.30.1". --verify-only stops
after installation/version qualification. Default mode runs either cargo-deny
--locked check or Gitleaks detect over Git history with full redaction, a bounded
run and a root-relative JSON report. It does not silently add an offline advisory
mode, alter deny.toml or introduce allow-list exceptions. The full dependency gate
must later establish fresh advisory/index results and inspect warnings.

All external commands use the existing process supervisor. Child PID/phase is
recorded before waiting; terminal-safe atomic receipt replacement preserves the
last recorded phase. A failed/interrupted receipt can retain a stale started PID:
inspect the actual process group before removing data, never infer live/dead from
that field alone. Setup failure prevents scanning. A nonzero scanner result stays
nonzero; receipt state completed means execution ended, not that a gate passed.
Read its scope and exit_code too. Requested operation lifetime is bounded at
twenty minutes. Terminal handling consumes the supervisor's cleanup before return;
shutdown failures retain the last identity and data for inspection.

After consumed normal execution, retire only that invocation's known archive and
executable; retain scanner.json, version.log, check.log and redacted report as
applicable. Setup failure/interruption retains diagnostics and files. This is no
permission to remove another run or historical evidence. No shared mutable tool
installation is used. Hash verification establishes the reviewed release bytes,
not independent review of their publisher or completion of G9 supply-chain gates.

## Executed controls and corrected native finding

The initial eleven-control instrumented suite passed in 3.080s, rc=0. A native
--verify-only run then passed cargo-deny and correctly refused Gitleaks according
to the then-incorrect expected string, rc=2. The archive still matched
b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5. Its retained
version.log held exactly 24 bytes: "gitleaks version 8.30.1" plus newline.

Root-cause probe: on the same retained executable (SHA-256
ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f), --version emits
the prefixed line while the version subcommand emits bare "8.30.1"; both rc=0.
The earlier installed-tool census used the subcommand. Correct the driver's exact
expected output and its instrumented fixture; do not broaden accepted versions.
This illustrates why the independent real-binary probe remains necessary even
when an instrumented suite is green. Both the failed log and successful retry
are retained.

The corrected full scanner suite passes 13 controls in 4.081s, rc=0. It covers
real child-copy setup/version-only and nonzero gate flows, exact root and local
stores, full redaction/report routing, altered bytes/length, failed download,
wrong version, six unsafe/missing/duplicate member cases, three resource-bound
refusals, linked destination preservation, terminal cancellation and exact version
child timeout/reaping. Synthetic release pins exist only in isolated test copies;
production accepts no runtime pin override. The final two affected configuration
controls pass again in 1.251s, rc=0, checking --disable first and absent TLS/QUIC
log destinations. Unique control fixture directories are absent.

| Native receipt | Outcome |
| --- | --- |
| target/ci-scanners/cargo-deny-tusa2ypb | version-only completed, rc=0; archive/executable retired; logs retained |
| target/ci-scanners/gitleaks-5issw4mh | original version-format refusal, rc=2; original archive/executable/log retained |
| target/ci-scanners/gitleaks-66e0zi8q | corrected version-only completed, rc=0; archive/executable retired |
| target/ci-scanners/gitleaks-flzu_pom | final curl-configuration version-only completed, rc=0; archive/executable retired; final recorded process group absent |

All native runs are aarch64 macOS. Linux/Intel archive identity/layout checks do
not claim execution on those platforms. Native and unit outputs are retained as
candidate.log/.exit, candidate-final.log/.exit, configuration-final.log/.exit,
native-*.log/.exit, gitleaks-version-contract.json and final-verification.json
under target/ci-workflow-controls/scanners. Final wrapped Python syntax/source/exit/
residue verification passes, rc=0: seven adjacent source/policy/workflow files are
byte-identical to 674c3b1, five successful exit receipts and the original failed
receipt are preserved, all eight catalog entries exist and zero control fixture
directories remain. All four native recorded process groups are independently
absent; successful archives/executables are retired and failed evidence remains.
make book, eight rendered operator-contract markers and git diff --check pass,
rc=0. Final staged doctrines run in the commit hook. No real dependency check, secret
scan, full CI, workflow change or push occurred in this leaf.

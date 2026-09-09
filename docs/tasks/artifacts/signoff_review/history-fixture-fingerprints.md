# Exact history-fixture fingerprint qualification

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.2`; REPAIR-0044. Source:
`457d3a78c8fd2f58a4d9b0fa71093e93daff7af2`. Raw evidence:
`target/history-scan-controls`; original full-checkpoint failure:
`target/ci-scanners/gitleaks-9u8ke3yi`. No Rust source, scanner implementation,
remote setting or existing Git history is changed.

## Classification from historical source and use

The two generic-api-key findings originate at commit
`82155f53146585dc047734447e7288ed5c14e187`,
`crates/reasonbraid-server/tests/pg_guard.rs`, lines 68 and 110. They are identical
48-digit hexadecimal fixture literals, constructed by repeating the sixteen
ascending hexadecimal digits three times. They are predictable test data, not
issued service credentials or a live PostgreSQL ownership proof.

The original pg_guard.rs is byte-identical to current source, SHA-256
`03d85c8bfc125a16c452ed9908c4a5fcebd7d35b904cc11162ac663d20f0b6a7`.
Both consuming tests are synchronous local metadata controls: they create private
receipt/postmaster files, call Ownership::read and assert URL/receipt/path refusals.
Neither calls connect, awaits a network operation or starts a server. Their local
fixed port/PID fields are synthetic metadata, not an issued proof.

Historical Ownership::read checks token shape, exact URL and bounded local
metadata before retaining the string in the object. It does not make a connection.
The historical/current support-module diff changes only visibility/documentation
of the separate verify method. Real pool connection proof comes through
connect/verify, and the live runner creates independent randomness with
secrets.token_hex(24); the third live pg_guard test reads that runner-owned value
from its environment. The fixture literal is not that live proof. The provenance
probe and complete source/diff review establish these two findings as non-secret
fixture data; classification does not generalize to arbitrary hex strings.

## Narrow policy and pinned implementation

.gitleaksignore contains exactly the two reported historical fingerprints, each
binding commit, file, detection rule and line. There is no file/rule/regex exclusion,
blanket baseline, inline allow comment or history rewrite. The rule still examines
new occurrences in these same tests. Comments name the owner and durable decision.

The [pinned Gitleaks 8.30.1 documentation](https://raw.githubusercontent.com/gitleaks/gitleaks/v8.30.1/README.md)
describes individual finding fingerprints and labels this feature experimental.
Requalify semantics when the scanner pin or exception changes. The actual
[pinned Detector implementation](https://raw.githubusercontent.com/gitleaks/gitleaks/v8.30.1/cmd/root.go)
loads an explicit ignore file and also the source-root .gitleaksignore. An alternate
ignore path adds to the source's policy; it does not replace that policy.

The first omission probe expected two findings with an explicit empty ignore file
but returned zero because the real source-root policy was still loaded. The
assertion failed, both groups were consumed, and controls.json/unexcluded.log/.json
remain preserved. This is a diagnosed probe-precondition error, not a negative
control pass. The source's two independent loads explain the result.

## Corrected native controls

A new same-volume bare repository receives the exact source HEAD/history through
a local fetch. No remote configuration, FETCH_HEAD or object alternates are stored.
It has no working-tree ignore file. Use it for omissions without temporarily
weakening the real repository's policy. The actual root uses its normal default
policy in a separate scan. Every scan uses the unchanged verified native Gitleaks
8.30.1 executable, SHA-256
`ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`,
with full redaction and bounded supervised execution.

| Control | Result |
| --- | --- |
| Identical history, empty ignore input, no source-root ignore file | rc=1; exactly both original fingerprints |
| Include only the line-68 fingerprint | rc=1; exactly line 110 remains |
| Include only the line-110 fingerprint | rc=1; exactly line 68 remains |
| Actual repository/default .gitleaksignore with both entries | rc=0; zero findings |
| Independent new commit with identical file, lines and literal contents | rc=1; both new-commit fingerprints remain detectable despite the real two-entry policy |

The independent synthetic commit is
`bca87366dd2222a418e31557c84932e675bbcbf1`; it is only in the isolated diagnostic
repository. All expected reports are fully redacted. Eleven scanner/Git commands
complete with their expected exits, and all eleven groups are independently
absent. The original scanner bytes are unchanged. Five controls therefore retain
both omission sensitivity and future same-file detection. qualified/controls.json
and verification.json bind commands, identities, outputs and locality.

## Actual entrypoint and limits

The actual `python3 -B scripts/ci_scanners.py gitleaks` confirmation passes,
rc=0, 4.093s, with its group consumed. Its verified 8.30.1 scanner returns an
empty report; the child is reaped and temporary archive/executable are retired.
Evidence: target/ci-scanners/gitleaks-q5qhbdbc. Independent checks prove the bare
and original reachable-history sets contain the identical 313 commits, all eleven
control groups are absent, retained tool bytes match, and six rendered book markers
are present. make book and git diff --check pass, rc=0. Rust and scanner sources
are unchanged; only the two-entry policy changes behavior.
The full checkpoint will scan the resulting committed history again before push.
A passing configured history scan does not prove absence of every possible secret,
scan uncommitted files or close the other Rust/PostgreSQL/release gates. Original
failed and corrected diagnostic results remain retained.

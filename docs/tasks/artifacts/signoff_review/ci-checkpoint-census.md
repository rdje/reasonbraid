# Scheduled CI checkpoint census

- Owner: `SIGNOFF-REPAIR.11.4.3.1.1`; REPAIR-0033.
- Source: `6bc76c6b56abb624858d04736965c1c17c9da4f4`; recorded origin/main is 301 commits behind this source. No full CI or push was performed by this inventory.
- Scope: read-only source, locked offline Cargo metadata, tool versions/help, environment-variable presence and filesystem metadata. Availability, registered targets and early-return test successes do not establish executed assertions or release qualification.
- Raw records: target/checkpoint-ci/{inventory,target-census,artifact-census,fixture-and-tool-boundaries}.json, metadata.stderr, deny-help.txt, deny-check-help.txt and gitleaks-help.txt. These local records are retained; the source identity, derivation, aggregate observations and repair owners below survive through Git if generated output is lost.

## Required execution matrix

All local commands enter `python3 -B scripts/project_env.py`; Make and the PostgreSQL shell runner already do so. Remove inherited DATABASE_URL for workspace checks. The full checkpoint executes on a named committed source after the prerequisite repairs below.

| Gate | Actual required command/scope | Evidence required before pass |
| --- | --- | --- |
| Format | cargo fmt --all -- --check | exit zero |
| Strict lint | cargo clippy --all-targets --all-features -- -D warnings | exit zero, exact target/features |
| Worker prerequisite | cargo build --workspace --bins --locked | installed extraction/browser worker artifacts, not an absent-worker skip |
| Workspace | cargo test --all --locked with DATABASE_URL removed | per-target results and exact skip/ignore census; browser and extraction actually executed |
| PostgreSQL | bash scripts/run_pg_tests.sh --demo | all 40 registered commands plus demonstration; actual assertions, shutdown and consumed receipts |
| Python controls | python3 -B -m unittest discover -s scripts/tests -p 'test_*.py' -v | all four modules, including live runner shutdown controls, actually executed |
| Doctrines | make gate | all 13 checks on intended committed/staged scope |
| Dependencies | cargo deny --locked check | fresh advisory/index result, bans/licenses/sources and warnings inspected; no new blanket exemptions |
| Secret history | gitleaks detect --source . --redact | exact history scope and redacted result; default detect is Git history, not a working-tree file scan |
| Book | make book | build and rendered changed content verified |
| Publication | normal push to existing private tracking branch | full local gate first; remote advancement and triggered workflow outcomes consumed |

There is no pre-push hook. Current workflow source has three workflows: rust (check and pg-tests), doctrines and supply-chain. The Rust check job uses raw cargo after the toolchain action; only pg-tests currently establishes repository-local compiler stores. No workflow invokes the four Python control modules. Supply-chain uses a cargo-deny container action whose writable-store boundary must be established in the workflow repair. Local Make targets already enter the project launcher. Do not equate this source review with GitHub execution.

## Target and skip census

`cargo metadata --format-version 1 --no-deps --locked --offline` returned zero. Counting package entries and targets with test=true yields 12 packages and 86 test-enabled targets. This counts lib/bin/integration targets, not test functions. Compare the server integration target names with `scripts/run_pg_tests.py` SERVER_SUITES: 38 registered server suites, no registration without a target, and three other server targets. Full source reads show mtls (127 lines) uses local TLS, publisher (104 lines) uses bare Git, and reconciler (124 lines) uses pure matrix/idempotency controls; none requires PostgreSQL. CLI cli_end_to_end and MCP lib are the two additional registered runner commands, making 40.

The direct source-marker census is a navigation aid, not a proof about every transitive helper. Runtime result inspection remains mandatory. The registered server list is:

- pg_guard
- atomic_transaction
- outbox_worker
- node_channel
- authority
- authority_transaction
- authority_issuance
- enrollment_transaction
- bootstrap_recovery
- budget
- command_api
- node_work
- aggregate_library
- identity_store
- node_enrollment
- node_inbox
- invitations
- backup_restore
- migration_upgrade
- escalation
- node_replacement
- profiles
- evaluation
- routing
- policy
- rls
- quota
- quarantine
- classification
- federation
- cards
- mcp_listen
- mcp_write
- allowlist
- regions
- site_authority
- site_operator_cli
- site_registry_http

| Boundary | Current source behavior | Checkpoint treatment |
| --- | --- | --- |
| PG helpers | early return without DATABASE_URL; direct caller DB without owned receipt refuses | offline returns are not live passes; run the full owned collection |
| backup_restore | pg_dump/pg_restore absence can skip | verify installed PG 16 tools and actual backup/restore assertions |
| browser_roundtrip | absent browser returns SKIP | installed executable Chrome is available locally; explicit runtime evidence required |
| server extraction control | missing worker binary returns SKIP | build worker bins and inspect actual execution |
| core write_schema_goldens | ignored generator rewrites tracked goldens | deliberately excluded; not a missing runtime assertion |
| Codex live qualification | ignored and RB_LIVE_CODEX gated | external provider execution excluded without requested live-provider qualification |
| Claude live qualification | ignored and RB_LIVE_CLAUDE gated | same explicit external boundary; do not claim live provider success |
| Python controls | five project-env, thirteen runner unit, four live runner, seven acceptance methods | 29 declared controls; actual count/pass must come from execution |

No caller DATABASE_URL, RB_DEMO, R3_BROWSER_BIN, GITLEAKS_CONFIG, GITLEAKS_CONFIG_TOML, RB_LIVE_CODEX or RB_LIVE_CLAUDE was present in the census process. Only presence booleans were recorded. No tracked .gitleaksignore, .gitleaks.toml or .cargo/config[.toml] was found. Later execution must check its own inputs rather than inherit these observations as a guarantee. Force --demo for the broad runner, since ambient RB_DEMO=0 otherwise changes its default.

## Tool and data boundary

The twelve version probes returned zero: Cargo/rustc 1.98.0, rustfmt 1.9.0, Clippy 0.1.98, cargo-deny 0.20.2, gitleaks 8.30.1, mdBook 0.5.4, Python 3.14.7, PostgreSQL 16.15, jq 1.7.1-apple, Git 2.50.1 and gh 2.100.0. Installed executables are necessary read-only inputs on device 16777232; repository-owned outputs are on device 16777244. No tool installation was performed. Tool availability is not a gate result.

The launcher replaces 17 writable-store variables with target or .project-data descendants, validates directory/symlink/device boundaries and selects the pinned installed compiler read-only. It does not sanitize every caller control or confine arbitrary explicit paths. The pinned cargo-deny 0.20.2 [advisory configuration implementation](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0.20.2/src/advisories/cfg.rs) chooses CARGO_HOME/advisory-dbs when db-path is absent; its [cargo_home implementation](https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0.20.2/src/lib.rs) honors a nonempty CARGO_HOME before home fallback. Thus the launcher supplies the local advisory destination. Execution must still verify other scanner/download outputs and freshness; an offline database is not a fresh advisory pass. Installed gitleaks help confirms full redaction by default with --redact and no implicit report path. No scanner diagnostics/report should default outside the checkout.

Pinned chromiumoxide 0.9.1 config uses temp_dir()/chromiumoxide-runner when user_data_dir is absent. Our worker leaves it absent; project TMPDIR keeps that path on-volume but does not give invocations exclusive profiles. Browser Drop relies on child kill-on-drop and runtime background reaping, explicitly without a completion guarantee. The worker aborts its handler without joining it; browser fixture origin/child waits also lack complete bounded ownership. These are inspected source contracts, not a reproduced orphan or claim that Drop never kills the child. Owner .11.4.3.1.5 will reproduce and qualify the actual lifecycle.

## Artifact pressure and retention

A metadata-only target walk completed in 22.044 seconds and skipped zero links/other devices in that walk. It found 11,284 incremental .bin files totalling 53,267,691,753 bytes; 7,994 / 34,291,611,338 bytes were older than 24 hours. Filename counts independently partition the total: query-cache.bin, dep-graph.bin and work-products.bin each 3,749; dep-graph.part.bin 37. The old subset is 2,654 each plus 32. Other target .log files total 638 / 5,750,863 bytes, with 526 / 4,939,559 bytes older than 24 hours. No .bin/.log entries appeared under the separately classified release/deps trees; this says nothing about other artifact types. A subsequent available-space observation was 3,320,137,674,752 bytes. This is not a space emergency.

No artifact was deleted. Existing target/publisher-tests/pub-83115, old fixed CLI fixture directories and historical logs are retained as ambiguous/evidence-bearing residue. The restore-exercise directory was empty. Candidate age does not authorize deletion. Owner .11.4.3.1.6 must establish exact compiler-cache ownership, inactivity and evidence independence before selective cleanup, then prove residue and regeneration.

## Concrete follow-through

- .11.4.3.1.3: workflow locality, script coverage and explicit worker/demo prerequisites.
- .11.4.3.1.4: publisher fixture exclusive creation and checked cleanup; preserve pub-83115.
- .11.4.3.1.5: browser profile/process/fixture lifetime, coordinated with existing .7.2/.11.2 scope.
- .11.4.3.1.6: evidence-based compiler artifact cleanup or justified retention.
- .11.4.3.1.2: all required local gates, consumed results, authorized push and remote outcome; then return to bounded CLI transport .3.3.4.3.3.3.3.2.3.2.

The current CI guide's 31-server-suite count, Codex-only ignored-provider wording, implied working-tree secret scan and dormant dependency-policy description are corrected with this inventory. No gate policy, product code or workflow behavior changes in REPAIR-0033. Corrective review and release qualifications remain open.

## Source identity and recovery

Recover a source exactly with `git show 6bc76c6:<root-relative-path>`. The original source records are:

| Source | Bytes | SHA-256 |
| --- | --- | --- |
| Makefile | 3921 | 649766f4da1779c81d1f10f41638af44d406262ac46facfc4a5f609e0e81279f |
| docs/ci.md | 4494 | d0147c55d0a144934c799ef178e846eeec8b835961733d68b023d192cf3d559b |
| .github/workflows/rust.yml | 2239 | 01f9b89cad6facf85f7591d54aeab01f86d9f762642f0d02a43856839425474f |
| .github/workflows/doctrines.yml | 534 | 0663be5f7e27fa5a7923495c666393ffaf721292da0a726bb7b2390b04008acc |
| .github/workflows/supply-chain.yml | 1354 | c2bd39e7209dfb7ee46dd541ee87c8725495195ac0f2e7191e28993d0752d13a |
| scripts/run_pg_tests.py | 17574 | a7b5f40f70896f500d792b53c80e86865ac12a4a9b14dc539494c819b2d120e7 |
| scripts/run_pg_tests.sh | 494 | 53da08fba825afb9cd07def9a8ec60462782c7ddd42f970e72a605d05c5fca7b |
| scripts/project_env.py | 10881 | 996ce9df5f4352400ed80c1d5b495d36b21cf86d1310fe3481fd10945f30164e |
| deny.toml | 4693 | c53bdb4e712213ab61cae32307b9d1d6443d4535d0c4ec384b9db7a2eb7f29e4 |
| Cargo.toml | 502 | ba4fb11fa331c7f3d8d787b8e4693900deb12e7e2f69e606431c2562bec0d870 |
| Cargo.lock | 127530 | d8ce561e95c0ba9089e6d21b808b6f06ece078b95e6513adcbf6d62d1ebfb132 |
| rust-toolchain.toml | 66 | 07487db9d6b45de2f308f9fb578c9c46041381c3ee44ef782eca64150a394772 |

Verification: final wrapped Python source/dimensional probe returned zero: all twelve source blobs match recorded byte sizes/SHA-256; twelve altered-content controls differ; repeated locked offline metadata reproduces 12/86/38/40; exact missing/extra registration controls detect their changed sets. Independent filename partitions reproduce 11,284 and 7,994 candidates. A second wrapped probe returned zero: parsed old/current deny.toml values are identical, all six checkpoint owners exist, nine expected markers are present in rendered deployment.html, and unchanged README remains 52 lines / 2,054 bytes. make book and git diff --check returned zero. Logs/structured summaries: target/checkpoint-ci/inventory-verification.json and document-verification.json. These are inventory/document checks, not runtime gate passes. Final staged doctrines run in the commit hook.

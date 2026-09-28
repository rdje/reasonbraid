# Makefile — standard commands. `make gate` = the doctrine enforcer; `make check` = Rust.
SHELL := /usr/bin/env bash
PROJECT_RUN := python3 -B scripts/project_env.py

.PHONY: help gate check fmt clippy test deny secret-scan book demo dev showcase release hooks bootstrap update-scaffold

help:
	@echo "make gate            - run the doctrine enforcer (scripts/check_doctrines.sh)"
	@echo "make check           - format + strict lint + make test (pinned browser)"
	@echo "make fmt             - cargo fmt --all"
	@echo "make clippy          - cargo clippy --all-targets -- -D warnings"
	@echo "make test            - build workers + locked tests with pinned browser"
	@echo "make deny            - cargo deny check: advisories/bans/licenses/sources (requires cargo-deny)"
	@echo "make secret-scan     - gitleaks detect (secret scan; requires gitleaks)"
	@echo "make book            - build the mdBook (requires mdbook)"
	@echo "make demo            - the two-host crash/reconnect demo (ephemeral PG + evidence bundle)"
	@echo "make dev             - one-command dev environment: ephemeral PG + rb-server (Ctrl-C cleans up)"
	@echo "make showcase        - try it live: two answering agents + a local page (ask, progress, feedback)"
	@echo "make release         - the four release binaries (target/release/{rb,rb-server,rb-node,rb-journal})"
	@echo "make hooks           - install the git hooks (core.hooksPath=.githooks)"
	@echo "make bootstrap       - first-time project bootstrap"
	@echo "make update-scaffold - pull the latest ReasonBraid spine (set URL=<reasonbraid-repo>)"

gate:
	$(PROJECT_RUN) scripts/check_doctrines.sh

check:
	$(PROJECT_RUN) cargo fmt --all -- --check
	$(PROJECT_RUN) cargo clippy --all-targets --all-features -- -D warnings
	$(MAKE) test

fmt:
	$(PROJECT_RUN) cargo fmt --all

clippy:
	$(PROJECT_RUN) cargo clippy --all-targets --all-features -- -D warnings

test:
	$(PROJECT_RUN) cargo build --workspace --bins --locked
# Build the test targets BEFORE entering the browser harness: its deadline should
# bound test EXECUTION, not a compile. Measured at SIGNOFF-REPAIR.11.4.8 — a cold
# tree spent so long compiling inside the harness that the run was cut off at 78 of
# 94 binaries, while the same work on a warm tree executes in 1,817 s.
	$(PROJECT_RUN) cargo test --all --locked --no-run
	$(PROJECT_RUN) python3 -B scripts/ci_browser.py --timeout 5400 -- cargo test --all --locked --no-fail-fast

# Supply-chain checks (wired into .github/workflows/supply-chain.yml — see docs/ci.md).
deny:
	$(PROJECT_RUN) cargo deny check

secret-scan:
	$(PROJECT_RUN) gitleaks detect --source . --redact

book:
	$(PROJECT_RUN) mdbook build docs/book

# The WP6 two-host demonstration (.6.2): the full crash/reconnect scenario with
# real kill points and a grep-verified acceptance bundle. Runs the server suites +
# CLI e2e first; the evidence lands under target/demo/<run-id>/.
demo:
	RB_DEMO=1 $(PROJECT_RUN) bash scripts/run_pg_tests.sh

# Try it live (SHOWCASE.1, the director's request): a disposable system with two
# answering agents and a local page to ask, watch progress and leave feedback.
# Ctrl-C tears everything down. `python3 -B scripts/showcase.py status` needs no server.
showcase:
	python3 -B scripts/showcase.py up

# The one-command development environment (PHASE-1.7.1): ephemeral on-volume
# PostgreSQL + rb-server in the foreground; Ctrl-C tears everything down.
# `bash scripts/dev.sh --check` is the self-verification beat.
dev:
	$(PROJECT_RUN) scripts/dev.sh

# The local/LAN deployment package (PHASE-1.7.2): the four self-contained
# binaries (migrations + the console embed at compile time). The LAN runbook
# is deploy/README.md; the release-built proof is
# `bash scripts/demo_two_host.sh --database-url ... --release`.
# The `.2.3` signing step (ADR-027): the per-binary digest manifest + the
# Ed25519 signature — the release identity key generates on first use (the
# dev placement: the releaser's local file, gitignored).
#
# ⛔ THE SIGNED SET IS DERIVED, NOT LISTED (`SIGNOFF-REPAIR.6.8.1`). This target
# used to carry `--bin rb --bin rb-server --bin rb-node --bin rb-journal` — FOUR
# — while the line above it built TEN, and nothing could notice: the six it
# omitted landed in the same directory and were named by no manifest. Two of
# them are the acquisition workers `rb-server` SPAWNS, and
# `extraction::worker_path` resolves those from `current_exe().parent()` — this
# very directory. The flags now come from `.doctrine/release_binaries.tsv`,
# which carries every binary with a disposition and a reason, and
# `scripts/census_release_binaries.py --check` is the registered gate that
# refuses a workspace binary the ledger does not adjudicate.
#
# The census EMITS the flags rather than make parsing the ledger itself: one
# parser, asked by name. The first attempt had make run its own awk over the
# TSV and failed immediately for a reason worth keeping — make strips `\043` as
# a comment inside `$(shell ...)`, so the awk program's own skip-comments rule
# truncated the call.
# ⛔ `=` and NOT `:=` (`SIGNOFF-REPAIR.6.8.1.1`). A simply-expanded `:=` is
# evaluated when the Makefile is PARSED, so every `make` invocation — `gate`,
# `check`, `hooks` — spawned this census, and a broken or missing script
# printed an error on targets that have nothing to do with releasing.
# Recursive expansion evaluates it only where it is used, which is once, in
# the recipe below.
RELEASE_BIN_FLAGS = $(shell python3 -B scripts/census_release_binaries.py --release-flags)

release:
	$(PROJECT_RUN) cargo build --release --bins
	@ls -l target/release/rb target/release/rb-server target/release/rb-node target/release/rb-journal
	@$(PROJECT_RUN) bash -c 'test -f release-key.pk8 || ./target/release/rb-release-manifest keygen'
	$(PROJECT_RUN) ./target/release/rb-release-manifest generate --bin-dir target/release \
		$(RELEASE_BIN_FLAGS) \
		--out target/release/release-manifest.json
	$(PROJECT_RUN) ./target/release/rb-release-manifest verify --bin-dir target/release \
		--manifest target/release/release-manifest.json \
		--sig target/release/release-manifest.json.sig
# The identity a verifier needs (`SIGNOFF-REPAIR.11.24.1.4.1`). Until this step
# existed, `verify` derived the public key from the PRIVATE one, so the only
# party who could check a release was the party who signed it. The export is
# skipped when the file is already there, because `pubkey` refuses to overwrite
# a published identity; a RE-KEY therefore writes a new file under a new name,
# which is what docs/runbooks/signing-key-incident.md says to do.
	@$(PROJECT_RUN) bash -c 'test -f target/release/release-identity.pub || \
		./target/release/rb-release-manifest pubkey \
		--out target/release/release-identity.pub'

hooks:
	$(PROJECT_RUN) git config core.hooksPath .githooks
	@echo "git hooks activated (core.hooksPath=.githooks)"

bootstrap:
	$(PROJECT_RUN) scripts/bootstrap.sh

update-scaffold:
	$(PROJECT_RUN) scripts/update_scaffold.sh $(URL)

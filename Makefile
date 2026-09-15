# Makefile — standard commands. `make gate` = the doctrine enforcer; `make check` = Rust.
SHELL := /usr/bin/env bash
PROJECT_RUN := python3 -B scripts/project_env.py

.PHONY: help gate check fmt clippy test deny secret-scan book demo dev release hooks bootstrap update-scaffold

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
release:
	$(PROJECT_RUN) cargo build --release --bins
	@ls -l target/release/rb target/release/rb-server target/release/rb-node target/release/rb-journal
	@$(PROJECT_RUN) bash -c 'test -f release-key.pk8 || ./target/release/rb-release-manifest keygen'
	$(PROJECT_RUN) ./target/release/rb-release-manifest generate --bin-dir target/release \
		--bin rb --bin rb-server --bin rb-node --bin rb-journal \
		--out target/release/release-manifest.json
	$(PROJECT_RUN) ./target/release/rb-release-manifest verify --bin-dir target/release \
		--manifest target/release/release-manifest.json \
		--sig target/release/release-manifest.json.sig

hooks:
	$(PROJECT_RUN) git config core.hooksPath .githooks
	@echo "git hooks activated (core.hooksPath=.githooks)"

bootstrap:
	$(PROJECT_RUN) scripts/bootstrap.sh

update-scaffold:
	$(PROJECT_RUN) scripts/update_scaffold.sh $(URL)

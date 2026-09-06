# Makefile — standard commands. `make gate` = the doctrine enforcer; `make check` = Rust.
SHELL := /usr/bin/env bash

.PHONY: help gate check fmt clippy test deny secret-scan book demo dev hooks bootstrap update-scaffold

help:
	@echo "make gate            - run the doctrine enforcer (scripts/check_doctrines.sh)"
	@echo "make check           - cargo fmt --check + clippy (deny warnings) + test"
	@echo "make fmt             - cargo fmt --all"
	@echo "make clippy          - cargo clippy --all-targets -- -D warnings"
	@echo "make test            - cargo test --all"
	@echo "make deny            - cargo deny check: advisories/bans/licenses/sources (requires cargo-deny)"
	@echo "make secret-scan     - gitleaks detect (secret scan; requires gitleaks)"
	@echo "make book            - build the mdBook (requires mdbook)"
	@echo "make demo            - the two-host crash/reconnect demo (ephemeral PG + evidence bundle)"
	@echo "make dev             - one-command dev environment: ephemeral PG + rb-server (Ctrl-C cleans up)"
	@echo "make hooks           - install the git hooks (core.hooksPath=.githooks)"
	@echo "make bootstrap       - first-time project bootstrap"
	@echo "make update-scaffold - pull the latest ReasonBraid spine (set URL=<reasonbraid-repo>)"

gate:
	scripts/check_doctrines.sh

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all

# Supply-chain checks (wired into .github/workflows/supply-chain.yml — see docs/ci.md).
deny:
	cargo deny check

secret-scan:
	gitleaks detect --source . --redact

book:
	mdbook build docs/book

# The WP6 two-host demonstration (.6.2): the full crash/reconnect scenario with
# real kill points and a grep-verified acceptance bundle. Runs the server suites +
# CLI e2e first; the evidence lands under target/demo/<run-id>/.
demo:
	RB_DEMO=1 bash scripts/run_pg_tests.sh

# The one-command development environment (PHASE-1.7.1): ephemeral on-volume
# PostgreSQL + rb-server in the foreground; Ctrl-C tears everything down.
# `bash scripts/dev.sh --check` is the self-verification beat.
dev:
	scripts/dev.sh

hooks:
	git config core.hooksPath .githooks
	@echo "git hooks activated (core.hooksPath=.githooks)"

bootstrap:
	scripts/bootstrap.sh

update-scaffold:
	scripts/update_scaffold.sh $(URL)

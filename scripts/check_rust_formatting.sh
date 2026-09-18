#!/usr/bin/env bash
# scripts/check_rust_formatting.sh — RUST-FORMATTING doctrine.
#
# `cargo fmt --all -- --check` must pass. Nothing ran it per commit, and
# unformatted Rust reached `main` (`SIGNOFF-REPAIR.11.21`).
#
# THE MEASURED DEFECT. `crates/reasonbraid-server/src/git.rs` sat on `main`
# failing this check at two sites — an over-split `||` chain and an un-wrapped
# `assert!` — introduced by REPAIR-0251/0252 and found only because a LATER leaf
# happened to run the check while verifying a file it had not touched.
#
# It is a policy gap rather than a lapse. `CLAUDE.md` §16 routes the full CI to
# pre-push and the per-commit gate to `scripts/check_doctrines.sh`, whose 18
# registered checks contained no formatting check; `cargo fmt --check` appeared
# only in `COMMIT.md`'s pre-push list. Since pushes go out in batches of ~300
# commits, unformatted code could sit on `main` for as many commits as that, and
# here it did.
#
# WHY IT BELONGS IN THE PER-COMMIT GATE, measured before proposing it
# (`SIGNOFF-REPAIR.11.6`: no rule before its population). Three runs each, this
# machine, warm:
#
#     cargo fmt --all -- --check    0.81s, 0.66s, 0.66s
#     scripts/check_doctrines.sh    12.57s, 12.52s, 13.00s
#
# So it adds about 5% to a gate that already costs twelve seconds. `.11.5`'s
# constraint is that a gate people route around is a gate that lies, and nobody
# routes around two-thirds of a second. ⛔ The rejected alternative is leaving it
# in the pre-push list: that is where it already was, and it is the arrangement
# that let the defect ship — a check that runs once per ~300 commits cannot say
# WHICH commit broke formatting, so the cost of finding out is the thing being
# economised on.
#
# ⚠️ WHOLE-WORKSPACE, NOT STAGED-SCOPE, and deliberately. The defect was a file
# NO staged change touched; a staged-scope check would not have found it and
# would not find the next one. This is affordable only because the workspace is
# formatted as of REPAIR-0256 — a strict check over pre-existing debt would have
# had to be a ratchet, like TABLE-ARITY.
#
# ⛔ A MISSING TOOLCHAIN IS ANNOUNCED, NOT PASSED OVER. `rustfmt` absent means
# this proves nothing, and a silent skip looks exactly like a pass.
#
# CONTRACT: exit code is the verdict; explains on stderr; deterministic;
# read-only (never rewrites a file); fast; path-agnostic.
#
# Self-test: scripts/check_rust_formatting.sh --self-test
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# --- the decision, as a pure function so it can carry ground truth ---
# $1 = rustfmt/cargo-fmt exit status. Echoes: ok | unformatted
formatting_verdict() {
    [ "$1" -eq 0 ] && echo "ok" || echo "unformatted"
}

if [ "${1:-}" = "--self-test" ]; then
    fails=0

    # The pure verdict, both sides.
    [ "$(formatting_verdict 0)" = "ok" ] || { echo "SELF-TEST: rc=0 is not ok" >&2; fails=$((fails+1)); }
    [ "$(formatting_verdict 1)" = "unformatted" ] || { echo "SELF-TEST: rc=1 is not a refusal" >&2; fails=$((fails+1)); }

    # And the real tool, against real files — the half a pure function cannot
    # reach. §13: the scratch lives on the repository volume, never /tmp.
    mkdir -p "$ROOT/target/doctrine_scratch"
    scratch="$(mktemp -d "$ROOT/target/doctrine_scratch/fmt-gate.XXXXXX")"
    trap 'rm -rf "$scratch"' EXIT

    if ! command -v rustfmt >/dev/null 2>&1; then
        echo "SELF-TEST: rustfmt is absent, so neither real arm ran — this gate is UNPROVEN here" >&2
        fails=$((fails+1))
    else
        # A deliberately unformatted file must be REFUSED...
        printf 'fn main() {\n      let  x=1 ;\n  println!("{}",x)\n}\n' > "$scratch/bad.rs"
        rustfmt --check --edition 2021 "$scratch/bad.rs" >/dev/null 2>&1
        if [ "$(formatting_verdict $?)" != "unformatted" ]; then
            echo "SELF-TEST: an unformatted file was accepted" >&2; fails=$((fails+1))
        fi
        # ...and rustfmt's OWN output of that same file must be accepted, so the
        # arm above cannot be passing because the tool refuses everything.
        cp "$scratch/bad.rs" "$scratch/good.rs"
        rustfmt --edition 2021 "$scratch/good.rs" >/dev/null 2>&1
        rustfmt --check --edition 2021 "$scratch/good.rs" >/dev/null 2>&1
        if [ "$(formatting_verdict $?)" != "ok" ]; then
            echo "SELF-TEST: a formatted file was refused" >&2; fails=$((fails+1))
        fi
        if cmp -s "$scratch/bad.rs" "$scratch/good.rs"; then
            echo "SELF-TEST: rustfmt changed nothing, so the refusal arm proves nothing" >&2
            fails=$((fails+1))
        fi
    fi

    [ "$fails" -eq 0 ] || exit 1
    echo "RUST-FORMATTING self-test: both verdict sides, a real unformatted file refused, its formatted form accepted, and the two shown to differ"
    exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "RUST-FORMATTING: cargo is absent — NOT CHECKED (this is not a pass)" >&2
    exit 0
fi

out="$(cargo fmt --all -- --check 2>&1)"
if [ "$(formatting_verdict $?)" = "ok" ]; then
    exit 0
fi

# `Diff in <abs path>:<line>:` — drop the trailing `:<line>:` and make the path
# repository-root-relative (§12: no checkout-specific absolute paths in output
# a reader is meant to act on).
echo "RUST-FORMATTING: the workspace is not formatted. Files:" >&2
printf '%s\n' "$out" \
    | sed -n 's/^Diff in \(.*\):[0-9][0-9]*:*$/\1/p' \
    | sed "s|^$ROOT/||" \
    | sort -u | sed 's/^/    /' >&2
cat >&2 <<'GUIDE'

  Run `cargo fmt --all` (through scripts/project_env.py) and stage the result.

  This runs per commit rather than only before a push because unformatted code
  reached `main` and sat there: pushes go out in batches of ~300 commits, so a
  pre-push-only check cannot say which commit broke formatting
  (SIGNOFF-REPAIR.11.21). It costs about 0.7s.
GUIDE
exit 1

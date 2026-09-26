#!/usr/bin/env bash
# scripts/lib/demo_checks.sh — the demonstration's check shapes that must not
# pass on a failed command (SIGNOFF-REPAIR.11.3.7). Sourced by
# scripts/demo_two_host.sh; exercised directly by scripts/tests/test_demo_checks.py.

# absent <pattern> <command...> — the command SUCCEEDS with output, and its
# output does not match <pattern>. The shape it replaces, a negated pipeline
# ("! cmd | grep -q pattern"), passed whenever cmd failed: no output, so no
# match, so the negation held — a broken server satisfied every "this did NOT
# happen" check.
absent() {
    local pattern="$1" output
    shift
    output="$("$@")" || return 1
    [ -n "$output" ] || return 1
    ! printf '%s\n' "$output" | grep -q -- "$pattern"
}

# fetch <file> <curl arguments...> — the response body into <file>, kept as
# evidence whatever it is, and success only for a 2xx status. The captures it
# replaces kept a failed request's body with no word that it had failed.
# ⛔ The arguments are never echoed: a probe's body can carry a fencing token.
fetch() {
    local file="$1" status
    shift
    status="$(curl -sS --connect-timeout 5 --max-time 30 -o "$file" -w '%{http_code}' "$@")" || return 1
    case "$status" in
        2??) return 0 ;;
        *) echo "fetch: HTTP $status (the body is kept in $file)" >&2; return 1 ;;
    esac
}

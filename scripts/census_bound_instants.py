#!/usr/bin/env python3
"""Census every timestamp column an INSERT or UPDATE fills from a RUST-SIDE
instant rather than from the database, and say which of them are read back
(`SIGNOFF-REPAIR.11.31`).

🔴 THE HAZARD IS KNOWN, WRITTEN DOWN TWICE, AND STILL SHIPPED AGAIN.
`site_authority/mod.rs` carries *"PostgreSQL stores microseconds. Normalize
before subset comparisons so an identical parent/child scope does not fail on
discarded nanoseconds"*, and the book documents it for readers. The budget path
then had the identical hazard with no mitigation and a comment asserting the
invariant it did not hold: a reservation proof carried a Rust nanosecond the
`TIMESTAMPTZ` column had truncated, so the ledger re-lent capacity up to 999 ns
before the proof stopped being honoured (`SIGNOFF-REPAIR.11.30`).

⛔ SO THIS IS AN UNENFORCED RULE RATHER THAN AN OVERSIGHT, and `.11.31` forbids
opening with a gate: *a check written before the population is known is a check
fitted to the two examples in hand*. This counts the population. It proposes
nothing.

⚠️ AND THE LOCAL GATES ARE STRUCTURALLY BLIND TO IT. This host's `CLOCK_REALTIME`
is microsecond-granular — 20,000 consecutive samples, zero sub-microsecond digits
(`.11.30`) — so the defect cannot occur here and is visible only on the Linux
runner. A census over the SOURCE is the only instrument that sees it from this
machine, which is why the population is counted rather than reproduced.

⭐ THE MAPPING IS EXACT, NOT HEURISTIC, and that is the design decision worth
defending. `sqlx` binds positionally, so a column's `$N` placeholder names the
Nth `.bind(…)` in the chain — the census resolves
`column -> placeholder -> bound expression` and reports the expression's own
text. A census that merely noticed *this statement mentions a time column and
also binds things* would report a correct `now()` site and a defective one
identically.

The three verdicts, per (statement, timestamp column):

  `db_generated` — filled by `now()`, `clock_timestamp()` or `CURRENT_TIMESTAMP`.
                   The database writes and truncates one value, so there is no
                   second copy to disagree with it. The negative class.
  `read_back`    — bound from Rust AND named in a `RETURNING` clause. This is
                   `.11.30`'s remedy: the caller gets the STORED value.
  `bound_unread` — bound from Rust and not read back. ⚠️ A CANDIDATE, NOT A
                   DEFECT: it is only a defect when that Rust value also escapes
                   to a caller or is compared against the stored column, which is
                   a judgement over the surrounding code and is NOT mechanized
                   here.

    python3 -B scripts/census_bound_instants.py
    python3 -B scripts/census_bound_instants.py --check
    python3 -B scripts/census_bound_instants.py --candidates
    python3 -B scripts/census_bound_instants.py --self-test

Self-test: scripts/census_bound_instants.py --self-test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

QUERY = re.compile(r"sqlx::query(?:_as|_scalar)?(?:::<[^>]*>)?\s*\(", re.S)

# ⭐ THE COLUMN TYPE IS DERIVED FROM THE SCHEMA, NOT GUESSED FROM THE NAME, and
# the first version of this census is why. A name pattern reported
# `issued_by <- &intent.actor`, `issued_under <- &record_id` and
# `from_region <- from.as_str()` as timestamp columns — 4 of 44 candidates that
# are TEXT, matched on `issued` and `from`. The migrations declare every column's
# type and are tracked in this repository, so the authority is available and
# there is no reason to approximate it.
TIMESTAMP_TYPE = re.compile(r"(?i)^timestamptz\b|^timestamp\b|^time\b|^date\b")
# Used only where the schema is silent — for a fixture, or a table the migration
# parser could not read. It is the OLD classifier, kept as a fallback and never
# as the primary, so its 4-in-44 error rate cannot come back by default.
NAME_FALLBACK = re.compile(
    r"(?i)(^|_)(at|from|to|time|timestamp|deadline|expires|expiry|issued|decided|"
    r"created|updated|revoked|acknowledged|offered|settled|checked|started|ended)(_|$)")
CREATE_TABLE = re.compile(r"(?is)create\s+table\s+(?:if\s+not\s+exists\s+)?([\w.]+)\s*\((.*?)\n\s*\)\s*;")
# ⛔ ONE `ALTER TABLE` CAN CARRY SEVERAL `ADD COLUMN` CLAUSES, comma-separated,
# and the first version of this matched only the first of them. `migrations/0031`
# adds `license`, `fresh_until` and `refreshed_at` in one statement, so two
# TIMESTAMPTZ columns were invisible and their write sites were reported as
# "absent from the schema". The statement is matched whole and every clause
# inside it is read.
ALTER_TABLE = re.compile(r"(?is)alter\s+table\s+(?:if\s+exists\s+)?([\w.]+)(.*?);")
ADD_COLUMN = re.compile(r"(?is)add\s+column\s+(?:if\s+not\s+exists\s+)?(\w+)\s+([\w ]+)")

# A file that drives SQLite writes to the node's own journal, whose schema lives
# in Rust source rather than in `migrations/`. ⛔ DERIVED FROM THE DRIVER THE FILE
# USES, not from its crate name: the hazard this census is about is PostgreSQL
# TIMESTAMPTZ truncating to microseconds, and a different store is a different
# question rather than a gap in this one.
SQLITE_DRIVER = re.compile(r"sqlx::sqlite|SqlitePool|SqliteConnectOptions")

DB_TIME = re.compile(r"(?i)\b(now|clock_timestamp|current_timestamp|statement_timestamp)\b")


def bare(table: str) -> str:
    """`public.site_audit` and `site_audit` are the same table to this census."""
    return table.split(".")[-1].lower()


def schema_types(migrations: list[str]) -> dict[tuple[str, str], str]:
    """Every (table, column) the migrations declare, mapped to its declared type.

    ⛔ READ FROM THE MIGRATIONS, which are the authority, rather than inferred.
    A column this returns nothing for is reported as `unknown_column` instead of
    being silently dropped: an absence from the schema is a fact about this
    parser, not about the database (`docs/knowledge/an-instruments-zero-describes-its-reach.md`).
    """
    out: dict[tuple[str, str], str] = {}
    for text in migrations:
        for table, body in CREATE_TABLE.findall(text):
            depth = 0
            current: list[str] = []
            for piece in body.split("\n"):
                line = piece.split("--")[0].strip().rstrip(",")
                if not line:
                    continue
                parts = line.split()
                if not parts:
                    continue
                name = parts[0].strip('"')
                if name.upper() in ("PRIMARY", "FOREIGN", "UNIQUE", "CHECK", "CONSTRAINT",
                                    "EXCLUDE", "LIKE"):
                    continue
                rest = " ".join(parts[1:])
                out[(bare(table), name.lower())] = rest
            del depth, current
        for table, body in ALTER_TABLE.findall(text):
            for column, coltype in ADD_COLUMN.findall(body):
                out[(bare(table), column.lower())] = coltype.strip()
    return out


@dataclass(frozen=True)
class Column:
    statement: str      # "insert" | "update"
    table: str
    name: str
    value: str          # the SQL expression filling it
    bound: str | None   # the Rust expression, when the value is a placeholder
    verdict: str
    fn: str = ""        # the enclosing function, when one was found
    escape: str = ""    # "", "reaches_return", "contained" — the triage only
    origin: str = ""    # where the bound instant came from


def unescape_rust_string(raw: str) -> str:
    """The literal's text, with Rust's line-continuation applied.

    ⛔ THIS IS NOT COSMETIC. This repository writes long SQL with a trailing
    backslash, which in Rust swallows the newline AND the following indentation.
    Leaving it in splices the last word of one line onto the first of the next
    (`created_at) \\\\\\n  VALUES` becomes `created_at) VALUES` only after the
    continuation is applied), and every column list would be parsed wrong.
    """
    if raw.startswith("r"):
        body = raw[raw.index('"') + 1:raw.rindex('"')]
        return body
    body = raw[1:-1]
    body = re.sub(r"\\\n\s*", "", body)
    return body.replace('\\"', '"').replace("\\\\", "\\")


def extract_literal(text: str, start: int) -> tuple[str, int] | None:
    """The string literal beginning at or after `start`, and the index after it."""
    i = start
    while i < len(text) and text[i] in " \t\r\n":
        i += 1
    if i >= len(text):
        return None
    if text.startswith("r#", i) or text.startswith('r"', i):
        m = re.compile(r'r(#*)"').match(text, i)
        if not m:
            return None
        close = '"' + m.group(1)
        end = text.find(close, m.end())
        if end == -1:
            return None
        return text[i:end + len(close)], end + len(close)
    if text[i] != '"':
        return None
    j = i + 1
    while j < len(text):
        if text[j] == "\\":
            j += 2
            continue
        if text[j] == '"':
            return text[i:j + 1], j + 1
        j += 1
    return None


def bind_chain(text: str, start: int) -> list[str]:
    """The `.bind(…)` arguments after a query call, in order.

    Parenthesis-balanced, because a bound expression is routinely a call of its
    own — `serde_json::to_value(requested).expect("dims serialize")` — and a
    comma- or paren-naive split tears it in half.
    """
    args: list[str] = []
    i = start
    # ⛔ The chain must be CONTIGUOUS: each `.bind(` has to follow immediately
    # (whitespace and comments aside) from where the previous one ended. Searching
    # forward instead would walk past `.execute(&mut *tx)` into the NEXT query's
    # binds and shift every placeholder index after it.
    # ⚠️ NO `\A` HERE. `re.match(text, pos)` anchors at `pos`, but `\A` anchors at
    # the start of the STRING and ignores it, so the first version matched nothing
    # and every bound expression read `(no matching bind)`.
    step = re.compile(r"(?:\s|//[^\n]*\n)*\.\s*bind\s*\(")
    while True:
        m = step.match(text, i)
        if not m:
            break
        depth, j = 1, m.end()
        while j < len(text) and depth:
            if text[j] == "(":
                depth += 1
            elif text[j] == ")":
                depth -= 1
            elif text[j] == '"':
                lit = extract_literal(text, j)
                if lit:
                    j = lit[1]
                    continue
            j += 1
        args.append(" ".join(text[m.end():j - 1].split()))
        i = j
    return args


def split_top_level(s: str) -> list[str]:
    """Split on commas that are not inside parentheses."""
    out, depth, cur = [], 0, []
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def analyse(sql: str, binds: list[str],
            types: dict[tuple[str, str], str] | None = None) -> list[Column]:
    """Every timestamp column this statement fills, with how it is filled.

    `types` is the schema. With it, a column is a timestamp because the migration
    SAYS SO; without it the name pattern stands in, which is what the self-test
    fixtures use and what a table this parser could not read falls back to.
    """
    flat = " ".join(sql.split())
    returning = ""
    m = re.search(r"(?i)\breturning\b(.*)$", flat)
    if m:
        returning = m.group(1)
        flat = flat[:m.start()]

    pairs: list[tuple[str, str]] = []
    statement = table = ""
    # ⛔ THE VALUES LIST IS TAKEN BY BALANCED PARENTHESES, not by a greedy regex.
    # `VALUES ($1, …, $5) ON CONFLICT (event_id) DO NOTHING` made a greedy `\((.*)\)`
    # swallow the conflict clause, so the last column's value read
    # `$5) ON CONFLICT (event_id` and the site was reported unresolvable — a
    # parser limit presented as a finding about the code.
    ins = re.search(r"(?i)\binsert\s+into\s+([\w.]+)\s*\((.*?)\)\s*values\s*\(", flat)
    if ins:
        statement, table = "insert", ins.group(1)
        depth, k = 1, ins.end()
        while k < len(flat) and depth:
            if flat[k] == "(":
                depth += 1
            elif flat[k] == ")":
                depth -= 1
            k += 1
        cols = split_top_level(ins.group(2))
        vals = split_top_level(flat[ins.end():k - 1])
        pairs = list(zip(cols, vals))
    else:
        upd = re.search(r"(?i)\bupdate\s+([\w.]+)\s+set\s+(.*?)(?:\bwhere\b|$)", flat)
        if upd:
            statement, table = "update", upd.group(1)
            for assign in split_top_level(upd.group(2)):
                if "=" in assign:
                    left, right = assign.split("=", 1)
                    pairs.append((left.strip(), right.strip()))
    if not pairs:
        return []

    out: list[Column] = []
    for name, value in pairs:
        name = name.strip().strip('"')
        key = (bare(table), name.lower())
        if types is None:
            if not NAME_FALLBACK.search(name):
                continue
        elif key in types:
            if not TIMESTAMP_TYPE.match(types[key]):
                continue
        elif NAME_FALLBACK.search(name):
            # ⚠️ The schema does not declare this column, so the census cannot
            # say what it is. Reported rather than dropped: an absence here is a
            # fact about this parser's reach, not about the database.
            out.append(Column(statement, table, name, value.strip(), None, "unknown_column"))
            continue
        else:
            continue
        bound = None
        if DB_TIME.search(value):
            verdict = "db_generated"
        else:
            ph = re.fullmatch(r"\$(\d+)(::\w+)?", value.strip())
            if not ph:
                verdict = "db_generated" if not value.strip().startswith("$") else "unresolved"
            else:
                idx = int(ph.group(1)) - 1
                bound = binds[idx] if 0 <= idx < len(binds) else "(no matching bind)"
                named = re.search(rf"(?i)\b{re.escape(name)}\b", returning) is not None
                verdict = "read_back" if named else "bound_unread"
        out.append(Column(statement, table, name, value.strip(), bound, verdict))
    return out


# ⭐ PROVENANCE IS THE DISCRIMINATOR THE LEAF'S FRAMING MISSED, and it is the one
# that decides. `.11.31` asks for sites that bind "a Rust-side instant", but an
# instant this process READ BACK from PostgreSQL is already truncated to
# microseconds — binding it and returning it creates no second opinion, because
# there is only ever one value. The hazard needs a SECOND clock, not merely a
# Rust variable.
DB_ORIGIN = re.compile(r"database_now_in_tx|database_now\s*\(|clock_timestamp")
RUST_CLOCK = re.compile(r"Utc::now\s*\(\)|SystemTime::now|Instant::now")
NORMALIZED = re.compile(r"\.checked\s*\(\)|from_timestamp_micros")

FN_HEAD = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)", re.M)
ROOT_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
RETURN_SHAPE = re.compile(r"\breturn\b|\bOk\s*\(|\bSome\s*\(")


def enclosing_fn(text: str, pos: int) -> tuple[str, int, int]:
    """(name, start, end) of the `fn` containing `pos`.

    ⛔ THE END IS FOUND BY BRACE BALANCE FROM THE SIGNATURE, not by the next
    `fn` keyword: a nested closure or an inner `fn` would end the span early and
    the escape triage would stop reading before the function's own return.
    """
    heads = [m for m in FN_HEAD.finditer(text) if m.start() <= pos]
    if not heads:
        return "", 0, len(text)
    head = heads[-1]
    brace = text.find("{", head.end())
    if brace == -1:
        return head.group(1), head.start(), len(text)
    depth, k = 0, brace
    while k < len(text):
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
            if depth == 0:
                return head.group(1), head.start(), k + 1
        k += 1
    return head.group(1), head.start(), len(text)


def escape_triage(text: str, pos: int, bound: str) -> tuple[str, str]:
    """Does the bound expression's root binding reach a return-shaped position?

    ⚠️ A TRIAGE, NOT A VERDICT, and the distinction is the whole point. It answers
    a decidable question — *does this identifier appear inside a `return`, `Ok(`
    or `Some(` after the write, within the same function* — which is neither
    sound nor complete for "the caller compares this against the stored column".
    It exists to ORDER the hand adjudication, and the adjudication is what
    decides. Publishing its output as the answer would be the error
    `docs/knowledge/a-census-is-as-wide-as-its-key.md` describes.

    🔴 AND ITS UNSOUNDNESS IS NOW MEASURED RATHER THAN DECLARED
    (`SIGNOFF-REPAIR.11.31.1.1`). `budget::create_reservation_in_tx` writes
    `budget_reservations.created_at` in its DENIED branch — a statement with no
    `RETURNING`, followed by `Err(BudgetError::Unavailable)` carrying no instant
    — and this function reports it as `reaches_return`, because the same
    function's ACTIVE branch ends in `Ok(Reservation { … })`. A match across two
    branches is exactly the case a one-function text scan cannot separate, and
    it produced a false positive on the one subsystem where the original defect
    lived. ⛔ The adjudication carries the verdict `triage_false_positive` for
    that site so the disagreement is recorded, not smoothed over.
    """
    name, start, end = enclosing_fn(text, pos)
    if not bound:
        return name, ""
    m = ROOT_IDENT.search(bound.lstrip("&*"))
    if not m:
        return name, ""
    root = m.group(0)
    after = text[pos:end]
    for line in after.splitlines():
        if RETURN_SHAPE.search(line) and re.search(rf"\b{re.escape(root)}\b", line):
            return name, "reaches_return"
    # a struct literal spanning lines: the field appears after an `Ok(`/`return`
    tail = after
    for m2 in RETURN_SHAPE.finditer(tail):
        if re.search(rf"\b{re.escape(root)}\b", tail[m2.start():m2.start() + 400]):
            return name, "reaches_return"
    return name, "contained"


def provenance(text: str, pos: int, bound: str) -> str:
    """Where the bound instant came from, within its own function.

    `database_clock` — assigned from `database_now_in_tx` / `tx.database_now()`
                       / `clock_timestamp()`. Already microsecond-truncated by
                       PostgreSQL, so there is no second clock and no hazard.
    `normalized`     — assigned through `.checked()` or `from_timestamp_micros`,
                       the mitigation `site_authority/mod.rs` documents.
    `rust_clock`     — `Utc::now()` and friends. The only origin that can carry
                       a nanosecond the column will discard.
    `caller_supplied`— a parameter: the origin is the caller's and this pass does
                       not follow it. ⚠️ Reported as its own class rather than
                       guessed, because guessing is how the name heuristic got
                       four columns wrong one measurement ago.
    `other`          — a local whose right-hand side matches none of the above.
    """
    b = bound.lstrip("&*")
    m = ROOT_IDENT.search(b)
    if not m:
        return "other"
    root = m.group(0)
    if RUST_CLOCK.search(b):
        return "rust_clock"
    name, start, end = enclosing_fn(text, pos)
    body = text[start:end]
    assign = re.search(rf"let\s+{re.escape(root)}\s*(?::[^=]+)?=\s*([^;]*);", body, re.S)
    if assign:
        rhs = assign.group(1)
        if DB_ORIGIN.search(rhs):
            return "database_clock"
        if NORMALIZED.search(rhs):
            return "normalized"
        if RUST_CLOCK.search(rhs):
            return "rust_clock"
        return "other"
    signature = body[:body.find("{")] if "{" in body else ""
    if re.search(rf"\b{re.escape(root)}\s*:", signature):
        return "caller_supplied"
    return "other"


def scan_file(text: str, types: dict[tuple[str, str], str] | None = None) -> list[Column]:
    other_store = bool(SQLITE_DRIVER.search(text))
    found: list[Column] = []
    for m in QUERY.finditer(text):
        lit = extract_literal(text, m.end())
        if not lit:
            continue
        raw, after = lit
        sql = unescape_rust_string(raw)
        # step over the closing paren of the query call
        close = text.find(")", after)
        if close == -1:
            continue
        for col in analyse(sql, bind_chain(text, close + 1), types):
            if other_store and col.verdict == "unknown_column":
                col = Column(col.statement, col.table, col.name, col.value,
                             col.bound, "other_store")
            if col.verdict == "bound_unread":
                fn, esc = escape_triage(text, m.start(), col.bound or "")
                col = Column(col.statement, col.table, col.name, col.value,
                             col.bound, col.verdict, fn, esc,
                             provenance(text, m.start(), col.bound or ""))
            found.append(col)
    return found


# ── the adjudication: a hand judgement over code paths, carried as data ───────
#
# ⛔ PROSE WAS NOT ENOUGH, and `SIGNOFF-REPAIR.11.31.1.1` is why. `.11.31.1`
# required this to be "carried as DATA and guarded, the way
# `census_route_controls.ADJUDICATION` is", and published sentences instead —
# which is how a claim about 24 sites came to be written from 8 code paths with
# nothing to notice. Each entry records the (origin, escape) the census computed
# WHEN the judgement was made, so a site whose inputs change refuses rather than
# keeping a verdict reached about different facts
# (`docs/knowledge/an-adjudication-is-keyed-to-the-words-it-judged.md`).
#
# ⚠️ THE KEY IS (file, function, table.column) AND TWO SITES CAN SHARE ONE.
# `profiles::write_profile_in_tx` writes `agent_profiles.updated_at` from TWO
# statements — the INSERT at crates/reasonbraid-server/src/profiles.rs:394 and the UPDATE at :425, the two
# halves of its upsert — so 43 sites map to 42 keys. `adjudication_drift` checks
# EVERY site against its key's recorded pair, so if the two ever diverge the
# census refuses instead of quietly judging one by the other's reading.
#   🔴 THIS COMMENT NAMED THE WRONG FUNCTION UNTIL `SIGNOFF-REPAIR.11.31.1.1.1`.
#   It said `snapshots::submit`/`evidence_snapshots.retrieved_at`, which has two
#   SITES in the census but two distinct KEYS (`submit` and `submit_external`),
#   so it never collapsed at all. The guard was unaffected — it checks sites, not
#   keys — but a warning that names the wrong instance sends the next reader to
#   the wrong file.
#
# Verdicts:
#   contained             — written, never read back; the caller's clock is
#                           irrelevant because nothing compares two copies.
#   database_clock        — escapes, but the value was read from the database.
#   normalized            — escapes, but was rounded to microseconds first.
#   value_granularity     — escapes, but cannot carry sub-microsecond precision.
#   input_echo            — escapes as an echo of the caller's own request.
#   triage_false_positive — the escape triage says it escapes and it does not.
VERDICTS = ("contained", "database_clock", "normalized", "value_granularity",
            "input_echo", "triage_false_positive")

ADJUDICATION: dict[tuple[str, str, str], tuple[str, str, str, str]] = {
    # ── six sites added after `.11.31.1` and judged at `SIGNOFF-REPAIR.11.31.2`
    #    (2026-09-25): this census's `--check` ran in no gate, so they arrived
    #    unjudged, two of them in the budget ledger. None escapes.
    ("reasonbraid-server/src/authority/federation_admin.rs",
     "propose_direction_in_one_transaction", "federation_agreements.expires_at"):
        ("caller_supplied", "contained", "contained",
         "the request's expiry is written and never read back into Rust; its one comparison, `IS DISTINCT FROM EXCLUDED.expires_at`, runs inside the database between two values already truncated to microseconds"),
    ("reasonbraid-server/src/authority/node_admin.rs",
     "adjudicate_ambiguous_attempt_in_one_transaction", "node_ambiguous_attempts.adjudicated_at"):
        ("database_clock", "reaches_return", "database_clock",
         "`at` is tx.database_now() in the same transaction"),
    ("reasonbraid-server/src/authority/node_admin.rs",
     "replay_command_with_possible_duplicate_in_one_transaction", "node_inbox.decided_at"):
        ("database_clock", "reaches_return", "database_clock",
         "`at` is tx.database_now() in the same transaction"),
    ("reasonbraid-server/src/authority/profile_admin.rs",
     "import_after_admission", "card_imports.imported_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back: the caller binds `at` from tx.database_now(), and CardImportResult carries no instant"),
    ("reasonbraid-server/src/budget.rs",
     "hold_for_unknown_outcome_in_tx", "budget_reservations.outcome_unknown_at"):
        ("caller_supplied", "contained", "contained",
         "bound from the caller's Utc::now() and never read back: the function returns only whether the hold was taken, and the column is otherwise consulted only for NULL"),
    ("reasonbraid-server/src/budget.rs",
     "settle_adjudicated_hold_in_tx", "budget_reservations.settled_at"):
        ("caller_supplied", "contained", "contained",
         "bound from the caller's Utc::now() and never read back: the function returns the reservation id alone, and the settlement it also writes is judged at settle_reservation_in_tx"),
    ("reasonbraid-server/src/api.rs",
     "apply_node_result_in_tx", "node_inbox.quarantined_at"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority.rs",
     "insert_boundary_in_tx", "enrollment_boundaries.expires_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority.rs",
     "insert_boundary_in_tx", "enrollment_boundaries.valid_from"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority.rs",
     "insert_grant_row", "authority_grants.expires_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority.rs",
     "insert_grant_row", "authority_grants.valid_from"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority.rs",
     "record_selection_in_tx", "authorization_records.decided_at"):
        ("caller_supplied", "reaches_return", "database_clock",
         "both callers bind `at` from transaction::database_now_in_tx, which is SELECT clock_timestamp()"),
    ("reasonbraid-server/src/authority/effects.rs",
     "record_administrative_effect_in_tx", "administrative_effects.effected_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority/federation_admin.rs",
     "accept_direction_in_one_transaction", "federation_agreements.accepted_at"):
        ("database_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority/node_admin.rs",
     "issue_enrollment_token_in_one_transaction", "node_enrollment_tokens.superseded_at"):
        ("database_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/authority/node_admin.rs",
     "quarantine_command_in_one_transaction", "node_inbox.quarantined_at"):
        ("database_clock", "reaches_return", "database_clock",
         "`at` is tx.database_now() in the same transaction"),
    ("reasonbraid-server/src/authority/node_admin.rs",
     "replay_command_in_one_transaction", "node_inbox.decided_at"):
        ("database_clock", "reaches_return", "database_clock",
         "`at` is tx.database_now() in the same transaction"),
    ("reasonbraid-server/src/budget.rs",
     "check_spend_breaker_in_tx", "spend_breakers.tripped_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/budget.rs",
     "create_reservation_in_tx", "budget_reservations.created_at"):
        ("caller_supplied", "reaches_return", "triage_false_positive",
         "the bound value is the DENIED branch, which executes and returns Err(BudgetError::Unavailable) carrying no instant; the Ok(Reservation) the triage matched belongs to the ACTIVE branch, built from RETURNING reservation_id, expires_at, created_at per SIGNOFF-REPAIR.11.30. api.rs:7113 does pass Utc::now() here and it still cannot escape"),
    ("reasonbraid-server/src/budget.rs",
     "overrun_of", "budget_reservations.settled_at"):
        ("other", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/budget.rs",
     "release_reservation", "budget_reservations.settled_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/corrections.rs",
     "record_correction", "policy_corrections.expires_at"):
        ("other", "reaches_return", "input_echo",
         "the response echoes input.expires_at, the caller's own request field, rather than asserting what the row holds"),
    ("reasonbraid-server/src/node_channel.rs",
     "enqueue_in_tx", "node_inbox.decided_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "enroll", "incarnations.valid_from"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "enroll", "incarnations.valid_to"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "enroll", "node_certificates.expires_at"):
        ("other", "reaches_return", "value_granularity",
         "leaf.not_after is built in ca.rs as from_timestamp(unix_timestamp(), 0) — whole seconds, nanosecond component literally zero"),
    ("reasonbraid-server/src/node_channel.rs",
     "enroll", "node_certificates.issued_at"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "enroll", "node_enrollment_tokens.used_at"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "issue_lease", "node_leases.issued_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "issue_lease", "node_leases.last_seen_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "record_event_in_tx", "node_events.received_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "renew_lease", "node_leases.last_seen_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/node_channel.rs",
     "rotate", "node_certificates.expires_at"):
        ("other", "reaches_return", "value_granularity",
         "leaf.not_after is built in ca.rs as from_timestamp(unix_timestamp(), 0) — whole seconds, nanosecond component literally zero"),
    ("reasonbraid-server/src/node_channel.rs",
     "rotate", "node_certificates.issued_at"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/profiles.rs",
     "write_profile_in_tx", "agent_profiles.updated_at"):
        ("caller_supplied", "reaches_return", "database_clock",
         "all three callers bind `at` from database_now_in_tx or tx.database_now(); attest_capability_in_tx inherits it from profile_admin.rs"),
    ("reasonbraid-server/src/profiles.rs",
     "write_profile_in_tx", "profile_versions.written_at"):
        ("caller_supplied", "reaches_return", "database_clock",
         "all three callers bind `at` from database_now_in_tx or tx.database_now(); attest_capability_in_tx inherits it from profile_admin.rs"),
    ("reasonbraid-server/src/quota.rs",
     "check_in_tx", "quota_events.at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/recruitment.rs",
     "open_call", "recruitment_calls.advertises_at"):
        ("rust_clock", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/recruitment.rs",
     "open_call", "recruitment_calls.expires_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/recruitment.rs",
     "open_call", "recruitment_calls.join_deadline"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/site_authority/mod.rs",
     "audit", "public.site_audit.decided_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/site_authority/operator.rs",
     "issue_boundary", "public.site_boundaries.expires_at"):
        ("normalized", "reaches_return", "normalized",
         "scope.checked() rounds through from_timestamp_micros before any bind"),
    ("reasonbraid-server/src/site_authority/operator.rs",
     "issue_boundary", "public.site_boundaries.valid_from"):
        ("normalized", "reaches_return", "normalized",
         "scope.checked() rounds through from_timestamp_micros before any bind"),
    ("reasonbraid-server/src/site_authority/operator.rs",
     "issue_grant", "public.site_grants.expires_at"):
        ("normalized", "reaches_return", "normalized",
         "scope.checked() rounds through from_timestamp_micros before any bind"),
    ("reasonbraid-server/src/site_authority/operator.rs",
     "issue_grant", "public.site_grants.valid_from"):
        ("normalized", "reaches_return", "normalized",
         "scope.checked() rounds through from_timestamp_micros before any bind"),
    ("reasonbraid-server/src/snapshots.rs",
     "submit", "evidence_snapshots.fresh_until"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/snapshots.rs",
     "submit", "evidence_snapshots.retrieved_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
    ("reasonbraid-server/src/snapshots.rs",
     "submit_external", "evidence_snapshots.retrieved_at"):
        ("caller_supplied", "contained", "contained",
         "written and never read back, so the caller's clock is irrelevant: no second copy exists for the column to disagree with"),
}


def adjudication_drift(sites: list[tuple[tuple[str, str, str], str, str]],
                       table: dict[tuple[str, str, str], tuple[str, str, str, str]],
                       ) -> tuple[list[str], list[str], list[str]]:
    """(unclassified, phantom, moved) — the three ways a hand table rots.

    `unclassified` a site the census reports and nobody judged;
    `phantom`      a judgement about a site the census no longer reports;
    `moved`        a site whose (origin, escape) differs from what was judged,
                   so the recorded reason was reached about other facts.
    """
    keys = {k for k, _o, _e in sites}
    unclassified = sorted(f"{k[0]}::{k[1]}::{k[2]}" for k in keys if k not in table)
    phantom = sorted(f"{k[0]}::{k[1]}::{k[2]}" for k in table if k not in keys)
    moved = []
    for key, origin, escape in sites:
        entry = table.get(key)
        if entry and (entry[0], entry[1]) != (origin, escape):
            moved.append(f"{key[0]}::{key[1]}::{key[2]} judged as "
                         f"({entry[0]}, {entry[1]}) but the census now computes "
                         f"({origin}, {escape})")
    return unclassified, phantom, sorted(moved)


def repo_root() -> Path:
    out = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                         capture_output=True, text=True, check=True)
    return Path(out.stdout.strip())


def corpus(root: Path) -> list[str]:
    """Tracked product Rust, tests excluded.

    ⛔ A fixture binding a hand-made instant is not a product defect, and
    `SIGNOFF-REPAIR.11.32`'s measurement is the standing reason to exclude them:
    17 of 19 hits in a sibling census were the check scripts' own test data.
    """
    names = subprocess.run(["git", "ls-files", "--", "crates/*.rs"],
                           cwd=root, capture_output=True, text=True, check=True).stdout.split()
    return sorted(n for n in names if "/tests/" not in n)


def self_test() -> int:
    fails = 0
    ran = 0

    def check(name: str, got, want) -> None:
        nonlocal fails, ran
        ran += 1
        if got != want:
            print(f"BOUND-INSTANTS self-test: {name}: got {got!r}, want {want!r}",
                  file=sys.stderr)
            fails += 1

    check("continuation is applied",
          unescape_rust_string('"a \\\n     b"'), "a b")
    check("raw string is verbatim", unescape_rust_string('r#"a \\n b"#'), "a \\n b")

    check("top-level split keeps calls whole",
          split_top_level("a, f(b, c), d"), ["a", "f(b, c)", "d"])

    sql = ("INSERT INTO t (id, created_at, expires_at) "
           "VALUES ($1, now(), $2) RETURNING expires_at")
    cols = analyse(sql, ["the_id", "the_expiry"])
    check("db-generated column", [c.verdict for c in cols if c.name == "created_at"],
          ["db_generated"])
    check("read-back column", [c.verdict for c in cols if c.name == "expires_at"],
          ["read_back"])
    check("bound expression is resolved positionally",
          [c.bound for c in cols if c.name == "expires_at"], ["the_expiry"])
    check("a non-time column is not reported", [c.name for c in cols],
          ["created_at", "expires_at"])

    sql2 = "INSERT INTO t (id, created_at) VALUES ($1, $2)"
    cols2 = analyse(sql2, ["the_id", "at"])
    check("bound and unread is the candidate class",
          [(c.name, c.verdict, c.bound) for c in cols2],
          [("created_at", "bound_unread", "at")])

    sql3 = "UPDATE t SET revoked_at = $1, note = $2 WHERE id = $3"
    check("update assignments are read",
          [(c.name, c.verdict, c.bound) for c in analyse(sql3, ["when", "n", "i"])],
          [("revoked_at", "bound_unread", "when")])

    sql4 = "UPDATE t SET revoked_at = clock_timestamp() WHERE id = $1"
    check("a database clock in an update is negative",
          [c.verdict for c in analyse(sql4, ["i"])], ["db_generated"])

    check("a select yields nothing", analyse("SELECT created_at FROM t", []), [])

    upsert = ("INSERT INTO t (id, made_at) VALUES ($1, $2) "
              "ON CONFLICT (id) DO NOTHING")
    check("an ON CONFLICT clause does not join the VALUES list",
          [(c.name, c.value, c.verdict) for c in analyse(upsert, ["i", "when"])],
          [("made_at", "$2", "bound_unread")])

    # The schema reader, and the multi-clause ALTER that the first version lost.
    mig = ("CREATE TABLE t (\n  id TEXT NOT NULL,\n  made_at TIMESTAMPTZ NOT NULL,\n"
           "  PRIMARY KEY (id)\n);\n"
           "ALTER TABLE t\n    ADD COLUMN label TEXT,\n"
           "    ADD COLUMN fresh_until TIMESTAMPTZ,\n    ADD COLUMN seen_at TIMESTAMPTZ;\n")
    types = schema_types([mig])
    check("create-table column types", types[("t", "made_at")].split()[0], "TIMESTAMPTZ")
    check("a constraint line is not a column", ("t", "primary") in types, False)
    check("every clause of a multi-clause ALTER is read",
          sorted(c for (tb, c), v in types.items() if TIMESTAMP_TYPE.match(v)),
          ["fresh_until", "made_at", "seen_at"])
    check("a non-time added column is typed, not dropped",
          types[("t", "label")].split()[0], "TEXT")
    # With the schema present, a TEXT column whose NAME looks time-shaped is not
    # a timestamp — the 4-in-44 error the name pattern made.
    check("the schema overrules the name",
          analyse("INSERT INTO t (id, label) VALUES ($1, $2)", ["a", "b"], types), [])

    sqlite = ('use sqlx::SqlitePool;\n'
              'sqlx::query("INSERT INTO attempts (id, updated_at) VALUES ($1, $2)")\n'
              '    .bind(i)\n    .bind(when)\n')
    check("a SQLite file is a different store, not an unknown column",
          [c.verdict for c in scan_file(sqlite, {})], ["other_store"])

    code = 'sqlx::query("INSERT INTO t (a, created_at) VALUES ($1, $2)")\n    .bind(x)\n    .bind(f(y, z))\n'
    check("bind chain is parenthesis-balanced",
          [c.bound for c in scan_file(code)], ["f(y, z)"])

    # the drift guard's three failure modes, each pinned in both directions
    k1 = ("f.rs", "fn_a", "t.c")
    k2 = ("f.rs", "fn_b", "t.d")
    tab = {k1: ("caller_supplied", "contained", "contained", "why")}
    check("drift: agreeing table is clean",
          adjudication_drift([(k1, "caller_supplied", "contained")], tab), ([], [], []))
    check("drift: an unjudged site is caught",
          adjudication_drift([(k1, "caller_supplied", "contained"),
                              (k2, "rust_clock", "contained")], tab)[0],
          ["f.rs::fn_b::t.d"])
    check("drift: a judgement with no site is caught",
          adjudication_drift([], tab)[1], ["f.rs::fn_a::t.c"])
    check("drift: a site whose inputs moved is caught",
          len(adjudication_drift([(k1, "rust_clock", "reaches_return")], tab)[2]), 1)
    check("drift: every shipped verdict is in the vocabulary",
          sorted({v[2] for v in ADJUDICATION.values()} - set(VERDICTS)), [])

    if fails:
        print(f"BOUND-INSTANTS: {fails} of {ran} self-test control(s) failed", file=sys.stderr)
        return 1
    print(f"census_bound_instants: self-test ok — {ran} controls "
          "(continuation, splitting, placeholder mapping, all three verdicts)")
    return 0


def collect(root: Path) -> list[tuple[tuple[str, str, str], str, str]]:
    """Every `bound_unread` site, keyed the way the adjudication is."""
    types = schema_types([m.read_text(errors="replace")
                          for m in sorted((root / "migrations").glob("*.sql"))])
    out = []
    for rel in corpus(root):
        for c in scan_file((root / rel).read_text(errors="replace"), types):
            if c.verdict == "bound_unread":
                out.append(((rel.split("crates/")[-1], c.fn, f"{c.table}.{c.name}"),
                            c.origin, c.escape))
    return out


def check(root: Path) -> int:
    """Refuse when the hand adjudication and the census have come apart."""
    sites = collect(root)
    unclassified, phantom, moved = adjudication_drift(sites, ADJUDICATION)
    bad_verdict = sorted(f"{k[0]}::{k[2]} -> {v[2]}" for k, v in ADJUDICATION.items()
                         if v[2] not in VERDICTS)
    fails = 0
    for label, items, why in (
        ("unadjudicated", unclassified,
         "the census reports this site and nobody judged it"),
        ("phantom", phantom,
         "a judgement about a site the census no longer reports"),
        ("moved", moved,
         "the reason was reached about facts that have since changed"),
        ("unknown verdict", bad_verdict,
         f"a verdict outside {', '.join(VERDICTS)}"),
    ):
        for item in items:
            print(f"BOUND-INSTANTS: {label} — {item}\n    ({why})", file=sys.stderr)
            fails += 1
    if fails:
        print(f"BOUND-INSTANTS: {fails} adjudication breach(es).", file=sys.stderr)
        return 1
    by_verdict: dict[str, int] = {}
    for key, _o, _e in sites:
        by_verdict[ADJUDICATION[key][2]] = by_verdict.get(ADJUDICATION[key][2], 0) + 1
    print(f"BOUND-INSTANTS: OK — {len(sites)} sites over {len(ADJUDICATION)} keys, "
          f"every one adjudicated: "
          + ", ".join(f"{k} {v}" for k, v in sorted(by_verdict.items())))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--check", action="store_true",
                    help="refuse when the hand adjudication and the census have "
                         "come apart (unadjudicated, phantom, or moved sites)")
    ap.add_argument("--candidates", action="store_true",
                    help="list only the bound-and-unread columns, with the Rust "
                         "expression each one binds")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if args.check:
        return check(repo_root())

    root = repo_root()
    files = corpus(root)
    migrations = sorted((root / "migrations").glob("*.sql"))
    types = schema_types([m.read_text(errors="replace") for m in migrations])
    rows: list[tuple[str, Column]] = []
    for rel in files:
        for col in scan_file((root / rel).read_text(errors="replace"), types):
            rows.append((rel, col))

    counts = {k: sum(1 for _r, c in rows if c.verdict == k)
              for k in ("db_generated", "read_back", "bound_unread", "unresolved",
                        "unknown_column", "other_store")}
    print(f"=== timestamp columns written by INSERT/UPDATE — {len(files)} product Rust "
          f"files, {len(migrations)} migrations declaring "
          f"{sum(1 for v in types.values() if TIMESTAMP_TYPE.match(v))} timestamp columns, "
          f"{len(rows)} write sites ===\n")
    for key, label in (
        ("db_generated", "filled by the database clock — one value, no second copy"),
        ("read_back", "bound from Rust AND returned — the caller gets the stored value"),
        ("bound_unread", "bound from Rust and NOT read back — the candidate class"),
        ("unresolved", "the census could not resolve the value — read these by hand"),
        ("other_store", "the node's SQLite journal — a different store, its own schema"),
        ("unknown_column", "time-shaped but absent from the schema — this parser's reach"),
    ):
        print(f"  {key:<14} {counts[key]:>4}   {label}")

    print("\nCANDIDATES — bound from Rust, not read back")
    print("  ⚠️ A CANDIDATE IS NOT A DEFECT. It becomes one only when the Rust value")
    print("     also escapes to a caller or is compared against the stored column,")
    print("     which is a judgement over the surrounding code and is not mechanized.")
    by_file: dict[str, list[Column]] = {}
    for rel, col in rows:
        if col.verdict == "bound_unread":
            by_file.setdefault(rel, []).append(col)
    for rel in sorted(by_file):
        print(f"\n  {rel}")
        for col in by_file[rel]:
            print(f"      {col.statement:<6} {col.table:<28} {col.name:<18} <- {col.bound}")
    if args.candidates:
        return 0
    print("\n  ⛔ NO RULE IS PROPOSED HERE (`SIGNOFF-REPAIR.11.6`, and `.11.31`'s own")
    print("     prohibition): a check written before the population is known is a check")
    print("     fitted to the two examples in hand.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

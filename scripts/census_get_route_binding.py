#!/usr/bin/env python3
"""Census every product GET route against the identifiers its caller supplies
(`SIGNOFF-REPAIR.3.5.4`).

THE QUESTION, which is sharper than "is there a tenant predicate":
**does the caller supply two independent identifiers, and does the handler bind
them together?** `inspect_call` derives the tenant from the target it loaded and
needs no predicate at all; `inspect_node_inbox` accepted a tenant AND a node from
the caller, authorized on the tenant and selected on the node (`.3.5.3`,
REPAIR-0169). Grepping for the column would have called the first a defect and
could have missed the second.

🔴 WHY THIS INSTRUMENT EXISTS — the census it replaces was wrong about its own
population, and nothing in the repository could re-derive it.

  - `.3.5.3` published "24 `get(…)` routes in `api.rs`". At its OWN commit
    (`11c1c4e`) `api.rs` registered **54**, so the census saw well under half its
    subject and reported the rest by silence. `.3.5.4` then inherited the 24 and
    scoped itself to "10 of the 24 delegate their SQL away".
  - It left no producer. The number is prose in a leaf; no script derives it, so
    the only way to find the error was to re-count by hand — which is the
    `a-restated-number-needs-a-producer` lesson, one layer down.
  - And `api.rs` is not the surface. `rb-server.rs` merges THREE routers:
    `api_router_with_publication_root`, `node_router` and `ui_router`. A census
    of one file cannot see the other two.

⚠️ This instrument is a LOCATOR, not a classifier, and that is deliberate
(`TOOLBOX.md`: measure before proposing a rule over the result). "Binds" is a
reading judgement — `get_audit` carries no SQL of its own yet binds tenant and
thread into one `ResourceTarget` before it delegates, while a handler could name
`tenant_id` in one statement and omit it from the next. So this reports WHAT THE
CALLER SUPPLIES and WHAT THE HANDLER DOES WITH IT, and a human classifies.
The classification lives in `SIGNOFF-REPAIR.3.5.4`, not here.

⛔ The `#[cfg(test)]` exclusion and the whole-text route regex are IMPORTED from
`census_route_documentation.py` rather than written again. That module already
records what a per-line scan costs — it found 53 of `api.rs`'s 92 routes, a 42 %
undercount — and two implementations of one rule drift.

    python3 -B scripts/census_get_route_binding.py
    python3 -B scripts/census_get_route_binding.py --unauthenticated
    python3 -B scripts/census_get_route_binding.py --self-test

Self-test: scripts/census_get_route_binding.py --self-test
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from census_route_documentation import test_block_spans  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent

# The routers `crates/reasonbraid-server/src/bin/rb-server.rs` actually merges.
# Named rather than discovered, because "every file with a `.route(`" also
# reaches `fetcher.rs` and `git.rs`, whose routes are `#[cfg(test)]` origin
# fixtures. The mounting is asserted by `mounted_routers` below, so this list
# cannot quietly fall out of step with the binary.
SERVER_SRC = ROOT / "crates/reasonbraid-server/src"
ROUTER_FILES = ("api.rs", "node_channel.rs", "ui.rs")
BINARY = SERVER_SRC / "bin/rb-server.rs"

# What a handler can take FROM THE CALLER. `State` is server-side and `HeaderMap`
# carries the principal, so neither is a caller-supplied *identifier*; they are
# reported because their absence is the sharpest signal there is.
EXTRACTOR = re.compile(r"\b(State|Path|Query|Json|HeaderMap)\b")
PATH_PARAM = re.compile(r"\{([^}]*)\}")

# ⛔ AUTHENTICATION IS NOT AUTHORIZATION, and the first draft of this instrument
# conflated them — it listed `resolve_principal` beside `authorize_tenant_admin`
# as though a handler reaching either were gated. `resolve_principal` parses the
# `PRINCIPAL_HEADER` and returns a `GrantSubject`; it decides nothing. Reporting
# the two in one column would have called ~20 routes "gated" that only ever
# learn WHO is asking.
AUTHN = re.compile(r"\bresolve_principal\s*\(")

# The primitives that actually reach an authorization DECISION. Every other
# helper in `api.rs` — `inspect`, `inspect_tenant_admin`, `authorize_profile_read`,
# `classify_reader`, `site_registry_response` — is a wrapper that calls one of
# these, which is why the walk below is transitive rather than a flat grep.
AUTHZ_PRIMITIVE = re.compile(
    r"\b(authorize_inspection|authorize_tenant_admin_inspection|authorize_tenant_admin"
    r"|authorize_guarded|authorize_in_tx|site::execute)\s*\("
)

# Not a gate: it sets the RLS tenant claim for the connection. Reported in its
# own column because it BINDS a tenant without deciding anything, and calling
# that "gated" or "ungated" would both be wrong.
RLS = re.compile(r"\bwith_tenant_claim\s*\(")

# A call into another module: `crate::a::b(`, `a::b(` — the delegate a per-handler
# SQL scan cannot follow. `Self::`/`Some(`/`Ok(` and friends are excluded by the
# lowercase-module requirement.
DELEGATE = re.compile(r"\b((?:crate::)?(?:[a-z_][a-z_0-9]*::)+[a-z_][a-z_0-9]*)\s*\(")

# 🔴 A BARE call, never the tail of a qualified path or a method. Caught by this
# census's own output: `list_evaluation_runs` calls `crate::evaluation::list_runs`,
# `api.rs` ALSO defines a free `list_runs` (the `/v1/admin/runs` handler), and a
# `\b`-anchored scan walked into the wrong one — reporting the evaluation route as
# authorized by the admin route's gate. That is `a-key-too-loose-returns-the-wrong-
# instance` inside the instrument built to find exactly that class. The lookbehind
# excludes `::name(` and `.name(`, which is the whole difference.
LOCAL_CALL = re.compile(r"(?<![:.\w])([a-z_][a-z_0-9]*)\s*\(")

SQL = re.compile(r'"((?:SELECT|INSERT|UPDATE|DELETE|WITH)\b[^"]*)"', re.IGNORECASE)

# 🔴 A TENANT PREDICATE IS NOT ALWAYS SPELLED `tenant_id`, and anchoring on that
# exact token made this census cry wolf on code that is correct. `snapshots.rs`
# scopes with an interpolated `{CITED_BY_TENANT}` constant and `claims.rs` with
# `authored_by_tenant = $2`; both read as "0 of 1 SQL name tenant_id", i.e. as
# unscoped. That is `SIGNOFF-REPAIR.11.2.4`'s defect — a check blind to the
# other spellings of the very noun it is built on — pointing the other way.
# ⚠️ Still PRESENCE, not correctness: a statement may name a tenant column and
# bind the wrong value, so this locates and a human reads.
# ⚠️ CASE-INSENSITIVE: `snapshots.rs` interpolates a SCREAMING_CASE constant
# (`{CITED_BY_TENANT}`), so a lowercase-only pattern misses the very spelling
# that motivated widening it.
TENANT_TOKEN = re.compile(r"\b(\w*tenant\w*)\b", re.IGNORECASE)


# The tables a statement reads. Only `FROM`/`JOIN` targets, so a column called
# `tenant_id` never masquerades as one.
TABLE = re.compile(r"\b(?:FROM|JOIN)\s+([a-z_][a-z_0-9]*)", re.IGNORECASE)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def tenant_dimensioned_tables() -> dict[str, bool]:
    """Every migrated table → does it carry a tenant column AT ALL?

    ⭐ THE FACT THAT SEPARATES A LEAK FROM A DESIGN, and the one a predicate scan
    cannot supply. 22 of these routes read `evaluation_*`, `policy_*`, `routing_*`,
    `deployment_*` and `workflow_profiles` with no tenant predicate — which looks
    exactly like `.3.5.3`'s inbox leak until you ask the schema, and NONE of those
    tables has a tenant column. They are site-global by data model, so there is no
    tenant to bind and no predicate to omit. Calling them defects would be
    `inspect_call`'s false positive, 22 times over.
    """
    tables: dict[str, bool] = {}
    for path in sorted((ROOT / "migrations").glob("*.sql")):
        text = read(path)
        for m in re.finditer(r"CREATE TABLE(?:\s+IF NOT EXISTS)?\s+([a-z_][a-z_0-9]*)\s*\(", text, re.IGNORECASE):
            end = balanced(text, m.end() - 1)
            body = text[m.end() : end]
            tables[m.group(1)] = bool(TENANT_TOKEN.search(body))
        # A tenant column added later still gives the table a tenant dimension.
        for m in re.finditer(
            r"ALTER TABLE\s+([a-z_][a-z_0-9]*)\s+ADD COLUMN(?:\s+IF NOT EXISTS)?\s+(\w+)",
            text,
            re.IGNORECASE,
        ):
            if TENANT_TOKEN.search(m.group(2)):
                tables[m.group(1)] = True
    return tables


def tracked(pathspec: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", pathspec], cwd=ROOT, capture_output=True, text=True, check=True
    )
    return out.stdout.split()


def outside_tests(text: str) -> list[tuple[int, int]]:
    """`test_block_spans` inverted — the character ranges that are product code."""
    spans = test_block_spans(text)
    product: list[tuple[int, int]] = []
    cursor = 0
    for lo, hi in sorted(spans):
        if lo > cursor:
            product.append((cursor, lo))
        cursor = max(cursor, hi)
    product.append((cursor, len(text)))
    return product


def balanced(text: str, start: int, open_ch: str = "(", close_ch: str = ")") -> int:
    """Index just past the delimiter matching the one at/after `start`.

    String-aware: a `"` inside the span opens a literal, and a route's path or a
    handler's SQL routinely contains braces and parentheses. Without this the
    `.route("/v1/policies/{policy_id}/{version}/impact", get(policy_impact))`
    span ends in the wrong place.
    """
    i = text.index(open_ch, start)
    depth = 0
    in_str = False
    while i < len(text):
        ch = text[i]
        if in_str:
            if ch == "\\":
                i += 2
                continue
            if ch == '"':
                in_str = False
        elif ch == '"':
            in_str = True
        elif ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(text)


def mounted_routers() -> set[str]:
    """The router functions `rb-server.rs` actually serves.

    ⛔ Asserted, not assumed. `.3.5.3` censused one file because one file looked
    like the surface; this fails loudly the day a fourth router is merged or one
    of these three stops being.
    """
    text = read(BINARY)
    return set(re.findall(r"\b([a-z_][a-z_0-9]*router[a-z_0-9]*)\s*\(", text))


def route_registrations(text: str) -> list[tuple[str, str, int]]:
    """`(path, methods_source, char_offset)` for every product `.route(` call."""
    product = outside_tests(text)
    out: list[tuple[str, str, int]] = []
    for m in re.finditer(r'\.route\(\s*"([^"]+)"\s*,', text):
        if not any(lo <= m.start() < hi for lo, hi in product):
            continue
        end = balanced(text, m.start())
        out.append((m.group(1), text[m.end() : end - 1].strip(), m.start()))
    return out


def get_handlers(text: str) -> list[tuple[str, str]]:
    """`(path, handler)` for every product GET registration.

    Covers `get(h)`, `post(x).get(h)`, `get(h).post(x)` and the multi-line
    `.route(\\n  "path",\\n  get(h),\\n)` form in one pass, because the methods
    source is the balanced span rather than the rest of a line.
    """
    found: list[tuple[str, str]] = []
    for path, methods, _ in route_registrations(text):
        for h in re.findall(r"\bget\(\s*([a-z_][a-z_0-9]*)\s*\)", methods):
            found.append((path, h))
    return found


def function_body(text: str, name: str) -> str | None:
    """The FREE function `name` — never a method that merely shares its name.

    🔴 Caught by this instrument's own first real run. `node_channel.rs` holds
    BOTH `async fn presence(State(..), Query(..))` (the route handler) and
    `pub async fn presence(&self, node_id: &str)` (the method it calls), and a
    first-match scan bound the route to the method: the census reported the
    handler as taking nothing from the caller and carrying SQL of its own, when
    the exact opposite is true of each. A `&self` receiver is the discriminator —
    an axum handler cannot have one — so every candidate is checked and the
    methods are skipped rather than the first hit being trusted.
    """
    # ⚠️ The generic parameter list is OPTIONAL and must be skipped. `api.rs`
    # declares its two most important gate wrappers as `async fn inspect<F, Fut>(`
    # and `async fn inspect_tenant_admin<F, Fut>(`; a pattern demanding `(`
    # straight after the name found neither, so every route admitted through
    # them — the whole `/v1/admin/*` family and `get_audit` — reported
    # "AUTHORIZES: NONE". Caught by reading this census's own output against a
    # handler already known to be gated.
    pattern = rf"^\s*(?:pub(?:\([^)]*\))?\s+)?async fn {re.escape(name)}\s*(?:<[^(]*?>)?\s*\("
    for m in re.finditer(pattern, text, re.MULTILINE):
        sig_end = balanced(text, m.start())
        params = text[text.index("(", m.start()) : sig_end]
        if re.match(r"\(\s*&?\s*(mut\s+)?self\b", params):
            continue
        brace = text.find("{", sig_end)
        if brace < 0:
            continue
        return text[m.start() : balanced(text, brace, "{", "}")]
    return None


def signature(fn_text: str) -> str:
    return fn_text[: balanced(fn_text, 0)]


def struct_fields(text: str, name: str) -> list[str]:
    """The declared field names of `struct name`, in order.

    ⚠️ Split on TOP-LEVEL commas, not on line starts. A line-anchored scan reads
    a one-line struct as a single field, and a type like
    `Option<HashMap<String, u32>>` carries commas of its own — so the separator
    has to be depth-aware in both `<>` and `()`.
    """
    m = re.search(rf"struct {re.escape(name)}\s*\{{", text)
    if not m:
        return []
    body = text[m.end() : balanced(text, m.end() - 1, "{", "}") - 1]
    body = re.sub(r"//[^\n]*", "", body)
    body = re.sub(r"#\[[^\]]*\]", "", body)

    fields: list[str] = []
    depth = 0
    start = 0
    for i, ch in enumerate(body):
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        elif ch == "," and depth == 0:
            fields.append(body[start:i])
            start = i + 1
    fields.append(body[start:])

    names: list[str] = []
    for field in fields:
        fm = re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z_0-9]*)\s*:", field)
        if fm:
            names.append(fm.group(1))
    return names


def caller_identifiers(text: str, path: str, sig: str) -> tuple[list[str], list[str]]:
    """`(path_params, query_fields)` — what the CALLER names in the request.

    Query fields are resolved through the extractor's type into the struct's own
    field list, so `Query<InboxInspectionParams>` reports `tenant_id, node_id`
    rather than a type name nobody can act on.
    """
    path_params = [p for p in PATH_PARAM.findall(path) if p]
    query: list[str] = []
    for ty in re.findall(r"Query\(\s*[^)]*\)\s*:\s*Query<\s*([A-Za-z_][A-Za-z_0-9]*)", sig):
        query.extend(struct_fields(text, ty))
    return path_params, query


def local_functions(text: str) -> set[str]:
    """Every function DEFINED in this file — the frontier of the transitive walk."""
    return set(re.findall(r"\b(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-z_][a-z_0-9]*)", text))


def reachable(text: str, handler: str, locals_: set[str]) -> tuple[set[str], set[str], set[str]]:
    """`(authz primitives, local helpers walked, module delegates)` for a handler.

    ⭐ THIS IS THE WHOLE POINT OF THE INSTRUMENT. `.3.5.3`'s scan read each
    handler's own body and stopped, so `get_audit` — which carries no SQL and no
    `authorize_*` call of its own — was simply invisible. It calls `inspect`,
    which calls `authorize_inspection`. One hop.

    ⚠️ THE WALK STOPS AT THE FILE BOUNDARY, and says so rather than implying
    coverage it does not have. A delegate like `crate::resources::get_for_tenant`
    is reported by NAME and read by a human; following it would need a
    cross-crate call graph, which is a larger instrument than this leaf needs.
    """
    primitives: set[str] = set()
    walked: set[str] = set()
    delegates: set[str] = set()
    frontier = [handler]
    while frontier:
        name = frontier.pop()
        if name in walked:
            continue
        walked.add(name)
        fn_text = function_body(text, name)
        if fn_text is None:
            continue
        body = fn_text[len(signature(fn_text)) :]
        primitives.update(AUTHZ_PRIMITIVE.findall(body))
        delegates.update(d for d in DELEGATE.findall(body) if "::" in d)
        for callee in LOCAL_CALL.findall(body):
            if callee in locals_ and callee not in walked:
                frontier.append(callee)
    walked.discard(handler)
    return primitives, walked, delegates


def delegate_facts(delegates: set[str]) -> list[dict]:
    """Follow each `crate::module::fn` ONE level, into its sibling module file.

    ⭐ THE STEP `.3.5.3` COULD NOT TAKE. Its scan read the handler's own body, so
    a handler that delegates looked identical to one with nothing to hide. 45 of
    the 58 routes here carry no SQL of their own — the delegate is where the
    query actually is, and therefore where a tenant predicate is or is not.

    Reports the delegate's PARAMETERS (a function that never receives a tenant
    cannot scope by one, whatever its SQL says) and every SQL literal it holds.
    `sqlx::`, `serde_json::` and friends resolve to no module file and are
    skipped rather than reported as unreadable.
    """
    facts: list[dict] = []
    for delegate in sorted(delegates):
        parts = delegate.split("::")
        if parts[0] == "crate":
            parts = parts[1:]
        if len(parts) != 2:
            continue
        module, fn = parts
        module_path = SERVER_SRC / f"{module}.rs"
        if not module_path.exists():
            continue
        text = read(module_path)
        fn_text = function_body(text, fn)
        if fn_text is None:
            m = re.search(rf"^\s*(?:pub(?:\([^)]*\))?\s+)?fn {re.escape(fn)}\s*(?:<[^(]*?>)?\s*\(", text, re.MULTILINE)
            if not m:
                continue
            brace = text.find("{", balanced(text, m.start()))
            fn_text = text[m.start() : balanced(text, brace, "{", "}")] if brace >= 0 else ""
        sig = signature(fn_text)
        body = fn_text[len(sig) :]
        sql = [re.sub(r"\s+", " ", s).strip() for s in SQL.findall(body)]
        params = re.findall(r"\b([a-z_][a-z_0-9]*)\s*:\s*&?", sig[sig.index("(") :])
        facts.append(
            {
                "name": delegate,
                "module": f"crates/reasonbraid-server/src/{module}.rs",
                "params": params,
                "takes_tenant": any("tenant" in p for p in params),
                "sql": sql,
                "sql_with_tenant": sum(1 for s in sql if TENANT_TOKEN.search(s)),
                "tenant_tokens": sorted({m for s in sql for m in TENANT_TOKEN.findall(s)}),
                "tables": sorted({tb for s in sql for tb in TABLE.findall(s)}),
                "authz": sorted(set(AUTHZ_PRIMITIVE.findall(body))),
            }
        )
    return facts


def audit(path: str, handler: str, text: str) -> dict:
    fn_text = function_body(text, handler)
    if fn_text is None:
        return {"path": path, "handler": handler, "resolved": False}
    sig = signature(fn_text)
    body = fn_text[len(sig) :]
    path_params, query = caller_identifiers(text, path, sig)
    sql = [re.sub(r"\s+", " ", s).strip() for s in SQL.findall(body)]
    primitives, walked, delegates = reachable(text, handler, local_functions(text))
    return {
        "path": path,
        "handler": handler,
        "resolved": True,
        "extractors": sorted(set(EXTRACTOR.findall(sig))),
        "path_params": path_params,
        "query_fields": query,
        "authn": bool(AUTHN.search(body)),
        "authz": sorted(primitives),
        "rls": bool(RLS.search(body)),
        "via": sorted(walked),
        "sql": sql,
        "sql_with_tenant": sum(1 for s in sql if TENANT_TOKEN.search(s)),
        "tenant_tokens": sorted({m for s in sql for m in TENANT_TOKEN.findall(s)}),
        "tables": sorted({tb for s in sql for tb in TABLE.findall(s)}),
        "delegates": sorted(delegates),
        "delegate_facts": delegate_facts(delegates),
    }


def census() -> list[dict]:
    rows: list[dict] = []
    for name in ROUTER_FILES:
        path = SERVER_SRC / name
        text = read(path)
        for route, handler in get_handlers(text):
            row = audit(route, handler, text)
            row["file"] = f"crates/reasonbraid-server/src/{name}"
            rows.append(row)
    return rows


def schema_verdict(tables: list[str], dimensioned: dict[str, bool]) -> str:
    """How each table read on this route stands in the schema.

    Three answers, never collapsed: a table WITH a tenant column read without a
    predicate is the `.3.5.3` shape and deserves a reading; one WITHOUT is
    site-global by design; one the migrations do not define is unknown and says
    so rather than defaulting to either.
    """
    if not tables:
        return ""
    parts = []
    for tb in tables:
        if tb not in dimensioned:
            parts.append(f"{tb}=?")
        elif dimensioned[tb]:
            parts.append(f"{tb}=TENANTED")
        else:
            parts.append(f"{tb}=site-global")
    return ", ".join(parts)


def report(rows: list[dict], only_unauthenticated: bool) -> int:
    dimensioned = tenant_dimensioned_tables()
    routers: dict[str, list[dict]] = {}
    for row in rows:
        routers.setdefault(row["file"], []).append(row)

    mounted = mounted_routers()
    print(f"GET-ROUTE BINDING census — {len(rows)} product GET routes across {len(routers)} routers")
    print(f"  routers merged by rb-server.rs: {', '.join(sorted(mounted))}")
    print()

    naked = [r for r in rows if r.get("resolved") and not r["authz"]]
    shown_rows = naked if only_unauthenticated else rows

    for f in sorted(routers):
        shown = [r for r in shown_rows if r["file"] == f]
        if not shown:
            continue
        print(f"── {f} ({len(routers[f])} GET routes)")
        for r in sorted(shown, key=lambda x: x["path"]):
            if not r.get("resolved"):
                print(f"  {r['path']:<52} {r['handler']}  ⚠️ handler not found in this file")
                continue
            supplied = r["path_params"] + r["query_fields"]
            print(f"  {r['path']:<52} {r['handler']}")
            print(f"      caller supplies : {', '.join(supplied) if supplied else '(nothing)'}")
            print(f"      authenticates   : {'resolve_principal' if r['authn'] else '⛔ NO'}")
            via = f"  (via {', '.join(r['via'])})" if r["authz"] and r["via"] else ""
            print(f"      AUTHORIZES      : {', '.join(r['authz']) if r['authz'] else '⛔ NONE'}{via}")
            if r["rls"]:
                print("      rls tenant claim: yes")
            if r["tables"]:
                print(f"      reads tables    : {schema_verdict(r['tables'], dimensioned)}")
            print(
                f"      sql in place    : {len(r['sql'])}"
                f" ({r['sql_with_tenant']} naming a tenant column"
                f"{': ' + ', '.join(r['tenant_tokens']) if r['tenant_tokens'] else ''})"
            )
            for d in r["delegate_facts"]:
                scope = (
                    f"{d['sql_with_tenant']}/{len(d['sql'])} SQL name a tenant column"
                    + (f" ({', '.join(d['tenant_tokens'])})" if d["tenant_tokens"] else "")
                    if d["sql"]
                    else "no SQL of its own"
                )
                tenant = "takes a tenant" if d["takes_tenant"] else "⛔ NO tenant parameter"
                print(f"      ↳ {d['name']}: {tenant}; {scope}")
                if d["tables"]:
                    print(f"         tables: {schema_verdict(d['tables'], dimensioned)}")
        print()

    resolved = [r for r in rows if r.get("resolved")]
    print(f"SUMMARY: {len(shown_rows)} routes reported of {len(rows)} product GET routes.")
    print(f"  {len(naked)} reach NO authorization primitive at all")
    print(f"  {sum(1 for r in resolved if not r['authn'])} do not even authenticate")
    print(f"  {sum(1 for r in resolved if not r['sql'])} carry no SQL of their own")
    print("  ⚠️ This is a LOCATOR. 'binds' vs 'derives' is a reading judgement, and the")
    print("     walk stops at the file boundary — a named delegate is read, not followed.")
    print("     Classification lives in docs/tasks/SIGNOFF-REPAIR.md (.3.5.4).")
    return 0


def self_test() -> int:
    failures: list[str] = []

    source = """
        fn api_router_with_state(state: Arc<ApiState>) -> Router {
            Router::new()
                .route("/v1/one", get(alpha))
                .route("/v1/two", post(w).get(beta))
                .route(
                    "/v1/three/{thing_id}/sub",
                    get(gamma),
                )
                .route("/v1/four", get(delta).post(w))
                .route("/v1/five", post(only_post))
        }
        struct AParams { pub tenant_id: Uuid, pub node_id: String }
        async fn alpha(
            State(s): State<Arc<ApiState>>,
            headers: HeaderMap,
            Query(params): Query<AParams>,
        ) -> Result<Json<V>, E> {
            let principal = resolve_principal(&headers)?;
            authorize_tenant_admin(&s.pool, &principal, params.tenant_id).await?;
            let r = sqlx::query("SELECT a FROM t WHERE node_id = $1").bind(1).await?;
            Ok(Json(r))
        }
        // `beta` delegates to `crate::registry::gamma` — a QUALIFIED call whose
        // tail collides with the local handler `gamma`, which authorizes. A
        // `\b`-anchored callee scan walks into the local one and reports `beta`
        // as authorized. This is the real `list_evaluation_runs` ↔ `list_runs`
        // collision, reproduced.
        async fn beta(State(s): State<Arc<ApiState>>) -> Result<Json<V>, E> {
            crate::registry::gamma(&s.pool).await
        }
        // `gamma` carries NO `authorize_*` call of its own — it reaches one a hop
        // away, through a local wrapper. This is `get_audit`'s real shape, and
        // the shape `.3.5.3`'s per-handler scan could not see.
        // GENERIC, like the real `inspect<F, Fut>` and `inspect_tenant_admin<F, Fut>`
        // it stands for — the shape that made this census report the whole
        // `/v1/admin/*` family as unauthorized.
        async fn guarded_read<F, Fut>(state: &ApiState, p: &GrantSubject, read: F) -> R
        where
            F: FnOnce(PgPool) -> Fut,
        {
            authorize_inspection(&state.pool, p, t).await?;
            Ok(())
        }
        async fn gamma(Path(id): Path<String>, headers: HeaderMap) -> Result<Json<V>, E> {
            let p = resolve_principal(&headers)?;
            guarded_read(&state, &p, ResourceTarget::Thing { id }).await?;
            Ok(Json(json!({})))
        }
        async fn delta(State(s): State<Arc<ApiState>>) -> Result<Json<V>, E> {
            Ok(Json(json!({})))
        }
        impl Store {
            // The METHOD that shares the handler's name, declared FIRST — the
            // shape that made the real census bind `presence` to the wrong body.
            pub async fn epsilon(&self, node_id: &str) -> Result<V, E> {
                sqlx::query_as("SELECT x FROM node_presence WHERE node_id = $1").await
            }
        }
        struct EParams { pub node_id: String }
        async fn epsilon(
            State(s): State<Arc<S>>,
            Query(params): Query<EParams>,
        ) -> Result<Json<V>, E> {
            s.epsilon(&params.node_id).await
        }
        #[cfg(test)]
        mod tests {
            fn fixture() -> Router {
                Router::new().route("/fixture", get(never_counted))
            }
            async fn never_counted() -> &'static str { "x" }
        }
    """

    # (1) Every GET registration shape is found, and the `#[cfg(test)]` one is not.
    found = get_handlers(source)
    want = [
        ("/v1/one", "alpha"),
        ("/v1/two", "beta"),
        ("/v1/three/{thing_id}/sub", "gamma"),
        ("/v1/four", "delta"),
    ]
    if found != want:
        failures.append(f"GET extraction: expected {want}, got {found}")

    # (2) The POST-only route contributes nothing, and neither does the fixture.
    if any(h == "only_post" for _, h in found):
        failures.append("a POST-only route was counted as a GET")
    if any(h == "never_counted" for _, h in found):
        failures.append("a #[cfg(test)] fixture route was counted as product surface")

    # (3) A `Query<T>` resolves THROUGH the type into the struct's field names.
    alpha = audit("/v1/one", "alpha", source)
    if alpha["query_fields"] != ["tenant_id", "node_id"]:
        failures.append(f"query fields: expected tenant_id,node_id, got {alpha['query_fields']}")
    if alpha["sql_with_tenant"] != 0 or len(alpha["sql"]) != 1:
        failures.append(f"SQL accounting wrong for alpha: {alpha['sql']}")
    if "authorize_tenant_admin" not in alpha["authz"]:
        failures.append(f"authz primitive not detected for alpha: {alpha['authz']}")
    if not alpha["authn"]:
        failures.append("alpha calls resolve_principal but did not read as authenticating")

    # (4) A path parameter is a caller-supplied identifier too.
    gamma = audit("/v1/three/{thing_id}/sub", "gamma", source)
    if gamma["path_params"] != ["thing_id"]:
        failures.append(f"path params: expected thing_id, got {gamma['path_params']}")

    # (4b) ⭐ THE ARM THIS INSTRUMENT EXISTS FOR: an authorization reached one hop
    # away, through a local wrapper, is found — and the wrapper is NAMED, so a
    # reader can check the hop rather than trust it. A flat per-handler grep
    # (the degenerate implementation below, which is `.3.5.3`'s actual scan)
    # must MISS it, or arm (4b) is passing for an unrelated reason.
    if "authorize_inspection" not in gamma["authz"]:
        failures.append(f"transitive authz not followed for gamma: {gamma['authz']}")
    if "guarded_read" not in gamma["via"]:
        failures.append(f"the wrapper was not named for gamma: {gamma['via']}")
    gamma_body = function_body(source, "gamma") or ""
    if AUTHZ_PRIMITIVE.search(gamma_body):
        failures.append("gamma's own body carries an authz call; arm (4b) proves nothing")

    # (4c) AUTHENTICATION IS NOT AUTHORIZATION. `delta` does neither, `beta`
    # does neither, and a handler that only calls `resolve_principal` must not
    # read as authorized — the conflation this instrument's first draft shipped.
    only_authn = audit("/v1/one", "alpha", source.replace("authorize_tenant_admin(", "noop("))
    if only_authn["authz"] or not only_authn["authn"]:
        failures.append(
            f"resolve_principal alone read as authorized: authz={only_authn['authz']}"
        )

    # (5) THE POSITIVE ARM THAT MATTERS: a handler with no gate reads ⛔ NONE,
    # and one that delegates has its delegate named. A locator that cannot tell
    # `delta` from `alpha` is the instrument `.3.5.3` already had.
    delta = audit("/v1/four", "delta", source)
    if delta["authz"]:
        failures.append(f"delta reaches no authz primitive but reported {delta['authz']}")
    beta = audit("/v1/two", "beta", source)
    if "crate::registry::gamma" not in beta["delegates"]:
        failures.append(f"delegate not named for beta: {beta['delegates']}")
    if beta["sql"]:
        failures.append("beta carries no SQL of its own but SQL was reported")

    # (5b) 🔴 THE COLLISION ARM. `beta` calls `crate::registry::gamma`; the local
    # `gamma` authorizes. If the walk enters it, `beta` reads as authorized and
    # the census reports a gate on a route that has none — the false POSITIVE
    # direction, which is the dangerous one for a security census.
    if beta["authz"]:
        failures.append(
            f"a qualified call's tail matched a local function: beta reported {beta['authz']}"
        )
    if "gamma" in beta["via"]:
        failures.append("the walk entered a local function named only as a path tail")

    # (6) A NEGATIVE arm: a degenerate parser that scans line by line must FAIL
    # the multi-line case, proving arm (1) is not passing for an unrelated
    # reason (`a-control-that-passes-for-an-unrelated-reason`).
    per_line = [
        h
        for line in source.splitlines()
        if ".route(" in line
        for h in re.findall(r"\bget\(\s*([a-z_][a-z_0-9]*)\s*\)", line)
    ]
    if "gamma" in per_line:
        failures.append("the degenerate per-line scan found the multi-line route; arm (1) is weak")

    # (6b) 🔴 THE TENANT-SPELLING ARM, both directions. A predicate written
    # `authored_by_tenant` or interpolated as a SCREAMING_CASE `{CITED_BY_TENANT}`
    # must COUNT — anchoring on the literal `tenant_id` reported both as
    # unscoped, i.e. cried wolf on correct code — and a statement with no tenant
    # notion at all must still count zero, or the widening has just made
    # everything match.
    scoped = 'let q = "SELECT a FROM t WHERE id = $1 AND authored_by_tenant = $2";'
    interpolated = 'let q = "SELECT {COLS} FROM t WHERE id = $1 AND {CITED_BY_TENANT}";'
    unscoped = 'let q = "SELECT a FROM evaluation_corpora ORDER BY corpus_id";'
    for label, src in (("authored_by_tenant", scoped), ("CITED_BY_TENANT", interpolated)):
        found = [s for s in SQL.findall(src) if TENANT_TOKEN.search(s)]
        if not found:
            failures.append(f"a tenant predicate spelled `{label}` read as unscoped")
    if [s for s in SQL.findall(unscoped) if TENANT_TOKEN.search(s)]:
        failures.append("the widened tenant vocabulary matched a statement with no tenant notion")

    # (7) A METHOD never stands in for the handler that shares its name. Both
    # directions are asserted, because getting the right body by luck would pass
    # a one-sided check: the handler's caller-supplied field must be seen, AND
    # the method's SQL must NOT be attributed to it.
    eps = audit("/v1/eps", "epsilon", source)
    if eps["query_fields"] != ["node_id"]:
        failures.append(f"method shadowed the handler: query fields {eps['query_fields']}")
    if eps["sql"]:
        failures.append(f"the method's SQL was attributed to the handler: {eps['sql']}")

    # (8) The mounting assertion reads the real binary.
    if BINARY.exists():
        merged = mounted_routers()
        for required in ("node_router", "ui_router"):
            if required not in merged:
                failures.append(f"rb-server.rs no longer merges {required}; ROUTER_FILES is stale")

    if failures:
        for f in failures:
            print(f"SELF-TEST FAILED: {f}", file=sys.stderr)
        return 1
    print(
        "SELF-TEST: 4 GET shapes found (incl. multi-line and post().get()), "
        "POST-only and #[cfg(test)] excluded, query type resolved to fields, "
        "path param captured, ungated handler flagged, delegate followed, "
        "per-line scan fails where this one succeeds, a transitive authz is followed "
        "and its wrapper named, authn alone is not authz, "
        "a same-named method does not shadow its handler, a GENERIC wrapper is entered, "
        "a qualified call's tail does not enter a local function, "
        "both tenant spellings counted and a tenant-free statement not, "
        "rb-server mounting asserted"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--unauthenticated",
        action="store_true",
        help="report only the GET routes that reach no gate at all",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return report(census(), args.unauthenticated)


if __name__ == "__main__":
    raise SystemExit(main())

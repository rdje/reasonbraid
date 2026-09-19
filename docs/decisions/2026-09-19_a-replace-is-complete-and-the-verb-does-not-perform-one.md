# A replace of a resolver advertise is complete, and the HTTP verb does not perform one

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** `SIGNOFF-REPAIR.7.3.6.2` (REASONBRAID-REPAIR-0267) — measured

## The fact / decision

`resolvers::register` now writes **every** advertised column on conflict, not
six of eighteen. `registered_at` is the one exception and stays at the first
registration.

`POST /v1/resolvers` **refuses** an already-registered `resolver_id` with
`invalid_transition` (409), naming the row. It registers; it does not replace.

Two callers, two trusts, and the split is the decision:

| caller | may replace? | why |
| --- | --- | --- |
| `sync_gated_entries`, at boot | **yes, completely** | the advertisement compiled into the binary is the truth; a row that has drifted loses to it |
| `POST /v1/resolvers` | **no** | `resolver_capabilities` has no tenant column, so any tenant administrator addresses any row |

## Why

`register`'s `ON CONFLICT (resolver_id) DO UPDATE SET` wrote `schemes`,
`egress_class`, `sandbox_level`, `version`, `security_evidence` and `max_bytes`.
The other eleven columns it inserts kept their old values silently:
`locator_patterns`, `media_types`, `abilities`, `authentication_classes`, all
four advertised `*_policy` columns, `snapshot_formats`, `derivation_formats`
and `latency_range_ms`.

Two documented promises could not be kept through it:

- `api::register_resolver`'s own doc comment said the operator *registers (or
  **replaces**)* an advertise.
- `resolvers::advertised_media_types`' doc comment — citing
  `[[2026-09-12_r2-acquisition-accept-set]]` — says *a deployment that narrows a
  pack's advertisement narrows what that pack may acquire, in the same act*.
  `media_types` is the field that decision is about, and it is one of the
  eleven.

So the documented use of the verb — narrowing a pack's advertised formats —
answered `200 {"registered": true}` and changed nothing. **A silent no-op with a
success response is worse than either alternative**, because the operator has no
way to find out.

### Why the HTTP verb does not get the complete replace

This is the part that was nearly got wrong, and the reason is a finding this
leaf did not make. `SIGNOFF-REPAIR.11.9.1.1.1` measured that
`resolver_capabilities` has **no tenant column** and a single-column
`resolver_id` primary key, while `register_resolver` admits on the caller's own
`tenant_admin` grant. Any tenant's administrator can therefore address any row,
**including the built-in `r0-https-fetcher`'s**. Binding that authority is
`.7.1`'s work and is still open.

Completing the replace at the route would have handed that unbound principal
eleven more columns on a site-global row — `media_types`, which decides what
the acquisition leg admits, and all four advertised policies among them. **A
repair that satisfies a doc comment by enlarging the surface another leaf has
to bind is not a repair.**

Of the three available answers, only one does not grow that surface:

| answer | the silent no-op | the site-global write surface |
| --- | --- | --- |
| leave it | remains | 6 columns |
| complete the replace at the route | fixed | **17 columns** |
| **refuse at the route** ✅ | fixed | **0 columns on an existing row** |

The refusal also **narrows** `.7.1`'s surface: the replace half of
`SIGNOFF-REPAIR.11.9.1.1.1`'s finding closes. ⛔ It does **not** discharge that
leaf — creating a new site-global resolver on a tenant-admin grant is
untouched, and that is the larger half.

### What is lost, stated plainly

An operator can no longer change a registered resolver's advertise through the
API. That capability was never usable for its documented purpose — it could not
narrow `media_types` and could not correct a policy line — while the part that
did work was the part that should not have (rewriting a built-in pack's egress
class). Removing it removes a hazard and no working use case. An operator who
must change an advertise deletes the row and registers again, which is a
deliberate two-step, or waits for `.7.1` to define the authorized path.

## How to apply

- **`register` is the product's function, not an operator's.** Call it where the
  binary's advertisement is the truth. Anything reachable from a request goes
  through a verb that decides its own replace semantics explicitly.
- **A new advertised column is added to the `DO UPDATE SET` list in the same
  edit that adds it to the INSERT.** The gate that catches the advertisement
  side of this is `scripts/census_advertised_policies.py`
  (`[[2026-09-19_an-advertised-line-carries-an-adjudicated-verdict]]`); the
  column list itself is covered by the control below.
- **Do not widen a site-global write to satisfy a doc comment.** Check whether
  the row has a tenant column first. When it does not, the honest repair is the
  one that refuses.
- Re-derive the column gap at any time:

```bash
python3 - <<'EOF'
import re, pathlib
t = pathlib.Path("crates/reasonbraid-server/src/resolvers.rs").read_text()
sql = re.search(r'sqlx::query\(\s*"(.*?)",\s*\)', t, re.S).group(1).replace("\\\n", "").replace("\\", "")
ins = ["resolver_id"] + [c.strip() for c in re.search(r"\(resolver_id,(.*?)\)\s*VALUES", sql, re.S).group(1).split(",") if c.strip()]
upd = re.findall(r"(\w+)\s*=\s*\$\d+", sql.split("DO UPDATE SET", 1)[1])
print(len(ins), "inserted;", len(upd), "updated; not updated:",
      [c for c in ins if c not in upd and c != "resolver_id"])
EOF
```

Related: `[[2026-09-12_r2-acquisition-accept-set]]` — the decision whose promise
the upsert could not keep.

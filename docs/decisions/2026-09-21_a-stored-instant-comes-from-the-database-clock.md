answers: is it safe to bind a Rust DateTime into a TIMESTAMPTZ column; why do most timestamps here come from database_now_in_tx; when must a stored instant be read back; does the nanosecond truncation hazard exist anywhere else; should there be a gate on binding Rust instants; what makes a bound instant hazardous

# A stored instant comes from the database clock, and that is why binding it is safe

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Owner:** leaf `SIGNOFF-REPAIR.11.31.1`, adjudicating the population
  `.11.31` counted.
- **Status:** accepted.

## The question

`SIGNOFF-REPAIR.11.30` found a reservation proof carrying a Rust nanosecond that
the `TIMESTAMPTZ` column had truncated, so the ledger re-lent capacity for up to
999 ns after the proof stopped being honoured. `.11.31` then counted the
population: **43 timestamp columns are filled from a Rust-side value and not read
back**. Are any of them the same defect?

Re-derive, never read from here:

```bash
python3 -B scripts/census_bound_instants.py
python3 -B scripts/census_bound_instants.py --candidates
python3 -B scripts/census_bound_instants.py --check   # the guarded adjudication
```

⛔ Every per-site verdict below is carried as DATA in `census_bound_instants.ADJUDICATION`
and guarded, not asserted here: `--check` refuses an unjudged site, a judgement
about a site that no longer exists, and a site whose `(origin, escape)` has
moved since the judgement was made. `SIGNOFF-REPAIR.11.31.1` published these as
prose, which is how a claim about 24 sites came to be written from 8 code paths.

## The answer: none of them, and the empty cell is the reason

The hazard needs **two clocks**. A Rust variable is not a second clock if the
value in it came from the database in the first place. Crossing the bound
instant's *origin* with whether it *escapes* to a caller gives eight cells, and
the dangerous one is empty:

| origin ↓ / escape → | contained | reaches a return |
| --- | --- | --- |
| `database_clock` | 2 | 2 |
| `normalized` | 0 | 4 |
| `caller_supplied` | 19 | 5 |
| `other` | 1 | 3 |
| **`rust_clock`** | **7** | **0** |

> **No site both invents a clock value and hands it out.** That is the finding,
> and it is a property of the corpus rather than of anyone's intention.

## Why each populated cell is safe, followed to its origin

- **`database_clock` (4).** The value is `SELECT clock_timestamp()`, read back
  through `database_now_in_tx` or `tx.database_now()`. PostgreSQL has already
  truncated it to microseconds, so the Rust copy and the stored column are the
  same value by construction.
- **`normalized` (4).** `site_authority::Scope::checked()` rounds through
  `from_timestamp_micros` before anything is bound — the mitigation this hazard
  already had, working.
- **`caller_supplied` (24).** 🔴 **CORRECTED BY `SIGNOFF-REPAIR.11.31.1.1`.**
  This record first said every one of the twenty-four passes
  `database_now_in_tx(&mut tx)` or `tx.database_now()`. That was written from
  eight functions and is **false**: `api.rs:7113` calls
  `budget::create_reservation_in_tx(…, Utc::now())`, and `issue_lease`,
  `renew_lease`, `record_event_in_tx`, `check_in_tx`, `open_call`, `submit` and
  `submit_external` are in the same position.

  ⭐ **The argument never needed that claim, and is stronger without it.**
  Provenance only matters for a site whose value ESCAPES. **Nineteen of the
  twenty-four are contained**, so their caller's clock is irrelevant. Of the five
  that escape: `record_selection_in_tx`, `write_profile_in_tx` ×2 (through
  `attest_capability_in_tx`) and both `*_command_in_one_transaction` paths were
  each verified individually to the database clock; and
  `create_reservation_in_tx` is a **false positive of the escape triage** — the
  bound value is the denied branch, which returns `Err` carrying no instant,
  while the `Ok(Reservation …)` the triage matched is the active branch built
  from `RETURNING` under `.11.30`'s own repair. So `Utc::now()` does reach that
  site and still cannot escape it.
- **`rust_clock` (7).** All contained. `acknowledge_in_tx` receives `Utc::now()`
  and fills seven columns nothing reads back; `recruitment::open_call`'s `now`
  is written and never returned (its `RETURNING` clause takes `call_id` alone);
  `api.rs` binds `Utc::now()` **inline**, so there is no variable that could
  escape.
- **`other` (4).** Two are `leaf.not_after`, an X.509 validity bound, whose
  encoding is second-granular and therefore cannot carry a sub-microsecond tail.
  One echoes the caller's own request field back in the response rather than
  making a claim about the stored row. One is contained.

## ⛔ The gate is declined, and the reason is the soundness of one leg

The rule that would be correct is *an instant bound into a `TIMESTAMPTZ` column
must have database provenance, be normalized, or not escape*. The census
computes all three legs, so it looks mechanizable — and two of the three are.

**The escape leg is a triage, not an analysis.** It answers *does this
identifier appear inside a `return`, `Ok(` or `Some(` after the write, in the
same function*, which is neither sound nor complete for *the caller compares this
against the stored column*. A gate resting on it would refuse correct code with
a reason its author could not argue with, and this repository's standing rule is
that a gate people route around is a gate that lies.

The narrower rule that needs no escape analysis — *prefer `database_now_in_tx`
to `Utc::now()`* — fires on **7 of 43 correct sites**, which is the shape
rejected at `.11.9` (87%), `.11.15` (93%) and `.11.16` (66/50/71%).

⇒ **What ships is the census, run on demand, plus the convention written down.**
⚠️ **Trigger**: the first site that lands in the `(rust_clock, reaches_return)`
cell reopens the gate question with a real instance to calibrate against, which
is the calibration a rule proposed today would not have.

## ⭐ The convention, which is the thing that was missing

🔴 **CORRECTED BY `SIGNOFF-REPAIR.11.31.1.1`, which is the same overreach one
paragraph down.** This said *thirty of the forty-three sites are safe because
somebody used `database_now_in_tx`*. The guarded adjudication now puts the
numbers beyond argument: **contained 29, database_clock 6, normalized 4,
value_granularity 2, input_echo 1, triage_false_positive 1** — so six sites, not
thirty, are safe *because of the convention*, and twenty-nine are safe because
nothing reads them back.

⭐ **That does not weaken the point; it relocates it.** Six sites rely on the
convention TODAY, and every one of them is a site whose value a caller sees —
which is precisely where getting it wrong costs something, and precisely what
`.11.30` cost. **Nothing said that was why.** `database_now_in_tx`'s own doc
comment explains a different property — that `clock_timestamp()` samples after
the guard and idempotency waits, where `now()` would cache BEGIN time — and no
design document mentioned it at all.

> **An instant that will be stored, and whose value a caller may see, is sampled
> from the database clock.** It is then truncated once, by the write and the read
> being the same value, so a Rust copy of it cannot disagree with the column.
> `Utc::now()` is correct only for an instant that stays inside the process.

That sentence is the unenforced rule `.11.31` opened on, and writing it where a
reader will meet it is the repair available here — the population needs no code
change.

## ⚠️ What is not claimed

- Not that the census is complete: it reads `sqlx::query*` call sites with
  literal SQL and the schema in `migrations/`, so a dynamically built statement
  or another store is outside it. The node's SQLite journal is classified
  separately for that reason.
- Not that this host could have found it: the local clock is microsecond-granular
  (20,000 samples, zero sub-microsecond digits at `.11.30`), so no run here can
  reproduce the class. Every conclusion above is from the source, not a run.

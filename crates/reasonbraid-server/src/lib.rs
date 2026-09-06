//! reasonbraid-server — the control-plane crate (`KICKOFF.md` §3).
//!
//! WP2 (`.2.1`) lands the first capability here: a single PostgreSQL transaction that
//! atomically writes the four durability tables — current `aggregate_state`, the ordered
//! `event_log`, the `idempotency` result, and an `outbox` item — so that a successful
//! response corresponds to committed durable state, a resubmission with the same key+hash
//! returns the original result, a different hash is a conflict, and a transport redelivery
//! produces exactly one domain effect.
//!
//! The schema lives at the repository-root `migrations/` (as `KICKOFF.md` §3 sketches);
//! it is applied by the integration tests via [`sqlx::migrate!`] and by
//! `scripts/run_pg_tests.sh` / CI. The HTTP/SSE command surface and the leased outbox
//! worker are later leaves (`.2.2`, WP6).

mod tx;

pub use tx::{apply_command, ApplyError, Command, CommandOutcome};

//! `rb-server`'s boot ORDER: every declared-configuration refusal happens
//! before anything mutates (`SIGNOFF-REPAIR.11.12`).
//!
//! ⭐ The instrument is the ERROR's IDENTITY, not a database. Both controls
//! point the server at an address nothing listens on, so a boot that reaches
//! `PgPool::connect` reports `PoolTimedOut` and a boot that refuses first
//! reports the configuration it refused. One knob — which argument is wrong —
//! decides which message appears, and neither run needs PostgreSQL, a
//! migration version to read back, or a port to bind.
//!
//! ⚠️ The honest limit, stated rather than left to be discovered: this proves
//! the refusal precedes the CONNECT, which is strictly earlier than the
//! migration the leaf's finding named. It does not read a schema version back.

use std::process::Command;

/// A database URL whose host is a reserved port nothing serves.
const UNREACHABLE_DB: &str = "postgres://rb:rb@127.0.0.1:1/none";

fn boot(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_rb-server"))
        .args(args)
        .env_remove("DATABASE_URL")
        .output()
        .expect("the server binary runs");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), text)
}

#[test]
fn an_undeclared_secret_store_profile_refuses_before_the_database_is_touched() {
    let (ok, text) = boot(&[
        "--database-url",
        UNREACHABLE_DB,
        "--secret-store-profile",
        "not-a-declared-profile",
    ]);
    assert!(!ok, "an undeclared profile must refuse the boot: {text}");
    assert!(
        text.contains("the secret-store profile `not-a-declared-profile` is not declared"),
        "the refusal names the PROFILE, not a connection: {text}"
    );
    // ⛔ The load-bearing half. Before the repair this same invocation reached
    // `PgPool::connect` first and reported the unreachable database, so the
    // profile refusal arrived only after the schema had already moved.
    assert!(
        !text.contains("PoolTimedOut"),
        "the boot must not have reached the database at all: {text}"
    );
}

#[test]
fn an_unparseable_bind_address_refuses_before_the_database_is_touched() {
    let (ok, text) = boot(&["--database-url", UNREACHABLE_DB, "--host", "not a host"]);
    assert!(!ok, "an unparseable bind address must refuse: {text}");
    assert!(
        text.contains("the bind address `not a host:4310` is not a host and port"),
        "the refusal names the ARGUMENT and its value: {text}"
    );
    assert!(
        !text.contains("PoolTimedOut"),
        "the boot must not have reached the database at all: {text}"
    );
}

#[test]
fn a_declared_profile_still_reaches_the_database() {
    // The matched pair's other half: with BOTH arguments valid, the boot gets
    // as far as the connection it should. Without this, a repair that refused
    // every boot would pass the two controls above.
    let (ok, text) = boot(&["--database-url", UNREACHABLE_DB]);
    assert!(!ok, "an unreachable database still fails: {text}");
    assert!(
        !text.contains("is not declared"),
        "a declared profile is not refused: {text}"
    );
    assert!(
        text.contains("PoolTimedOut"),
        "a valid configuration still reaches the connection it should: {text}"
    );
}

//! Backup/restore status, read from the receipts the backup scripts write
//! (`SIGNOFF-REPAIR.4.6.1.5.2`; ROADMAP §18.5's ninth bullet and §17.5).
//!
//! `scripts/backup.sh` writes `<dump>.backup.json` after `pg_dump` succeeds;
//! `scripts/restore.sh` checks the dump against it, restores into an isolated
//! database, confirms the migrations came back, and only then writes
//! `<dump>.restore.json` (`scripts/backup_receipt.py` owns the format). This
//! module reads a backup directory and reports what those receipts establish,
//! re-checking what it cheaply can: every receipted dump must still exist at
//! its recorded size.
//!
//! ⛔ **§17.5: *a backup that has never been restored is not accepted as a
//! recovery control.*** The verdict is `accepted` only when at least one intact
//! backup carries a restore receipt for the SAME bytes (the restore receipt's
//! SHA-256 equals the backup's). A backup never restore-tested is listed, and
//! counts for nothing.
//!
//! ⚠️ A receipt is a file the operator's own tools wrote — evidence the
//! procedure ran, not proof against a hostile operator
//! (`docs/decisions/2026-09-22_an-incident-is-an-open-incident-review-thread-and-a-backup-is-reported-by-its-receipts.md`).

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

/// The receipt formats `scripts/backup_receipt.py` writes.
const BACKUP_FORMAT: &str = "reasonbraid-backup/1";
const RESTORE_FORMAT: &str = "reasonbraid-restore/1";
const BACKUP_SUFFIX: &str = ".backup.json";
const RESTORE_SUFFIX: &str = ".restore.json";

#[derive(Debug, Deserialize)]
struct BackupReceipt {
    receipt: String,
    dump: String,
    bytes: u64,
    sha256: String,
    taken_at: DateTime<Utc>,
    database: String,
}

#[derive(Debug, Deserialize)]
struct RestoreReceipt {
    receipt: String,
    dump: String,
    sha256: String,
    restored_at: DateTime<Utc>,
    target_database: String,
    migrations: u64,
}

/// A dump name as a receipt may state it: a plain file name, never a path.
fn plain_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != ".."
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// The report for `dir` as of `now`, or the undeclared answer when no backup
/// directory was configured. Synchronous: it reads a directory, so an async
/// caller runs it on the blocking pool.
pub fn report(dir: Option<&Path>, now: DateTime<Utc>) -> Value {
    let Some(dir) = dir else {
        return json!({
            "declared": false,
            "recovery_control": "not_accepted",
            "reason": "rb-server was started without --backup-dir, so no backup can be reported",
        });
    };
    let entries: Vec<String> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(error) => {
            return json!({
                "declared": true,
                "recovery_control": "not_accepted",
                "reason": format!("the backup directory cannot be read: {}", error.kind()),
            })
        }
    };

    let mut backups: Vec<(DateTime<Utc>, Value)> = Vec::new();
    let mut unreadable = Vec::new();
    let mut accepted = false;
    for name in entries.iter().filter(|n| n.ends_with(BACKUP_SUFFIX)) {
        let receipt: Option<BackupReceipt> = read_json(&dir.join(name));
        let Some(receipt) = receipt.filter(|r| {
            r.receipt == BACKUP_FORMAT
                && plain_name(&r.dump)
                && format!("{}{BACKUP_SUFFIX}", r.dump) == *name
        }) else {
            unreadable.push(name.clone());
            continue;
        };
        let file = match std::fs::metadata(dir.join(&receipt.dump)) {
            Err(_) => "missing",
            Ok(meta) if meta.len() != receipt.bytes => "size_mismatch",
            Ok(_) => "intact",
        };
        let restore: Option<RestoreReceipt> =
            read_json(&dir.join(format!("{}{RESTORE_SUFFIX}", receipt.dump)));
        let restore = restore.filter(|r| r.receipt == RESTORE_FORMAT && r.dump == receipt.dump);
        let restore_tested = restore.as_ref().is_some_and(|r| r.sha256 == receipt.sha256);
        accepted |= file == "intact" && restore_tested;
        backups.push((
            receipt.taken_at,
            json!({
                "dump": receipt.dump,
                "database": receipt.database,
                "taken_at": receipt.taken_at.to_rfc3339(),
                "age_ms": (now - receipt.taken_at).num_milliseconds(),
                "bytes": receipt.bytes,
                "file": file,
                "restore_test": match &restore {
                    None => json!(null),
                    Some(r) => json!({
                        "restored_at": r.restored_at.to_rfc3339(),
                        "age_ms": (now - r.restored_at).num_milliseconds(),
                        "target_database": r.target_database,
                        "migrations": r.migrations,
                        "matches_backup": r.sha256 == receipt.sha256,
                    }),
                },
            }),
        ));
    }
    backups.sort_by_key(|(taken_at, _)| std::cmp::Reverse(*taken_at));

    let receipted: std::collections::BTreeSet<&str> = backups
        .iter()
        .filter_map(|(_, b)| b["dump"].as_str())
        .collect();
    let mut unreceipted: Vec<&String> = entries
        .iter()
        .filter(|n| n.ends_with(".dump") && !receipted.contains(n.as_str()))
        .collect();
    unreceipted.sort();
    unreadable.sort();

    // The newest backup, and the newest one that counts under §17.5 — intact
    // AND restore-tested against the same bytes, the verdict's own condition.
    let newest = |counted: bool| {
        backups
            .iter()
            .find(|(_, b)| {
                !counted
                    || (b["file"] == json!("intact")
                        && b["restore_test"]["matches_backup"] == json!(true))
            })
            .and_then(|(_, b)| b["age_ms"].as_i64())
    };
    json!({
        "declared": true,
        "recovery_control": if accepted { "accepted" } else { "not_accepted" },
        "reason": if accepted {
            "at least one intact backup has passed a restore test"
        } else {
            "no intact backup has passed a restore test (§17.5: a backup that has never been restored is not a recovery control)"
        },
        "newest_backup_age_ms": newest(false),
        "newest_restore_tested_backup_age_ms": newest(true),
        "backups": backups.into_iter().map(|(_, b)| b).collect::<Vec<_>>(),
        "unreceipted_dumps": unreceipted,
        "unreadable_receipts": unreadable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasonbraid_core::fixture::Fixture;

    fn write(dir: &Path, name: &str, body: Value) {
        std::fs::write(dir.join(name), body.to_string()).unwrap();
    }

    fn backup(dir: &Path, dump: &str, bytes: &[u8], taken_at: &str, sha: &str) {
        std::fs::write(dir.join(dump), bytes).unwrap();
        write(
            dir,
            &format!("{dump}{BACKUP_SUFFIX}"),
            json!({ "receipt": BACKUP_FORMAT, "dump": dump, "bytes": bytes.len(),
                    "sha256": sha, "taken_at": taken_at, "database": "reasonbraid" }),
        );
    }

    fn restored(dir: &Path, dump: &str, sha: &str) {
        write(
            dir,
            &format!("{dump}{RESTORE_SUFFIX}"),
            json!({ "receipt": RESTORE_FORMAT, "dump": dump, "sha256": sha,
                    "restored_at": "2026-09-22T12:00:00Z", "target_database": "t",
                    "migrations": 88 }),
        );
    }

    #[test]
    fn an_undeclared_directory_is_not_a_recovery_control() {
        let body = report(None, Utc::now());
        assert_eq!(body["declared"], json!(false));
        assert_eq!(body["recovery_control"], json!("not_accepted"));
    }

    /// §17.5 exactly: an unrestored backup is listed and does not count; a
    /// restore receipt for DIFFERENT bytes does not count either; one intact,
    /// restore-tested backup does.
    #[test]
    fn only_an_intact_restore_tested_backup_is_accepted() {
        let fixture = Fixture::create("backup-report-tests", "verdict").unwrap();
        let dir = fixture.join("backups");
        std::fs::create_dir(&dir).unwrap();
        let now: DateTime<Utc> = "2026-09-22T13:00:00Z".parse().unwrap();

        backup(&dir, "a.dump", b"AAAA", "2026-09-22T10:00:00Z", "aa");
        let body = report(Some(&dir), now);
        assert_eq!(body["recovery_control"], json!("not_accepted"), "{body}");
        assert_eq!(body["backups"][0]["file"], json!("intact"));
        assert_eq!(body["backups"][0]["restore_test"], json!(null));
        assert_eq!(body["newest_backup_age_ms"], json!(3 * 3_600_000));
        assert_eq!(body["newest_restore_tested_backup_age_ms"], json!(null));

        restored(&dir, "a.dump", "not-aa");
        let body = report(Some(&dir), now);
        assert_eq!(body["recovery_control"], json!("not_accepted"), "{body}");
        assert_eq!(
            body["backups"][0]["restore_test"]["matches_backup"],
            json!(false)
        );

        restored(&dir, "a.dump", "aa");
        let body = report(Some(&dir), now);
        assert_eq!(body["recovery_control"], json!("accepted"), "{body}");
        assert_eq!(
            body["newest_restore_tested_backup_age_ms"],
            json!(3 * 3_600_000)
        );
    }

    /// A receipted dump that has shrunk or vanished is reported as such and
    /// cannot carry the verdict; a dump with no receipt, and a receipt that
    /// cannot be read, are both named rather than dropped.
    #[test]
    fn a_damaged_backup_is_named_and_does_not_count() {
        let fixture = Fixture::create("backup-report-tests", "damage").unwrap();
        let dir = fixture.join("backups");
        std::fs::create_dir(&dir).unwrap();
        let now = Utc::now();

        backup(&dir, "old.dump", b"OLD!", "2026-09-21T10:00:00Z", "o");
        restored(&dir, "old.dump", "o");
        std::fs::write(dir.join("old.dump"), b"OL").unwrap(); // truncated
        backup(&dir, "gone.dump", b"GONE", "2026-09-22T10:00:00Z", "g");
        std::fs::remove_file(dir.join("gone.dump")).unwrap();
        std::fs::write(dir.join("partial.dump"), b"PG").unwrap(); // a failed pg_dump
        std::fs::write(dir.join("junk.dump.backup.json"), b"{").unwrap();

        let body = report(Some(&dir), now);
        assert_eq!(body["recovery_control"], json!("not_accepted"), "{body}");
        let files: Vec<(&str, &str)> = body["backups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| (b["dump"].as_str().unwrap(), b["file"].as_str().unwrap()))
            .collect();
        assert_eq!(
            files,
            [("gone.dump", "missing"), ("old.dump", "size_mismatch")]
        );
        assert_eq!(body["unreceipted_dumps"], json!(["partial.dump"]));
        assert_eq!(
            body["unreadable_receipts"],
            json!(["junk.dump.backup.json"])
        );
    }

    /// A receipt cannot point outside the directory it sits in.
    #[test]
    fn a_receipt_naming_a_path_is_unreadable() {
        let fixture = Fixture::create("backup-report-tests", "path").unwrap();
        let dir = fixture.join("backups");
        std::fs::create_dir(&dir).unwrap();
        write(
            &dir,
            "x.dump.backup.json",
            json!({ "receipt": BACKUP_FORMAT, "dump": "../x.dump", "bytes": 1,
                    "sha256": "s", "taken_at": "2026-09-22T10:00:00Z", "database": "d" }),
        );
        let body = report(Some(&dir), Utc::now());
        assert_eq!(body["unreadable_receipts"], json!(["x.dump.backup.json"]));
        assert_eq!(body["backups"], json!([]));
    }
}

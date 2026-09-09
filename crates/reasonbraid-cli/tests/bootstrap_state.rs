//! Matched durable recovery-record controls, with no network or off-volume data.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use reasonbraid_cli::StateFile;
use serde_json::{json, Value};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let root = cwd
            .ancestors()
            .find(|p| p.join("crates/reasonbraid-cli/Cargo.toml").is_file())
            .unwrap();
        let device = std::fs::metadata(root).unwrap().dev();
        let target = root.join("target");
        let meta = std::fs::symlink_metadata(&target).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        let base = target.join("cli-bootstrap-controls");
        match std::fs::create_dir(&base) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => panic!("owned recovery fixture base: {error}"),
        }
        let meta = std::fs::symlink_metadata(&base).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        let path = base.join(format!("record-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if std::thread::panicking() {
                eprintln!("owned recovery fixture cleanup failed: {error}");
            } else {
                panic!("owned recovery fixture cleanup failed: {error}");
            }
        }
    }
}

fn request() -> Value {
    json!({
        "request_id":"req_00000000-0000-7000-8000-000000000001",
        "server":"http://127.0.0.1:4310", "name":"alice", "actions":null
    })
}

fn outcome() -> Value {
    json!({
        "bootstrap_request_id":"req_00000000-0000-7000-8000-000000000001",
        "kind":"human", "name":"alice",
        "principal_id":"hpr_00000000-0000-7000-8000-000000000001",
        "tenant_id":"ten_00000000-0000-7000-8000-000000000001",
        "boundary_id":"bnd_ten_00000000-0000-7000-8000-000000000001",
        "grant_id":"grt_hpr_00000000-0000-7000-8000-000000000001",
        "replayed":false
    })
}

fn snapshot(completed: bool) -> Value {
    let recovery = if completed {
        json!({"pending":null,"completed":{"request":request(),"outcome":outcome()}})
    } else {
        json!({"pending":request(),"completed":null})
    };
    json!({"version":2,"principals":{},"threads":{},"bootstrap":recovery})
}

#[test]
fn recovery_snapshots_preserve_pending_and_completed_request_identity() {
    let mut observations = Vec::new();
    for completed in [false, true] {
        let fixture = Fixture::new();
        let dir = fixture.path();
        std::fs::create_dir(&dir).unwrap();
        let expected = snapshot(completed);
        let raw = serde_json::to_vec_pretty(&expected).unwrap();
        std::fs::write(dir.join("state.json"), &raw).unwrap();
        let loaded = StateFile::load(&dir);
        let roundtrip = loaded.as_ref().is_ok_and(|state| {
            serde_json::to_value(state).unwrap() == expected
                && state.save(&dir).is_ok()
                && serde_json::to_value(StateFile::load(&dir).unwrap()).unwrap() == expected
        });
        eprintln!(
            "recovery record: completed={completed}, loaded={}, preserved={roundtrip}",
            loaded.is_ok()
        );
        observations.push((loaded.is_ok(), roundtrip));
        if loaded.is_err() {
            assert_eq!(std::fs::read(dir.join("state.json")).unwrap(), raw);
        }
    }
    assert!(observations
        .iter()
        .all(|(loaded, preserved)| *loaded && *preserved));
}

#[test]
fn malformed_recovery_records_are_refused_and_preserved() {
    let mut cases = Vec::new();
    for (pointer, bad) in [
        ("/version", json!(1)),
        ("/version", json!(3)),
        ("/bootstrap", Value::Null),
        ("/bootstrap", json!({"pending":null,"completed":null})),
        ("/bootstrap", json!({"pending":request()})),
        ("/bootstrap", json!({"completed":null})),
        ("/bootstrap", json!([request(), null])),
        (
            "/bootstrap/pending",
            json!([request()["request_id"], request()["server"], "alice", null]),
        ),
        (
            "/bootstrap/pending/request_id",
            json!("req_00000000-0000-4000-8000-000000000001"),
        ),
        (
            "/bootstrap/pending/request_id",
            json!("req_00000000-0000-7000-c000-000000000001"),
        ),
        (
            "/bootstrap/pending/request_id",
            json!("req_00000000-0000-7000-8000-00000000000A"),
        ),
        (
            "/bootstrap/pending/request_id",
            json!("ten_00000000-0000-7000-8000-000000000001"),
        ),
        ("/bootstrap/pending/actions", json!(true)),
        ("/bootstrap/pending/server", json!("ftp://example.test")),
        (
            "/bootstrap/pending/server",
            json!("http://example.test?query=secret"),
        ),
        (
            "/bootstrap/pending/server",
            json!("http://example.test#fragment"),
        ),
        (
            "/bootstrap/pending/server",
            json!("http://user:SECRET@127.0.0.1:4310"),
        ),
        (
            "/bootstrap/pending/server",
            json!("HTTP://EXAMPLE.TEST:80/"),
        ),
        ("/bootstrap/pending/server", json!("http://127.0.0.1:4310/")),
        ("/bootstrap/pending/server", json!(" http://example.test")),
        (
            "/bootstrap/pending/server",
            json!(format!("http://example.test/{}", "x".repeat(4096))),
        ),
    ] {
        let mut value = snapshot(false);
        *value.pointer_mut(pointer).unwrap() = bad;
        cases.push(serde_json::to_vec(&value).unwrap());
    }
    for field in ["request_id", "server", "name", "actions"] {
        let mut value = snapshot(false);
        value["bootstrap"]["pending"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push(serde_json::to_vec(&value).unwrap());
    }
    for field in outcome().as_object().unwrap().keys() {
        let mut value = snapshot(true);
        value["bootstrap"]["completed"]["outcome"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push(serde_json::to_vec(&value).unwrap());
    }
    for (pointer, bad) in [
        ("/bootstrap/completed", json!([request(), outcome()])),
        (
            "/bootstrap/completed/request",
            json!([request()["request_id"], request()["server"], "alice", null]),
        ),
        (
            "/bootstrap/completed/outcome",
            json!([
                outcome()["bootstrap_request_id"],
                "human",
                "alice",
                outcome()["principal_id"],
                outcome()["tenant_id"],
                outcome()["boundary_id"],
                outcome()["grant_id"],
                false
            ]),
        ),
        (
            "/bootstrap/completed/outcome/bootstrap_request_id",
            json!("req_00000000-0000-7000-8000-000000000002"),
        ),
        ("/bootstrap/completed/outcome/kind", json!("role")),
        ("/bootstrap/completed/outcome/name", json!("different")),
        (
            "/bootstrap/completed/outcome/principal_id",
            json!("rol_00000000-0000-7000-8000-000000000001"),
        ),
        (
            "/bootstrap/completed/outcome/tenant_id",
            json!("ten_invalid"),
        ),
        (
            "/bootstrap/completed/outcome/boundary_id",
            json!("bnd_different"),
        ),
        (
            "/bootstrap/completed/outcome/grant_id",
            json!("grt_different"),
        ),
        ("/bootstrap/completed/outcome/replayed", json!("false")),
    ] {
        let mut value = snapshot(true);
        *value.pointer_mut(pointer).unwrap() = bad;
        cases.push(serde_json::to_vec(&value).unwrap());
    }
    for pointer in [
        "/bootstrap",
        "/bootstrap/completed",
        "/bootstrap/completed/request",
        "/bootstrap/completed/outcome",
    ] {
        let mut value = snapshot(true);
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(1));
        cases.push(serde_json::to_vec(&value).unwrap());
    }
    let mut conflict = snapshot(true);
    conflict["bootstrap"]["pending"] = request();
    conflict["bootstrap"]["pending"]["name"] = json!("conflicting binding");
    cases.push(serde_json::to_vec(&conflict).unwrap());
    let raw = serde_json::to_string(&snapshot(false)).unwrap();
    cases.push(
        raw.replacen(
            "\"name\":\"alice\"",
            "\"name\":\"alice\",\"name\":\"alice\"",
            1,
        )
        .into_bytes(),
    );
    cases.push(
        raw.replacen(
            "\"completed\":null",
            "\"completed\":null,\"completed\":null",
            1,
        )
        .into_bytes(),
    );
    cases.push(
        serde_json::to_vec(&json!({"version":1,"principals":{},"threads":{},"bootstrap":null}))
            .unwrap(),
    );
    let mut absent_pending = snapshot(true);
    absent_pending["bootstrap"]
        .as_object_mut()
        .unwrap()
        .remove("pending");
    cases.push(serde_json::to_vec(&absent_pending).unwrap());
    let count = cases.len();
    for (index, raw) in cases.into_iter().enumerate() {
        let fixture = Fixture::new();
        let dir = fixture.path();
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("state.json"), &raw).unwrap();
        let error = StateFile::load(&dir).unwrap_err();
        assert!(!error.to_string().contains("SECRET"), "case {index}");
        assert!(StateFile::default().save(&dir).is_err(), "case {index}");
        assert_eq!(
            std::fs::read(dir.join("state.json")).unwrap(),
            raw,
            "case {index}"
        );
    }
    eprintln!("malformed recovery cases preserved: {count}");
}

#[test]
fn publication_preserves_pending_identity_and_requires_completed_principal_before_cleanup() {
    let fixture = Fixture::new();
    let dir = fixture.path();
    let mut current: StateFile = serde_json::from_value(snapshot(false)).unwrap();
    current.save(&dir).unwrap();
    let original = std::fs::read(dir.join("state.json")).unwrap();
    let mut conflicting = current.clone();
    conflicting
        .bootstrap
        .as_mut()
        .unwrap()
        .pending
        .as_mut()
        .unwrap()
        .request_id = "req_00000000-0000-7000-8000-000000000002".into();
    assert!(conflicting.save(&dir).is_err());
    assert!(StateFile::default().save(&dir).is_err());
    assert_eq!(std::fs::read(dir.join("state.json")).unwrap(), original);
    let done = serde_json::from_value(snapshot(true)).unwrap();
    // A complete response alone cannot retire the pending request before its
    // local principal mapping is published in that same complete snapshot.
    let missing_principal: StateFile = done;
    assert!(missing_principal.save(&dir).is_err());
    current.bootstrap.as_mut().unwrap().completed = missing_principal.bootstrap.unwrap().completed;
    assert!(current.save(&dir).is_err());
    current.principals.insert(
        "alice".into(),
        reasonbraid_cli::StoredPrincipal {
            kind: "human".into(),
            id: outcome()["principal_id"].as_str().unwrap().into(),
            tenant: outcome()["tenant_id"].as_str().unwrap().into(),
        },
    );
    current.save(&dir).unwrap();
    let before_cleanup = StateFile::load(&dir).unwrap();
    assert!(before_cleanup.bootstrap.as_ref().unwrap().pending.is_some());
    current.bootstrap.as_mut().unwrap().pending = None;
    current.save(&dir).unwrap();
    let completed = serde_json::to_value(StateFile::load(&dir).unwrap()).unwrap();
    assert_eq!(completed["bootstrap"]["completed"]["request"], request());
    assert!(completed["bootstrap"]["pending"].is_null());
    assert!(
        before_cleanup.save(&dir).is_ok(),
        "explicit recovery may reactivate the same retained request"
    );
    current.save(&dir).unwrap();
    let mut stale = current.clone();
    stale.bootstrap = None;
    stale.version = 1;
    assert!(stale.save(&dir).is_err());
    assert_eq!(
        serde_json::to_value(StateFile::load(&dir).unwrap()).unwrap(),
        completed
    );
}

#[test]
fn legacy_wire_shape_and_canonical_historical_receipts_remain_supported() {
    for version in [0, 1] {
        let fixture = Fixture::new();
        let state = StateFile {
            version,
            ..StateFile::default()
        };
        let expected = json!({"version":version,"principals":{},"threads":{}});
        assert_eq!(serde_json::to_value(&state).unwrap(), expected);
        state.save(&fixture.path()).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(
                &std::fs::read(fixture.path().join("state.json")).unwrap()
            )
            .unwrap(),
            expected
        );
    }
    for server in [
        "https://example.test",
        "http://[::1]:4310/api",
        "http://127.0.0.1:4310",
    ] {
        let fixture = Fixture::new();
        let mut value = snapshot(true);
        value["bootstrap"]["completed"]["outcome"]["replayed"] = json!(true);
        let mut pending = request();
        pending["request_id"] = json!("req_00000000-0000-7000-8000-000000000002");
        pending["name"] = json!("new exact name");
        pending["server"] = json!(server);
        pending["actions"] = json!(["ignored human input"]);
        value["bootstrap"]["pending"] = pending;
        // Canonical older UUID identities remain valid historical source IDs;
        // only the client request identity is specifically required to be v7.
        for field in ["principal_id", "tenant_id", "boundary_id", "grant_id"] {
            let old = value["bootstrap"]["completed"]["outcome"][field]
                .as_str()
                .unwrap()
                .replace("-7000-", "-4000-");
            value["bootstrap"]["completed"]["outcome"][field] = json!(old);
        }
        let state: StateFile = serde_json::from_value(value.clone()).unwrap();
        state.save(&fixture.path()).unwrap();
        assert_eq!(
            serde_json::to_value(StateFile::load(&fixture.path()).unwrap()).unwrap(),
            value
        );
    }
}

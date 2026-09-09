//! Matched local-state publication controls; no HTTP or off-volume fixture data.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::io::Read;
use std::os::unix::fs::{symlink, MetadataExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use reasonbraid_cli::{StateFile, StoredPrincipal};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::timeout;
use uuid::Uuid;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let base = root.join("target/cli-state-controls");
        std::fs::create_dir_all(&base).unwrap();
        let meta = std::fs::symlink_metadata(&base).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), std::fs::metadata(&root).unwrap().dev());
        let path = base.join(format!("state-{}", Uuid::now_v7()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn state_dir(&self) -> PathBuf {
        self.0.join("store")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // This exact unique workspace was created by this fixture, never shared.
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn state(name: &str) -> StateFile {
    let mut state = StateFile {
        version: 1,
        ..StateFile::default()
    };
    state.principals.insert(
        name.into(),
        StoredPrincipal {
            kind: "human".into(),
            id: "hpr_00000000-0000-7000-8000-000000000001".into(),
            tenant: "ten_00000000-0000-7000-8000-000000000001".into(),
        },
    );
    state
}

#[test]
fn an_open_reader_retains_its_original_snapshot_after_replacement() {
    let f = Fixture::new();
    let dir = f.state_dir();
    state("original").save(&dir).unwrap();
    let original = std::fs::read(dir.join("state.json")).unwrap();
    let mut reader = std::fs::File::open(dir.join("state.json")).unwrap();
    state("replacement").save(&dir).unwrap();
    let mut observed = Vec::new();
    reader.read_to_end(&mut observed).unwrap();
    eprintln!(
        "publication baseline: old reader retains snapshot={}",
        observed == original
    );
    assert_eq!(
        observed, original,
        "an already-open reader must never see a replacement's bytes"
    );
    let loaded = StateFile::load(&dir).unwrap();
    assert!(loaded.principals.contains_key("replacement"));
    assert!(!loaded.principals.contains_key("original"));
}

#[test]
fn state_file_links_refuse_without_modifying_the_owned_target() {
    let mut results = Vec::new();
    for hard in [false, true] {
        let f = Fixture::new();
        let dir = f.state_dir();
        std::fs::create_dir(&dir).unwrap();
        let target = f.0.join("owned-original.json");
        let original = serde_json::to_vec_pretty(&state("original")).unwrap();
        std::fs::write(&target, &original).unwrap();
        if hard {
            std::fs::hard_link(&target, dir.join("state.json")).unwrap();
        } else {
            symlink(&target, dir.join("state.json")).unwrap();
        }
        let saved = state("replacement").save(&dir);
        let unchanged = std::fs::read(&target).unwrap() == original;
        results.push((hard, saved.is_err(), unchanged));
    }
    eprintln!("link baseline (hard link, refused, target unchanged): {results:?}");
    assert!(results
        .iter()
        .all(|(_, refused, unchanged)| *refused && *unchanged));
}

#[test]
fn reading_a_missing_state_does_not_create_storage() {
    let f = Fixture::new();
    let dir = f.state_dir();
    let loaded = StateFile::load(&dir).unwrap();
    assert_eq!(loaded.version, 0);
    assert!(loaded.principals.is_empty() && loaded.threads.is_empty());
    assert!(!dir.exists());
}

#[tokio::test]
async fn another_process_excludes_publication_and_crash_releases_its_lock() {
    let f = Fixture::new();
    let dir = f.state_dir();
    state("original").save(&dir).unwrap();
    let original = std::fs::read(dir.join("state.json")).unwrap();
    // Fixed installed Python/OS fcntl are read-only tool dependencies. The sole
    // opened/created data file is the exact owned repo-volume lock path.
    let mut holder = tokio::process::Command::new("python3")
        .args(["-B", "-c", "import fcntl,os,sys; fd=os.open(sys.argv[1],os.O_RDWR|os.O_CREAT,0o600); fcntl.flock(fd,fcntl.LOCK_EX); print('locked',flush=True); sys.stdin.buffer.read(1)"])
        .arg(dir.join("state.lock"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .kill_on_drop(true).spawn().unwrap();
    let mut output = BufReader::new(holder.stdout.take().unwrap());
    let mut ready = String::new();
    let readiness = timeout(Duration::from_secs(10), output.read_line(&mut ready)).await;
    if !matches!(readiness, Ok(Ok(_))) || ready.trim() != "locked" {
        let _ = holder.start_kill();
        let _ = holder.wait().await;
        panic!("owned lock process did not become ready: {readiness:?}, {ready:?}");
    }
    let save_dir = dir.clone();
    let mut writer = tokio::task::spawn_blocking(move || state("replacement").save(&save_dir));
    let attempted = timeout(Duration::from_secs(1), &mut writer).await;
    let prompt = attempted.is_ok();
    // Consume the independent holder on every result, including an incorrectly
    // blocking publication. SIGKILL proves release by process lifetime.
    holder.start_kill().unwrap();
    let status = holder.wait().await.unwrap();
    assert!(!status.success());
    let result = match attempted {
        Ok(result) => result.unwrap(),
        Err(_) => timeout(Duration::from_secs(10), writer)
            .await
            .unwrap()
            .unwrap(),
    };
    let unchanged = std::fs::read(dir.join("state.json")).unwrap() == original;
    eprintln!(
        "process lock baseline: prompt={prompt}, refused={}, state unchanged={unchanged}",
        result.is_err()
    );
    assert!(prompt && result.is_err() && unchanged);
    state("replacement").save(&dir).unwrap();
    assert!(StateFile::load(&dir)
        .unwrap()
        .principals
        .contains_key("replacement"));
}

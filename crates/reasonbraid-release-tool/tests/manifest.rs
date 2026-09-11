//! The release-manifest proof (`PHASE-7.2.3`, ADR-027): the roundtrip over
//! the BUILT binary — the keygen, the generate, the verify, and the refusals
//! (a changed binary, a tampered manifest, a wrong key). OFFLINE — no
//! database, the subprocess boundary is the real tool.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tool() -> &'static str {
    env!("CARGO_BIN_EXE_rb-release-manifest")
}

/// A per-test control directory on the REPOSITORY's own volume (§13: project
/// data never lands in an ambient temporary directory), created exclusively so
/// an existing path is refused rather than adopted. The process id plus a
/// counter is already unique; the creation now proves it.
fn temp_dir() -> PathBuf {
    let parent =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release-tool-controls");
    std::fs::create_dir_all(&parent).expect("create the control parent");
    let dir = parent.join(format!(
        "rb-release-tool-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the control directory is new");
    dir
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(tool())
        .args(args)
        .output()
        .expect("the tool runs")
}

#[test]
fn the_manifest_signs_and_the_refusals_are_typed() {
    let dir = temp_dir();
    let key = dir.join("release-key.pk8");
    let manifest = dir.join("release-manifest.json");
    let sig = dir.join("release-manifest.json.sig");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("the bin dir");
    let bin_a = bin_dir.join("rb");
    let bin_b = bin_dir.join("rb-server");
    std::fs::write(&bin_a, b"binary-a-content").expect("write bin a");
    std::fs::write(&bin_b, b"binary-b-content").expect("write bin b");

    // 1. The keygen (the key exists; the tool refuses to overwrite it).
    let out = run(&["keygen", "--key", key.to_str().unwrap()]);
    assert!(out.status.success(), "the keygen: {:?}", out);
    assert!(key.exists(), "the key file lands");
    let out = run(&["keygen", "--key", key.to_str().unwrap()]);
    assert!(!out.status.success(), "the overwrite refuses");

    // 2. The generate: the manifest + the signature land; the manifest
    //    carries the ADR-011 digest scheme + the sorted binaries map.
    let out = run(&[
        "generate",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--bin",
        "rb",
        "--bin",
        "rb-server",
        "--out",
        manifest.to_str().unwrap(),
        "--release-name",
        "0.1.0",
    ]);
    assert!(out.status.success(), "the generate: {:?}", out);
    let manifest_text = std::fs::read_to_string(&manifest).expect("the manifest reads");
    let parsed: serde_json::Value = serde_json::from_str(&manifest_text).expect("the json");
    assert_eq!(parsed["release"], serde_json::json!("0.1.0"));
    use sha2::Digest;
    let expected_a = format!(
        "sha256:{}",
        sha2::Sha256::digest(b"binary-a-content")
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_eq!(parsed["binaries"]["rb"], serde_json::json!(expected_a));
    assert!(
        parsed["binaries"]["rb-server"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"),
        "the digest scheme"
    );
    assert!(sig.exists(), "the signature lands");

    // 3. The verify passes (the signature + the digests).
    let out = run(&[
        "verify",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "the verify: {:?}", out);

    // 4. A changed binary refuses (the re-derivation catches it).
    std::fs::write(&bin_a, b"tampered-binary-content").expect("tamper bin a");
    let out = run(&[
        "verify",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "the changed binary refuses");

    // 5. The restored binary verifies again; a tampered MANIFEST refuses
    //    (the signature over the exact bytes).
    std::fs::write(&bin_a, b"binary-a-content").expect("restore bin a");
    let manifest_bytes = std::fs::read(&manifest).expect("the manifest bytes");
    std::fs::write(&manifest, &manifest_bytes).expect("no-op write");
    let out = run(&[
        "verify",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "the restored verify: {:?}", out);
    let mut tampered = manifest_bytes.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    std::fs::write(&manifest, &tampered).expect("tamper the manifest");
    let out = run(&[
        "verify",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "the tampered manifest refuses");
    std::fs::write(&manifest, &manifest_bytes).expect("restore the manifest");

    // 6. A WRONG key refuses (a second identity signs nothing here).
    let other_key = dir.join("other-key.pk8");
    let out = run(&["keygen", "--key", other_key.to_str().unwrap()]);
    assert!(out.status.success(), "the second keygen: {:?}", out);
    let out = run(&[
        "verify",
        "--key",
        other_key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "the wrong key refuses");

    std::fs::remove_dir_all(&dir).ok();
}

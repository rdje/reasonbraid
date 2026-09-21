//! The release-manifest proof (`PHASE-7.2.3`, ADR-027): the roundtrip over
//! the BUILT binary — the keygen, the generate, the verify, and the refusals
//! (a changed binary, a tampered manifest, a wrong key). OFFLINE — no
//! database, the subprocess boundary is the real tool.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use reasonbraid_core::fixture::Fixture;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tool() -> &'static str {
    env!("CARGO_BIN_EXE_rb-release-manifest")
}

/// A per-test control directory on the REPOSITORY's own volume (§13: project
/// data never lands in an ambient temporary directory), created exclusively so
/// an existing path is refused rather than adopted. The process id plus a
/// counter is already unique; the creation now proves it.
/// A control directory, created exclusively on the repository's own volume and
/// REMOVED when the control passes (`SIGNOFF-REPAIR.11.2.1.3.2.6`). A failing
/// control keeps its keys, manifests and signatures as the diagnostic.
///
/// ⚠️ The guard must be BOUND to a local: as a temporary it would be dropped at
/// the end of its statement, taking the control's files with it.
fn temp_fixture() -> Fixture {
    Fixture::create(
        "release-tool-controls",
        &format!(
            "rb-release-tool-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ),
    )
    .expect("the control directory is new")
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(tool())
        .args(args)
        .output()
        .expect("the tool runs")
}

#[test]
fn the_manifest_signs_and_the_refusals_are_typed() {
    let fixture = temp_fixture();
    let dir = fixture.path();
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
}

/// 🔴 **THE SIGNING-KEY INCIDENT'S RECOVERY, DRIVEN END TO END**
/// (`SIGNOFF-REPAIR.11.24.1.4`). `docs/runbooks/signing-key-incident.md` says
/// *re-sign the SAME manifest content (the digests unchanged)*, and until
/// `re-sign` existed no command could do it.
///
/// ⛔ **And `generate` could not stand in for it, which this control measures
/// FIRST rather than asserting.** Two `generate` runs over an UNCHANGED binary
/// produce different manifests, because `created_at` is `Utc::now()` — so a
/// re-key performed that way publishes a new manifest rather than the same one
/// under a new identity. That leg is the reason the verb exists; without it a
/// reader would take the new command for a convenience.
#[test]
fn the_manifest_re_signs_under_a_new_identity_and_the_old_one_stops_verifying() {
    let fixture = temp_fixture();
    let dir = fixture.path();
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("the bin dir");
    std::fs::write(bin_dir.join("rb"), b"binary-a-content").expect("write bin");
    let old_key = dir.join("compromised.pk8");
    let new_key = dir.join("recovered.pk8");
    let manifest = dir.join("release-manifest.json");
    let sig = dir.join("release-manifest.json.sig");
    let new_sig = dir.join("release-manifest.json.sig.rekeyed");

    assert!(
        run(&["keygen", "--key", old_key.to_str().unwrap()])
            .status
            .success(),
        "the compromised identity"
    );
    let generate = |out: &std::path::Path| {
        let out = run(&[
            "generate",
            "--key",
            old_key.to_str().unwrap(),
            "--bin-dir",
            bin_dir.to_str().unwrap(),
            "--bin",
            "rb",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert!(out.status.success(), "the generate: {out:?}");
    };
    generate(&manifest);
    let original = std::fs::read(&manifest).expect("the manifest bytes");

    // LEG 0 — `generate` is NOT a re-signature, even over an unchanged binary.
    let second = dir.join("second-manifest.json");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    generate(&second);
    assert_ne!(
        std::fs::read(&second).expect("the second manifest"),
        original,
        "leg 0: two generates over one unchanged binary differ — `created_at` is now()"
    );

    // LEG 1 — the same key cannot perform the recovery, and says so instead of
    // writing a signature identical to the one already there.
    let refusal = run(&[
        "re-sign",
        "--key",
        old_key.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        new_sig.to_str().unwrap(),
    ]);
    assert!(!refusal.status.success(), "leg 1: the same key is refused");
    assert!(
        String::from_utf8_lossy(&refusal.stderr).contains("no re-key happened"),
        "leg 1: and the refusal names why: {refusal:?}"
    );
    assert!(!new_sig.exists(), "leg 1: a refused re-sign writes nothing");

    // LEG 2 — the recovery itself.
    assert!(
        run(&["keygen", "--key", new_key.to_str().unwrap()])
            .status
            .success(),
        "the recovered identity"
    );
    let out = run(&[
        "re-sign",
        "--key",
        new_key.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        new_sig.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "leg 2: the re-sign: {out:?}");

    // ⭐ THE PROPERTY THE WHOLE VERB IS FOR: the manifest did not change.
    assert_eq!(
        std::fs::read(&manifest).expect("the manifest after"),
        original,
        "leg 2: the manifest's bytes are untouched — the digests are unchanged"
    );
    // And the old signature survives: it is the incident's evidence.
    assert!(
        sig.exists(),
        "leg 2: the original signature is not overwritten"
    );

    let verify = |key: &std::path::Path, signature: &std::path::Path| {
        run(&[
            "verify",
            "--key",
            key.to_str().unwrap(),
            "--bin-dir",
            bin_dir.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--sig",
            signature.to_str().unwrap(),
        ])
        .status
        .success()
    };
    assert!(
        verify(&new_key, &new_sig),
        "leg 2: the manifest verifies under the recovered identity"
    );
    // ⛔ THE TWO NEGATIVE LEGS, and both are needed. The first says the new
    // signature belongs to the new key alone; the second says the OLD identity
    // has not somehow inherited it.
    assert!(
        !verify(&old_key, &new_sig),
        "leg 3: the compromised identity does not verify the new signature"
    );
    assert!(
        !verify(&new_key, &sig),
        "leg 3: nor does the recovered identity verify the old one"
    );
    // ⭐ POSITIVE CONTROL in the same run: the old pairing still verifies, so
    // the two refusals above are about the identities and not about a broken
    // fixture.
    assert!(
        verify(&old_key, &sig),
        "leg 3: the original key still verifies the original signature"
    );

    // LEG 4 — an existing signature path is never overwritten, and a file that
    // is not a manifest is never signed as one.
    let again = run(&[
        "re-sign",
        "--key",
        new_key.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        new_sig.to_str().unwrap(),
    ]);
    assert!(!again.status.success(), "leg 4: the overwrite refuses");
    let junk = dir.join("not-a-manifest.json");
    std::fs::write(&junk, b"{\"hello\":\"world\"}").expect("write the junk");
    let junk_sig = dir.join("junk.sig");
    let out = run(&[
        "re-sign",
        "--key",
        new_key.to_str().unwrap(),
        "--manifest",
        junk.to_str().unwrap(),
        "--sig",
        junk_sig.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "leg 4: a non-manifest is refused");
    assert!(!junk_sig.exists(), "leg 4: and nothing is signed");

    // LEG 5 — 🔴 **THE LEG THAT MAKES "VERBATIM" MEAN SOMETHING.** Every
    // manifest above round-trips through serde to its own bytes, so a `re-sign`
    // that signed `canonical_bytes(&parsed)` instead of the file would pass all
    // of them — the control would be measuring nothing about the property the
    // verb is named for. A PRETTY-PRINTED manifest parses to the same struct
    // and serializes to DIFFERENT bytes, which is the case that separates them:
    // `verify` checks the signature against the file's own bytes, so a
    // re-serializing implementation fails here and only here.
    let pretty_dir = dir.join("pretty");
    std::fs::create_dir_all(&pretty_dir).expect("the pretty dir");
    let pretty = pretty_dir.join("release-manifest.json");
    let parsed: serde_json::Value =
        serde_json::from_slice(&original).expect("the original manifest parses");
    let pretty_bytes = serde_json::to_vec_pretty(&parsed).expect("the pretty bytes");
    assert_ne!(
        pretty_bytes, original,
        "leg 5: the fixture is only meaningful if the bytes actually differ"
    );
    std::fs::write(&pretty, &pretty_bytes).expect("write the pretty manifest");
    let pretty_sig = pretty_dir.join("release-manifest.json.sig");
    let out = run(&[
        "re-sign",
        "--key",
        new_key.to_str().unwrap(),
        "--manifest",
        pretty.to_str().unwrap(),
        "--sig",
        pretty_sig.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "leg 5: the re-sign: {out:?}");
    assert_eq!(
        std::fs::read(&pretty).expect("the pretty manifest after"),
        pretty_bytes,
        "leg 5: the file is still the file"
    );
    let out = run(&[
        "verify",
        "--key",
        new_key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        pretty.to_str().unwrap(),
        "--sig",
        pretty_sig.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "leg 5: the signature is over the FILE's bytes, not a re-serialization: {out:?}"
    );
}

/// 🔴 **A SIGNATURE WHOSE VERIFIER MUST HOLD THE SIGNING KEY PROVES NOTHING TO
/// ANYBODY ELSE** (`SIGNOFF-REPAIR.11.24.1.4.1`). Until `pubkey` existed,
/// `verify` derived the public key from the PRIVATE one, so the only party who
/// could check a release was the party who signed it — and what that actually
/// caught was accidental corruption, not forgery.
///
/// ⭐ **The load-bearing leg is the one that proves the private key is NOT
/// read**, and it proves it structurally rather than by inspection: the
/// verification runs in a directory that contains the manifest, the signature,
/// the binaries and the exported public key, and **no private key at all**. A
/// `--public-key` path that silently still reached for `release-key.pk8` would
/// be the same defect with a new flag, and that is exactly what this refuses to
/// let happen.
#[test]
fn a_manifest_verifies_from_the_public_key_alone() {
    let fixture = temp_fixture();
    let dir = fixture.path();
    let signed = dir.join("signed");
    let published = dir.join("published");
    std::fs::create_dir_all(&signed).expect("the signing dir");
    std::fs::create_dir_all(&published).expect("the publishing dir");
    let bin_dir = published.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("the bin dir");
    std::fs::write(bin_dir.join("rb"), b"binary-a-content").expect("write bin");

    let key = signed.join("release-key.pk8");
    let manifest = published.join("release-manifest.json");
    let sig = published.join("release-manifest.json.sig");
    let pubkey = published.join("release-identity.pub");

    assert!(run(&["keygen", "--key", key.to_str().unwrap()])
        .status
        .success());
    let out = run(&[
        "generate",
        "--key",
        key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--bin",
        "rb",
        "--out",
        manifest.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "the generate: {out:?}");

    // The export, and its shape.
    let out = run(&[
        "pubkey",
        "--key",
        key.to_str().unwrap(),
        "--out",
        pubkey.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "the pubkey export: {out:?}");
    let published_key = std::fs::read_to_string(&pubkey).expect("the public key reads");
    assert_eq!(
        published_key.len(),
        64,
        "an Ed25519 public key is 32 bytes = 64 hex digits"
    );
    assert!(
        published_key.chars().all(|c| c.is_ascii_hexdigit()),
        "and it is hex: {published_key}"
    );
    // Never overwritten — a published identity replaced in place is the failure.
    assert!(
        !run(&[
            "pubkey",
            "--key",
            key.to_str().unwrap(),
            "--out",
            pubkey.to_str().unwrap()
        ])
        .status
        .success(),
        "the export refuses to overwrite"
    );

    let verify_public = |pub_path: &std::path::Path| {
        run(&[
            "verify",
            "--public-key",
            pub_path.to_str().unwrap(),
            "--bin-dir",
            bin_dir.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--sig",
            sig.to_str().unwrap(),
        ])
    };

    // ⭐ THE LEG THIS TEST EXISTS FOR. Everything the verifier needs is in
    // `published/`; the private key is in `signed/` and is about to stop
    // existing entirely, so a verification that still reached for it cannot
    // pass by accident.
    std::fs::remove_file(&key).expect("the private key is removed");
    assert!(!key.exists(), "the verifier's world holds no private key");
    let out = verify_public(&pubkey);
    assert!(
        out.status.success(),
        "the manifest verifies from the public key alone: {out:?}"
    );

    // ⛔ THE NEGATIVE SIDE: a DIFFERENT identity's public key refuses. Without
    // it the leg above passes against a `verify` that checks nothing.
    let other_key = dir.join("other.pk8");
    let other_pub = dir.join("other.pub");
    assert!(run(&["keygen", "--key", other_key.to_str().unwrap()])
        .status
        .success());
    assert!(run(&[
        "pubkey",
        "--key",
        other_key.to_str().unwrap(),
        "--out",
        other_pub.to_str().unwrap()
    ])
    .status
    .success());
    assert!(
        !verify_public(&other_pub).status.success(),
        "another identity's public key does not verify this manifest"
    );

    // ⛔ A public key of the wrong SHAPE is a caller error and is named as one,
    // rather than travelling into ring and coming back as "does not verify" —
    // which would blame the signature for a mistyped path.
    let short = dir.join("short.pub");
    std::fs::write(&short, b"deadbeef").expect("write the short key");
    let out = verify_public(&short);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("32"),
        "the refusal names the expected length: {out:?}"
    );

    // ⛔ BOTH inputs together are refused rather than silently preferring one:
    // they can disagree, and a pass whose meaning depends on argument order is
    // not a verification.
    let out = run(&[
        "verify",
        "--key",
        other_key.to_str().unwrap(),
        "--public-key",
        pubkey.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        sig.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "both inputs together are refused");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("exactly one"),
        "and the refusal says what to do: {out:?}"
    );

    // ⭐ POSITIVE CONTROL for the releaser's own path, which must not have moved:
    // `--key` alone still verifies, from the OTHER key's directory where a
    // private key does exist.
    let resigned = published.join("release-manifest.json.sig.other");
    assert!(run(&[
        "re-sign",
        "--key",
        other_key.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        resigned.to_str().unwrap(),
    ])
    .status
    .success());
    let out = run(&[
        "verify",
        "--key",
        other_key.to_str().unwrap(),
        "--bin-dir",
        bin_dir.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--sig",
        resigned.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "the private-key path still works: {out:?}"
    );
}

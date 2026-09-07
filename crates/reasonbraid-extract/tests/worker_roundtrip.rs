//! The worker's stdio contract (PHASE-4.4.2): spawn the BUILT binary, send
//! ONE request line on stdin, read ONE response line from stdout, assert the
//! Derivation shape — the process boundary is the quarantine.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

#[test]
fn the_worker_stdio_roundtrip_derives_the_chunks() {
    // The input: a small Atom feed on disk.
    let input_path = std::env::temp_dir().join(format!(
        "r2-roundtrip-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    let feed = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Roundtrip Feed</title>
  <entry><title>One</title><summary>the first</summary></entry>
</feed>"#;
    std::fs::write(&input_path, feed).expect("the input writes");

    let request = serde_json::json!({
        "input_path": input_path.display().to_string(),
        "media_type": "application/atom+xml",
        "limits": {
            "max_input_bytes": 1048576,
            "max_output_bytes": 1048576,
            "max_chunks": 64,
            "max_entry_bytes": 1048576,
            "max_decompression_ratio": 10.0
        }
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_reasonbraid-extract"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the worker spawns");
    {
        let mut stdin = child.stdin.take().expect("the worker stdin");
        writeln!(stdin, "{request}").expect("the request writes");
    }
    let mut output = String::new();
    child
        .stdout
        .take()
        .expect("the worker stdout")
        .read_to_string(&mut output)
        .expect("the response reads");
    let status = child.wait().expect("the worker exits");
    assert!(status.success(), "the worker exits cleanly: {status}");

    let response: serde_json::Value = serde_json::from_str(&output).expect("the response is JSON");
    assert!(response["parent_digest"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert_eq!(response["extractor_version"], "0.1.0");
    let chunks = response["chunks"].as_array().expect("the chunks");
    let joined: String = chunks
        .iter()
        .map(|c| c["text"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("|");
    assert!(joined.contains("Roundtrip Feed"), "{joined}");
    assert!(joined.contains("One"), "{joined}");
    assert!(joined.contains("the first"), "{joined}");
    for chunk in chunks {
        assert!(
            chunk["digest"].as_str().unwrap().starts_with("sha256:"),
            "every chunk carries its digest"
        );
    }

    std::fs::remove_file(&input_path).ok();
}

#[test]
fn the_worker_stdio_roundtrip_returns_the_named_refusal() {
    let input_path = std::env::temp_dir().join(format!(
        "r2-refusal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&input_path, b"not a feed").expect("the input writes");
    let request = serde_json::json!({
        "input_path": input_path.display().to_string(),
        "media_type": "application/atom+xml",
        "limits": {
            "max_input_bytes": 1048576,
            "max_output_bytes": 1048576,
            "max_chunks": 64,
            "max_entry_bytes": 1048576,
            "max_decompression_ratio": 10.0
        }
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_reasonbraid-extract"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the worker spawns");
    {
        let mut stdin = child.stdin.take().expect("the worker stdin");
        writeln!(stdin, "{request}").expect("the request writes");
    }
    let mut output = String::new();
    child
        .stdout
        .take()
        .expect("the worker stdout")
        .read_to_string(&mut output)
        .expect("the response reads");
    let status = child.wait().expect("the worker exits");
    assert!(
        status.success(),
        "the refusal still exits cleanly: {status}"
    );
    let response: serde_json::Value = serde_json::from_str(&output).expect("the response is JSON");
    assert_eq!(response["error"]["kind"], "feed_unreadable", "{response}");
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("feed"),
        "{response}"
    );
    std::fs::remove_file(&input_path).ok();
}

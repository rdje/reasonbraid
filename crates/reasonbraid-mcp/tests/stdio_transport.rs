//! The MCP surface is REACHABLE (`SIGNOFF-REPAIR.6.8`, ADR-024, §9.6).
//!
//! 🔴 **Before this control, six implemented, authorized and live-tested MCP
//! tools were reachable by no client**, and the evidence that they worked was a
//! unit test calling `McpTools::tool_router()` directly. That is exactly the
//! evidence that existed while nothing was served: a router constructed in
//! process proves the tools are correct and says nothing about whether anyone
//! can talk to them. The acceptance therefore asked for a control that drives a
//! tool **through the transport**, and this one spawns the real `rb-mcp` binary
//! and speaks newline-delimited JSON-RPC to its stdin.
//!
//! ⭐ **What each leg proves, named rather than implied:**
//!   1. the process starts and completes the MCP handshake — so the transport,
//!      the framing and the SDK wiring are real;
//!   2. it identifies itself as `reasonbraid`, not as `rmcp` — the string a
//!      client shows a person, and the one the generated handler got wrong;
//!   3. `tools/list` returns the six tools, over the wire rather than from the
//!      router;
//!   4. a `tools/call` reaches a real handler and its typed refusal comes back
//!      as a JSON-RPC error — the dispatch leg;
//!   5. every stdout line is a JSON frame — stdout IS the transport, and a
//!      stray `println!` would be a malformed frame the client's decoder
//!      reports as a protocol fault.
//!
//! ⚠️ **What it does NOT prove, stated rather than left to be assumed:** that a
//! tool can return a database-backed answer. `DATABASE_URL` here names a
//! cluster that need not exist, and the call in leg 4 is refused on its
//! arguments before the pool is touched. That is deliberate twice over — it
//! keeps the control offline, and it is the positive proof that the pool is
//! LAZY, which is what lets a client get a handshake and a tool list from a
//! process whose database is down. The database-backed path is the `mcp` suite's.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// The six tools the router exposes — the same list the in-process unit test
/// asserts, checked here against what a CLIENT receives.
const TOOLS: [&str; 6] = [
    "get_thread",
    "list_inbox",
    "get_policy_bundle",
    "respond",
    "join_call",
    "propose_policy_change",
];

struct Server {
    child: Child,
    /// An `Option` so the control can CLOSE it: ending the session is the
    /// client dropping its write half, and that is the behaviour under test.
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl Server {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rb-mcp"))
            // ⭐ A cluster that need not exist. If the binary connected
            // eagerly this control would not start at all, so the leg below
            // that gets a handshake IS the proof the pool is lazy.
            .env("DATABASE_URL", "postgres://rb-mcp-control.invalid/absent")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("rb-mcp spawns");
        let stdin = child.stdin.take().expect("stdin is piped");
        let stdout = BufReader::new(child.stdout.take().expect("stdout is piped"));
        Self {
            child,
            stdin: Some(stdin),
            stdout,
        }
    }

    fn send(&mut self, frame: &serde_json::Value) {
        let stdin = self.stdin.as_mut().expect("the session is still open");
        writeln!(stdin, "{frame}").expect("the frame writes");
        stdin.flush().expect("the frame flushes");
    }

    /// End the session the way a client does: drop the write half.
    fn close(&mut self) -> std::process::ExitStatus {
        self.stdin.take();
        self.child.wait().expect("the server exits")
    }

    /// One newline-delimited frame. ⛔ The raw line is returned alongside the
    /// parsed value so leg 5 can assert on the BYTES, not on something already
    /// known to have parsed.
    fn recv(&mut self) -> (String, serde_json::Value) {
        let mut line = String::new();
        let read = self.stdout.read_line(&mut line).expect("stdout reads");
        assert!(read > 0, "the server closed the stream before answering");
        let parsed = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("stdout carried a non-JSON frame ({e}): {line:?}"));
        (line, parsed)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn the_mcp_tools_are_reachable_over_the_stdio_transport() {
    let mut server = Server::start();

    // LEG 1 + 2 — the handshake, and who answers it.
    server.send(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "reasonbraid-control", "version": "0" }
        }
    }));
    let (raw_init, init) = server.recv();
    assert_eq!(
        init["id"],
        serde_json::json!(1),
        "the handshake answers: {raw_init}"
    );
    assert_eq!(
        init["result"]["serverInfo"]["name"],
        serde_json::json!("reasonbraid"),
        "🔴 the server names itself, and the generated handler named the SDK: {raw_init}"
    );
    assert_ne!(
        init["result"]["serverInfo"]["name"],
        serde_json::json!("rmcp"),
        "the superseded identity must not come back"
    );
    assert!(
        init["result"]["capabilities"]["tools"].is_object(),
        "the tools capability is advertised: {raw_init}"
    );
    // The negotiated version is the SDK's LATEST, not a version this transport
    // cannot back. Asserted as an exact string so a silent SDK bump is visible.
    assert_eq!(
        init["result"]["protocolVersion"],
        serde_json::json!("2025-11-25"),
        "the advertised protocol version is the one the SDK actually offers"
    );

    server.send(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));

    // LEG 3 — the tool list, over the wire.
    server.send(&serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}
    }));
    let (raw_list, list) = server.recv();
    let names: Vec<String> = list["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools/list returns an array: {raw_list}"))
        .iter()
        .map(|t| t["name"].as_str().unwrap_or_default().to_owned())
        .collect();
    for tool in TOOLS {
        assert!(
            names.contains(&tool.to_owned()),
            "{tool} is listed: {names:?}"
        );
    }
    assert_eq!(names.len(), TOOLS.len(), "and nothing else is: {names:?}");

    // LEG 4 — a tool CALL reaches a real handler, and its typed refusal comes
    // back through the transport. ⛔ The principal is malformed on purpose:
    // that refusal is decided before the pool, which keeps this arm offline and
    // is the positive control for the lazy pool.
    server.send(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_thread",
            "arguments": {
                "principal": "not-a-principal-id",
                "tenant_id": "ten_control",
                "thread_id": "thr_control"
            }
        }
    }));
    let (raw_call, call) = server.recv();
    assert_eq!(
        call["id"],
        serde_json::json!(3),
        "the call answers: {raw_call}"
    );
    assert!(
        call["error"].is_object(),
        "the malformed principal is refused, not answered: {raw_call}"
    );
    assert_eq!(
        call["error"]["code"],
        serde_json::json!(-32602),
        "and it is the SDK's invalid-params code, so the refusal is TYPED: {raw_call}"
    );

    // ⭐ POSITIVE CONTROL for leg 4: the same tool with a WELL-FORMED principal
    // gets past the argument check and fails on the database instead — which is
    // the proof the refusal above is about the principal and not about the tool
    // being unreachable. Two different failures from one tool is what separates
    // "dispatch works" from "everything errors".
    server.send(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "get_thread",
            "arguments": {
                "principal": "hpr_00000000-0000-7000-8000-00000000dead",
                "tenant_id": "ten_control",
                "thread_id": "thr_control"
            }
        }
    }));
    let (raw_deep, deep) = server.recv();
    assert_eq!(
        deep["id"],
        serde_json::json!(4),
        "the second call answers: {raw_deep}"
    );
    assert_ne!(
        deep["error"]["code"],
        serde_json::json!(-32602),
        "a well-formed principal is NOT refused on its arguments — the call went further: {raw_deep}"
    );

    // LEG 5 — stdout is the transport. Every frame read above parsed as JSON,
    // which `recv` enforced; this asserts the rule explicitly so a future
    // `println!` in the binary fails here with a sentence rather than as a
    // confusing decode error.
    for (label, raw) in [
        ("initialize", &raw_init),
        ("tools/list", &raw_list),
        ("tools/call", &raw_call),
    ] {
        assert!(
            raw.trim_start().starts_with('{') && raw.ends_with('\n'),
            "the {label} frame is one newline-terminated JSON object: {raw:?}"
        );
    }

    // Closing stdin ends the session — the client owns the lifetime, which is
    // the whole reason this is a binary of its own rather than a route.
    let status = server.close();
    assert!(
        status.success(),
        "closing stdin ends the session cleanly: {status:?}"
    );
}

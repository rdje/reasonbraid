//! The browser worker's stdio contract (PHASE-4.5.2): spawn the BUILT
//! binary against a local origin — the rendered text, the network log, and
//! the named refusals. The SKIP pattern covers machines without a browser.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// The skip guard: the worker's own browser probe (the startup check the
/// `.5.1` contract pins — the binary's provenance is named).
fn browser_present() -> bool {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
    ];
    std::env::var("R3_BROWSER_BIN")
        .map(|p| std::path::Path::new(&p).exists())
        .unwrap_or(false)
        || candidates.iter().any(|p| std::path::Path::new(p).exists())
}

struct Origin {
    base: String,
    hits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

async fn spawn_origin() -> Origin {
    use axum::routing::get;
    use axum::Router;
    let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = std::sync::Arc::clone(&hits);
    let app = Router::new()
        .route(
            "/page",
            get({
                let counter = std::sync::Arc::clone(&counter);
                move || {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    async {
                        (
                            [("content-type", "text/html")],
                            "<!doctype html><html><head><title>Render Title</title></head><body><h1>Rendered Heading</h1><p>the rendered body</p></body></html>",
                        )
                    }
                }
            }),
        )
        .route(
            "/sub.js",
            get({
                let counter = std::sync::Arc::clone(&counter);
                move || {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    async { ([("content-type", "application/javascript")], "console.log(1);") }
                }
            }),
        );
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("the origin binds");
    let port = listener.local_addr().expect("the port is known").port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("the origin serves");
    });
    Origin {
        base: format!("http://127.0.0.1:{port}"),
        hits,
    }
}

fn run_worker(request: &serde_json::Value) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
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
    output
}

#[tokio::test(flavor = "multi_thread")]
async fn the_browser_renders_the_page_and_logs_the_network() {
    if !browser_present() {
        println!("SKIP: no browser binary — set R3_BROWSER_BIN to run the render test");
        return;
    }
    let origin = spawn_origin().await;
    let request = serde_json::json!({
        "url": format!("{}/page", origin.base),
        "steps": [{ "action": "navigate", "url": format!("{}/page", origin.base) }],
        "limits": { "max_steps": 4, "max_output_bytes": 1048576, "time_budget_secs": 30 }
    });
    let output = run_worker(&request);
    let response: serde_json::Value = serde_json::from_str(&output).expect("the response is JSON");
    assert!(
        response["parent_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"),
        "{response}"
    );
    assert_eq!(response["page_title"], "Render Title", "{response}");
    assert_eq!(response["worker_version"], "0.1.0");
    let chunks = response["chunks"].as_array().expect("the chunks");
    let joined: String = chunks
        .iter()
        .map(|c| c["text"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("|");
    assert!(joined.contains("Rendered Heading"), "{joined}");
    assert!(joined.contains("the rendered body"), "{joined}");
    // The network log records the page request (the disclosure).
    let log = response["network_log"].as_array().expect("the network log");
    assert!(
        log.iter()
            .any(|e| e["url"].as_str().unwrap().contains("/page")),
        "{log:?}"
    );
    assert!(origin.hits.load(std::sync::atomic::Ordering::SeqCst) >= 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_step_budget_refuses_before_any_navigation() {
    if !browser_present() {
        println!("SKIP: no browser binary — set R3_BROWSER_BIN to run the render test");
        return;
    }
    let origin = spawn_origin().await;
    let request = serde_json::json!({
        "url": format!("{}/page", origin.base),
        "steps": [
            { "action": "navigate", "url": format!("{}/page", origin.base) },
            { "action": "click", "selector": "#nope" }
        ],
        "limits": { "max_steps": 1, "max_output_bytes": 1048576, "time_budget_secs": 30 }
    });
    let output = run_worker(&request);
    let response: serde_json::Value = serde_json::from_str(&output).expect("the response is JSON");
    assert_eq!(
        response["error"]["kind"], "step_budget_exceeded",
        "{response}"
    );
    assert_eq!(
        origin.hits.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "the budget refusal precedes any navigation"
    );
}

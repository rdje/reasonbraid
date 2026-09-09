//! The browser worker's stdio contract (PHASE-4.5.2): spawn the BUILT
//! binary against a local origin — the rendered text, the network log, and
//! named refusals, bounded worker groups and consumed origin shutdown.
//! Only rendering is unqualified when a browser is absent; budget admission always runs.

#![cfg(any(target_os = "linux", target_os = "macos"))]

mod support;

use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;
use std::time::Duration;
use support::{browser_binary, Fixture};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

struct Origin {
    base: String,
    hits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::io::Result<()>>>,
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
            "/slow",
            get({
                let counter = std::sync::Arc::clone(&counter);
                move || {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    async {
                        tokio::time::sleep(Duration::from_secs(6)).await;
                        "slow response"
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
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });
    Origin {
        base: format!("http://127.0.0.1:{port}"),
        hits,
        shutdown: Some(shutdown),
        task: Some(task),
    }
}

impl Origin {
    async fn finish(mut self) -> Result<(), String> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let mut task = self.task.take().ok_or("origin task missing")?;
        match timeout(Duration::from_secs(5), &mut task).await {
            Ok(result) => result
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string()),
            Err(_) => {
                task.abort();
                if timeout(Duration::from_secs(5), &mut task).await.is_err() {
                    return Err("origin abort completion not confirmed".to_owned());
                }
                Err("origin graceful shutdown not confirmed".to_owned())
            }
        }
    }
}

impl Drop for Origin {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
            eprintln!("origin aborted without confirmed graceful shutdown");
        }
    }
}

async fn conclude(fixture: Fixture, origin: Origin, result: std::thread::Result<()>) {
    let shutdown = origin.finish().await;
    fixture.finish(result.is_ok() && shutdown.is_ok()).unwrap();
    shutdown.expect("origin and connections stopped");
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_browser_renders_the_page_and_logs_the_network() {
    if browser_binary().is_none() {
        println!("SKIP: no browser binary — set R3_BROWSER_BIN to run the render test");
        return;
    }
    let fixture = Fixture::new();
    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let request = serde_json::json!({
            "url": format!("{}/page", origin.base),
            "steps": [{ "action": "navigate", "url": format!("{}/page", origin.base) }],
            "limits": { "max_steps": 4, "max_output_bytes": 1048576, "time_budget_secs": 30 }
        });
        let output = fixture
            .worker(&request)
            .await
            .expect("bounded worker completes");
        eprintln!(
            "test supervisor group stop requested: {}",
            output.group_stop_requested
        );
        assert!(
            !output.group_stop_requested,
            "worker must finish without supervisor assistance"
        );
        let receipt = completion(&output.stderr);
        assert_eq!(receipt["cleanup_confirmed"], true);
        assert_eq!(receipt["render_succeeded"], true);
        assert!(!fixture
            .path
            .join(receipt["workspace"].as_str().unwrap())
            .exists());
        let output = output.stdout;
        let response: serde_json::Value =
            serde_json::from_str(&output).expect("the response is JSON");
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
    })
    .catch_unwind()
    .await;
    conclude(fixture, origin, result).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_step_budget_refuses_before_any_navigation() {
    let fixture = Fixture::new();
    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let request = serde_json::json!({
            "url": format!("{}/page", origin.base),
            "steps": [
                { "action": "navigate", "url": format!("{}/page", origin.base) },
                { "action": "click", "selector": "#nope" }
            ],
            "limits": { "max_steps": 1, "max_output_bytes": 1048576, "time_budget_secs": 30 }
        });
        let output = fixture
            .worker(&request)
            .await
            .expect("bounded worker completes");
        eprintln!(
            "test supervisor group stop requested: {}",
            output.group_stop_requested
        );
        let output = output.stdout;
        let response: serde_json::Value =
            serde_json::from_str(&output).expect("the response is JSON");
        assert_eq!(
            response["error"]["kind"], "step_budget_exceeded",
            "{response}"
        );
        assert_eq!(
            origin.hits.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "the budget refusal precedes any navigation"
        );
    })
    .catch_unwind()
    .await;
    conclude(fixture, origin, result).await;
}

#[tokio::test]
async fn a_stalled_worker_is_bounded_and_its_group_is_consumed() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("sentinel"), b"owned unrelated witness").unwrap();
    let mut command = tokio::process::Command::new("python3");
    command.args([
        "-B",
        "-c",
        "import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); time.sleep(60)",
    ]);
    let result = fixture
        .command(command, b"", Duration::from_millis(500))
        .await;
    assert!(result.unwrap_err().contains("deadline exceeded"));
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.path.join("worker.json")).unwrap()).unwrap();
    assert_eq!(receipt["group_cleanup_confirmed"], true);
    assert_eq!(
        std::fs::read(fixture.path.join("sentinel")).unwrap(),
        b"owned unrelated witness"
    );
    fixture.finish(true).unwrap();
}

#[tokio::test]
async fn an_oversized_worker_stream_refuses_before_unbounded_capture() {
    for stream in ["stdout", "stderr"] {
        let fixture = Fixture::new();
        let mut command = tokio::process::Command::new("python3");
        command.args(["-B", "-c", &format!("import sys,time; sys.{stream}.buffer.write(b'x'*(3*1024*1024)); sys.{stream}.flush(); time.sleep(60)")]);
        let result = fixture.command(command, b"", Duration::from_secs(10)).await;
        assert!(result.unwrap_err().contains("output limit exceeded"));
        assert_eq!(
            std::fs::metadata(fixture.path.join(format!("{stream}.log")))
                .unwrap()
                .len(),
            2 * 1024 * 1024 + 1
        );
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(fixture.path.join("worker.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["group_cleanup_confirmed"], true);
        fixture.finish(true).unwrap();
    }
}

#[tokio::test]
async fn origin_shutdown_closes_the_listener() {
    let origin = spawn_origin().await;
    let address = origin.base.trim_start_matches("http://").to_owned();
    origin.finish().await.unwrap();
    assert!(timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(address)
    )
    .await
    .unwrap()
    .is_err());
}

#[tokio::test]
async fn a_stalled_descendant_cannot_outlive_the_owned_worker_group() {
    let fixture = Fixture::new();
    let mut command = tokio::process::Command::new("python3");
    command.args(["-B", "-c", "import os,signal,time; from pathlib import Path; signal.signal(signal.SIGTERM,signal.SIG_IGN); child=os.fork(); Path('descendant.pid').write_text(str(child)) if child else None; time.sleep(60)"]);
    let result = fixture.command(command, b"", Duration::from_secs(2)).await;
    assert!(result.unwrap_err().contains("deadline exceeded"));
    let raw: i32 = std::fs::read_to_string(fixture.path.join("descendant.pid"))
        .unwrap()
        .parse()
        .unwrap();
    assert!(raw > 1, "a real descendant started before the timeout");
    let pid = rustix::process::Pid::from_raw(raw).unwrap();
    assert_eq!(
        rustix::process::test_kill_process(pid),
        Err(rustix::io::Errno::SRCH)
    );
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.path.join("worker.json")).unwrap()).unwrap();
    assert_eq!(receipt["group_cleanup_confirmed"], true);
    fixture.finish(true).unwrap();
}

fn completion(stderr: &str) -> serde_json::Value {
    let line = stderr
        .lines()
        .find_map(|line| line.strip_prefix("browser completion: "))
        .expect("worker emits its owned completion receipt");
    let receipt = serde_json::from_str(line).unwrap();
    eprintln!("qualified browser completion: {line}");
    receipt
}

#[tokio::test(flavor = "multi_thread")]
async fn real_step_error_and_output_refusal_finish_before_returning() {
    if browser_binary().is_none() {
        println!("SKIP: no browser binary — failure cleanup unqualified");
        return;
    }
    for (step, output_limit, expected) in [
        (
            Some(serde_json::json!({"action":"click", "selector":"#absent"})),
            1048576,
            "click_failed",
        ),
        (None, 1, "output_too_large"),
    ] {
        let fixture = Fixture::new();
        let origin = spawn_origin().await;
        let result = AssertUnwindSafe(async {
            let url = format!("{}/page", origin.base);
            let mut steps = vec![serde_json::json!({"action":"navigate", "url":url})];
            steps.extend(step);
            let request = serde_json::json!({"url":url, "steps":steps,
                "limits":{"max_steps":4,"max_output_bytes":output_limit,"time_budget_secs":15}});
            let output = fixture
                .worker(&request)
                .await
                .expect("owned worker cleanup");
            assert!(!output.group_stop_requested);
            let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
            assert_eq!(response["error"]["kind"], expected, "{response}");
            let receipt = completion(&output.stderr);
            assert_eq!(receipt["cleanup_confirmed"], true);
            assert_eq!(receipt["render_succeeded"], false);
            let path = fixture.path.join(receipt["workspace"].as_str().unwrap());
            assert!(path.join("owner.json").is_file());
            assert!(path.join("completion.json").is_file());
            assert!(
                std::fs::metadata(path.join("browser.stderr"))
                    .unwrap()
                    .len()
                    <= 65536
            );
        })
        .catch_unwind()
        .await;
        conclude(fixture, origin, result).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_deadline_cancels_launch_but_still_reaps_the_owned_browser() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let program = fixture.path.join("stalled-browser");
    std::fs::write(
        &program,
        "#!/bin/sh\ntrap '' TERM\nwhile :; do sleep 1; done\n",
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let request = serde_json::json!({"url":"about:blank", "steps":[{"action":"navigate","url":"about:blank"}],
        "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":1}});
    let result = AssertUnwindSafe(async {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        command.env("R3_BROWSER_BIN", &program);
        let output = fixture
            .command(
                command,
                request.to_string().as_bytes(),
                Duration::from_secs(20),
            )
            .await
            .unwrap();
        fixture.verify_browser_groups().await.unwrap();
        assert!(!output.group_stop_requested);
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(
            response["error"]["kind"], "time_budget_exceeded",
            "{response}"
        );
        assert_eq!(completion(&output.stderr)["cleanup_confirmed"], true);
    })
    .catch_unwind()
    .await;
    fixture.finish(result.is_ok()).unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_real_navigation_deadline_stops_the_browser_and_origin() {
    if browser_binary().is_none() {
        println!("SKIP: no browser binary — navigation deadline unqualified");
        return;
    }
    let fixture = Fixture::new();
    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let url = format!("{}/slow", origin.base);
        let request = serde_json::json!({"url":url,"steps":[{"action":"navigate","url":url}],
            "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":4}});
        let output = fixture.worker(&request).await.unwrap();
        assert!(!output.group_stop_requested);
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(
            response["error"]["kind"], "time_budget_exceeded",
            "{response}"
        );
        assert!(
            origin.hits.load(std::sync::atomic::Ordering::SeqCst) >= 1,
            "actual navigation must start before the deadline"
        );
        assert_eq!(completion(&output.stderr)["cleanup_confirmed"], true);
    })
    .catch_unwind()
    .await;
    conclude(fixture, origin, result).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn overlapping_workers_use_distinct_profiles_under_the_same_root() {
    let Some(binary) = browser_binary() else {
        println!("SKIP: no browser binary — overlapping profiles unqualified");
        return;
    };
    let first = Fixture::new();
    let second = Fixture::new();
    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let url = format!("{}/page", origin.base);
        let mut steps = vec![serde_json::json!({"action":"navigate", "url":url})];
        steps.extend((0..15).map(|_| serde_json::json!({"action":"scroll","y":0})));
        let request = serde_json::json!({"url":url,"steps":steps,
            "limits":{"max_steps":16,"max_output_bytes":1048576,"time_budget_secs":20}})
        .to_string();
        let mut one = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        let mut two = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        one.env("R3_BROWSER_BIN", &binary).current_dir(&first.path);
        two.env("R3_BROWSER_BIN", &binary).current_dir(&first.path);
        let observe = async {
            timeout(Duration::from_secs(12), async {
                loop {
                    if origin.hits.load(std::sync::atomic::Ordering::SeqCst) >= 2 {
                        let paths: Vec<_> =
                            std::fs::read_dir(first.path.join(".project-data/browser"))
                                .unwrap()
                                .map(|e| e.unwrap().path())
                                .collect();
                        assert_eq!(paths.len(), 2, "both distinct profiles must still be live");
                        let mut groups = Vec::new();
                        for path in paths {
                            let receipt: serde_json::Value = serde_json::from_slice(
                                &std::fs::read(path.join("owner.json")).unwrap(),
                            )
                            .unwrap();
                            let pid = rustix::process::Pid::from_raw(
                                i32::try_from(receipt["browser_group"].as_i64().unwrap()).unwrap(),
                            )
                            .unwrap();
                            rustix::process::test_kill_process_group(pid)
                                .expect("both browser groups are live at the same observation");
                            assert!(path.join("profile").is_dir());
                            assert!(!path.join("completion.json").exists());
                            eprintln!("overlapping live browser: {receipt}");
                            groups.push(pid);
                        }
                        assert_ne!(groups[0], groups[1]);
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .expect("both real browsers navigate while their profiles coexist");
        };
        // Catch the observer panic without cancelling the two owned command futures.
        let (one, two, observed) = tokio::join!(
            first.command(one, request.as_bytes(), Duration::from_secs(35)),
            second.command(two, request.as_bytes(), Duration::from_secs(35)),
            AssertUnwindSafe(observe).catch_unwind(),
        );
        first.verify_browser_groups().await.unwrap();
        second.verify_browser_groups().await.unwrap();
        if let Err(panic) = observed {
            std::panic::resume_unwind(panic);
        }
        let one = one.unwrap();
        let two = two.unwrap();
        assert!(!one.group_stop_requested && !two.group_stop_requested);
        let receipts = [completion(&one.stderr), completion(&two.stderr)];
        assert_ne!(receipts[0]["workspace"], receipts[1]["workspace"]);
        for (output, receipt) in [(one, &receipts[0]), (two, &receipts[1])] {
            let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
            assert_eq!(response["page_title"], "Render Title", "{response}");
            assert_eq!(receipt["cleanup_confirmed"], true);
            assert!(!first
                .path
                .join(receipt["workspace"].as_str().unwrap())
                .exists());
        }
    })
    .catch_unwind()
    .await;
    let shutdown = origin.finish().await;
    let success = result.is_ok() && shutdown.is_ok();
    first.finish(success).unwrap();
    second.finish(success).unwrap();
    shutdown.unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_startup_drains_and_caps_browser_diagnostics() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let program = fixture.path.join("failed-browser");
    std::fs::write(
        &program,
        r#"#!/bin/sh
exec python3 -B - <<'PYTHON'
import os
from pathlib import Path
root = Path.cwd()
expected = {
    "CHROME_LOG_FILE": "chrome.log",
    "BREAKPAD_DUMP_LOCATION": "crashes",
    "CHROME_USER_DATA_DIR": "profile",
}
for key, value in expected.items():
    assert Path(os.environ[key]) == root / value
assert all(key not in os.environ for key in ["SSLKEYLOGFILE", "QLOGDIR"])
os.write(2, b"x" * 100000)
raise SystemExit(7)
PYTHON
"#,
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let request = serde_json::json!({"url":"about:blank","steps":[{"action":"navigate","url":"about:blank"}],
        "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":10}});
    let result = AssertUnwindSafe(async {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        command.env("R3_BROWSER_BIN", &program);
        for key in [
            "CHROME_LOG_FILE",
            "BREAKPAD_DUMP_LOCATION",
            "CHROME_USER_DATA_DIR",
            "SSLKEYLOGFILE",
            "QLOGDIR",
        ] {
            command.env(key, fixture.path.join("must-not-be-used"));
        }
        let output = fixture
            .command(
                command,
                request.to_string().as_bytes(),
                Duration::from_secs(20),
            )
            .await
            .unwrap();
        fixture.verify_browser_groups().await.unwrap();
        assert!(!output.group_stop_requested);
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(
            response["error"]["kind"], "browser_launch_failed",
            "{response}"
        );
        let receipt = completion(&output.stderr);
        assert_eq!(receipt["cleanup_confirmed"], true);
        let path = fixture.path.join(receipt["workspace"].as_str().unwrap());
        assert_eq!(
            std::fs::read(path.join("browser.stderr")).unwrap(),
            vec![b'x'; 65536]
        );
    })
    .catch_unwind()
    .await;
    fixture.finish(result.is_ok()).unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

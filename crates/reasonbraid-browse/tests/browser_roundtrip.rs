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
use tokio::time::{timeout, Instant};

// Production allows twenty seconds for browser startup. Give its protocol/page
// setup ten more seconds to reach the origin; a short render budget cannot prove
// cancellation during navigation when it can expire during startup instead.
const NAVIGATION_WINDOW: Duration = Duration::from_secs(30);
const SECOND_LAUNCH_DELAY: Duration = Duration::from_secs(4);
const COMPLETION_WINDOW: Duration = Duration::from_secs(10);
// The render's ten-second cleanup allowance plus worker process startup margin.
const SUPERVISOR_MARGIN: Duration = Duration::from_secs(20);

// The receipt belongs to this exact listener. Reaching its old port after
// shutdown may instead reach a newly bound socket and says nothing about ownership.
struct OwnedListener {
    socket: Option<tokio::net::TcpListener>,
    closed: Option<oneshot::Sender<()>>,
}

impl axum::serve::Listener for OwnedListener {
    type Io = tokio::net::TcpStream;
    type Addr = std::net::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        axum::serve::Listener::accept(self.socket.as_mut().expect("live listener")).await
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.socket.as_ref().expect("live listener").local_addr()
    }
}

impl Drop for OwnedListener {
    fn drop(&mut self) {
        drop(self.socket.take());
        if let Some(closed) = self.closed.take() {
            let _ = closed.send(());
        }
    }
}

struct Origin {
    base: String,
    hits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    navigation_release: tokio::sync::watch::Sender<bool>,
    navigation_arrivals: tokio::sync::watch::Sender<u64>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<std::io::Result<()>>>,
    listener_closed: oneshot::Receiver<()>,
}

async fn spawn_origin() -> Origin {
    use axum::routing::get;
    use axum::Router;
    let hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = std::sync::Arc::clone(&hits);
    let (navigation_release, navigation_gate) = tokio::sync::watch::channel(false);
    let (navigation_arrivals, _) = tokio::sync::watch::channel(0_u64);
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
            "/gated",
            get({
                let counter = std::sync::Arc::clone(&counter);
                let arrivals = navigation_arrivals.clone();
                move || {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    arrivals.send_modify(|count| *count += 1);
                    let mut gate = navigation_gate.clone();
                    async move {
                        let _ = gate.wait_for(|released| *released).await;
                        ([("content-type", "text/html")], "<title>Render Title</title><body>gated overlap</body>")
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
    let (closed, listener_closed) = oneshot::channel();
    let listener = OwnedListener {
        socket: Some(listener),
        closed: Some(closed),
    };
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
        navigation_release,
        navigation_arrivals,
        shutdown: Some(shutdown),
        task: Some(task),
        listener_closed,
    }
}

impl Origin {
    async fn wait_for_navigations(&self, count: u64, limit: Duration) -> Result<(), String> {
        let mut arrivals = self.navigation_arrivals.subscribe();
        timeout(limit, arrivals.wait_for(|observed| *observed >= count))
            .await
            .map_err(|_| {
                format!(
                    "expected {count} gated navigations within {limit:?}; observed {}",
                    *self.navigation_arrivals.borrow()
                )
            })?
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn finish(mut self) -> Result<(), String> {
        self.navigation_release.send_replace(true);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let mut task = self.task.take().ok_or("origin task missing")?;
        let result = match timeout(Duration::from_secs(5), &mut task).await {
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
        };
        result?;
        self.listener_closed
            .try_recv()
            .map_err(|e| format!("original listener close unconfirmed: {e}"))?;
        Ok(())
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

#[tokio::test]
async fn gated_origin_requires_arrival_and_explicit_response_release() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let mut stream = timeout(
            NAVIGATION_WINDOW,
            tokio::net::TcpStream::connect(origin.base.trim_start_matches("http://")),
        )
        .await
        .unwrap()
        .unwrap();
        stream
            .write_all(b"GET /gated HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        origin
            .wait_for_navigations(1, NAVIGATION_WINDOW)
            .await
            .unwrap();
        let missing_second = origin
            .wait_for_navigations(2, Duration::from_millis(50))
            .await
            .unwrap_err();
        assert!(missing_second.contains("observed 1"));
        assert!(timeout(Duration::from_millis(50), stream.read_u8())
            .await
            .is_err());
        origin.navigation_release.send_replace(true);
        let mut response = String::new();
        timeout(NAVIGATION_WINDOW, stream.read_to_string(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response.contains("gated overlap"));
        assert_eq!(*origin.navigation_arrivals.borrow(), 1);
    })
    .catch_unwind()
    .await;
    origin.finish().await.expect("owned origin stopped");
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
    let mut origin = spawn_origin().await;
    assert_eq!(
        origin.listener_closed.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
    // finish consumes both the serving task and the exact listener's drop receipt.
    // A later connection to a reused port is deliberately not the postcondition.
    origin.finish().await.unwrap();
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
        // The fast-host leg of the same contract the injected slow host asserts.
        assert_eq!(response["error"]["cleanup_confirmed"], true, "{response}");
        assert!(
            response["error"].get("cleanup_error").is_none(),
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
        let url = format!("{}/gated", origin.base);
        let request = serde_json::json!({"url":url,"steps":[{"action":"navigate","url":url}],
            "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":NAVIGATION_WINDOW.as_secs()}});
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        command.env("R3_BROWSER_BIN", browser_binary().unwrap());
        let input = request.to_string();
        let started = Instant::now();
        // Keep the response gated until the worker has returned. A fixed slow
        // response can either finish before the budget or outlast a budget that
        // was already exhausted by startup; neither proves navigation cancellation.
        let (output, arrived) = tokio::join!(
            fixture.command(command, input.as_bytes(), NAVIGATION_WINDOW + SUPERVISOR_MARGIN),
            origin.wait_for_navigations(1, NAVIGATION_WINDOW),
        );
        fixture.verify_browser_groups().await.unwrap();
        eprintln!("navigation deadline witness: {}", serde_json::json!({
            "arrived": arrived.is_ok(), "arrival_error": arrived.as_ref().err(),
            "worker_error": output.as_ref().err(), "elapsed_ms": started.elapsed().as_millis(),
            "response_released": *origin.navigation_release.borrow(),
            "render_budget_secs": NAVIGATION_WINDOW.as_secs(),
        }));
        arrived.expect("actual gated navigation must occur before the render deadline");
        let output = output.unwrap();
        assert!(!output.group_stop_requested);
        assert!(!*origin.navigation_release.borrow());
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(
            response["error"]["kind"], "time_budget_exceeded",
            "{response}"
        );
        // The real-browser fast host asserts the same contract the injected
        // slow host does: the render's own kind, plus an explicit cleanup fact.
        assert_eq!(response["error"]["cleanup_confirmed"], true, "{response}");
        assert!(response["error"].get("cleanup_error").is_none(), "{response}");
        assert!(started.elapsed() >= NAVIGATION_WINDOW);
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
        let url = format!("{}/gated", origin.base);
        let observation_window = NAVIGATION_WINDOW * 2 + SECOND_LAUNCH_DELAY;
        let render_budget = observation_window + COMPLETION_WINDOW;
        let worker_limit = render_budget + SUPERVISOR_MARGIN;
        let started = Instant::now();
        let request = serde_json::json!({"url":url,"steps":[{"action":"navigate", "url":url}],
            "limits":{"max_steps":1,"max_output_bytes":1048576,"time_budget_secs":render_budget.as_secs()}})
        .to_string();
        let mut one = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        let mut two = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        one.env("R3_BROWSER_BIN", &binary).current_dir(&first.path);
        two.env("R3_BROWSER_BIN", &binary).current_dir(&first.path);
        let observe = async {
            origin.wait_for_navigations(2, observation_window).await
                .expect("both real browsers navigate while their profiles coexist");
            let paths: Vec<_> = std::fs::read_dir(first.path.join(".project-data/browser"))
                .unwrap().map(|e| e.unwrap().path()).collect();
            assert_eq!(paths.len(), 2, "both distinct profiles must still be live");
            let mut groups = Vec::new();
            for path in paths {
                let receipt: serde_json::Value = serde_json::from_slice(
                    &std::fs::read(path.join("owner.json")).unwrap(),
                ).unwrap();
                let pid = rustix::process::Pid::from_raw(
                    i32::try_from(receipt["browser_group"].as_i64().unwrap()).unwrap(),
                ).unwrap();
                rustix::process::test_kill_process_group(pid)
                    .expect("both browser groups are live at the same observation");
                assert!(path.join("profile").is_dir());
                assert!(!path.join("completion.json").exists());
                eprintln!("overlapping live browser: {receipt}");
                groups.push(pid);
            }
            assert_ne!(groups[0], groups[1]);
        };
        let observe_and_release = async {
            // An observer failure must release the origin before command futures
            // are consumed; it cannot strand the gated requests.
            let result = AssertUnwindSafe(observe).catch_unwind().await;
            origin.navigation_release.send_replace(true);
            result
        };
        let delayed_second = async {
            origin.wait_for_navigations(1, NAVIGATION_WINDOW).await?;
            eprintln!("first gated navigation witnessed after {} ms", started.elapsed().as_millis());
            // Deliberately exceed the old three-second scroll-based window.
            tokio::time::sleep(SECOND_LAUNCH_DELAY).await;
            second
                .command(two, request.as_bytes(), worker_limit)
                .await
        };
        let (one, two, observed) = tokio::join!(
            first.command(one, request.as_bytes(), worker_limit),
            delayed_second,
            observe_and_release,
        );
        eprintln!("overlap dispatch witness: {}", serde_json::json!({
            "first_error": one.as_ref().err(), "second_error": two.as_ref().err(),
            "observation_panicked": observed.is_err(),
            "gated_arrivals": *origin.navigation_arrivals.borrow(),
            "elapsed_ms": started.elapsed().as_millis(),
            "render_budget_secs": render_budget.as_secs(),
        }));
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

async fn worker_in_root(
    fixture: &Fixture,
    root: &std::path::Path,
    request: &serde_json::Value,
) -> support::WorkerOutput {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
    command
        .env(
            "R3_BROWSER_BIN",
            browser_binary().expect("browser prerequisite checked"),
        )
        .current_dir(root);
    let result = fixture
        .command(
            command,
            request.to_string().as_bytes(),
            Duration::from_secs(45),
        )
        .await;
    fixture
        .verify_browser_groups()
        .await
        .expect("browser group cleanup independently confirmed");
    result.expect("bounded worker completes")
}

#[tokio::test(flavor = "multi_thread")]
async fn moving_the_runtime_root_preserves_storage_and_next_invocation() {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    if browser_binary().is_none() {
        println!("SKIP: no browser binary — runtime-root relocation unqualified");
        return;
    }
    let first = Fixture::new();
    let second = Fixture::new();
    let origin = spawn_origin().await;
    let result = AssertUnwindSafe(async {
        let before = first.path.join("root-before");
        let after = first.path.join("root-after");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&before)
            .unwrap();
        std::fs::write(before.join("Cargo.toml"), "# movable owned runtime root\n").unwrap();
        std::fs::create_dir(before.join("migrations")).unwrap();
        let original = std::fs::metadata(&before).unwrap();
        let url = format!("{}/page", origin.base);
        let request = serde_json::json!({"url":url,"steps":[{"action":"navigate","url":url}],
            "limits":{"max_steps":1,"max_output_bytes":1048576,"time_budget_secs":15}});
        let one = worker_in_root(&first, &before, &request).await;
        let first_receipt = completion(&one.stderr);
        let first_response: serde_json::Value = serde_json::from_str(&one.stdout).unwrap();
        assert_eq!(
            first_response["page_title"], "Render Title",
            "{first_response}"
        );
        assert!(!one.group_stop_requested);
        assert_eq!(first_receipt["cleanup_confirmed"], true);
        assert!(!before
            .join(first_receipt["workspace"].as_str().unwrap())
            .exists());
        let witness = std::path::Path::new(".project-data/browser/witness");
        std::fs::write(before.join(witness), b"preserve through relocation").unwrap();
        std::fs::rename(&before, &after).unwrap();
        let moved = std::fs::metadata(&after).unwrap();
        assert_eq!((moved.dev(), moved.ino()), (original.dev(), original.ino()));
        assert!(!before.exists());
        let two = worker_in_root(&second, &after, &request).await;
        let second_receipt = completion(&two.stderr);
        let second_response: serde_json::Value = serde_json::from_str(&two.stdout).unwrap();
        assert_eq!(
            second_response["page_title"], "Render Title",
            "{second_response}"
        );
        assert!(!two.group_stop_requested);
        assert_eq!(second_receipt["cleanup_confirmed"], true);
        assert!(!after
            .join(second_receipt["workspace"].as_str().unwrap())
            .exists());
        assert_ne!(first_receipt["workspace"], second_receipt["workspace"]);
        assert_eq!(
            std::fs::read(after.join(witness)).unwrap(),
            b"preserve through relocation"
        );
        assert_eq!(
            std::fs::read_dir(after.join(".project-data/browser"))
                .unwrap()
                .count(),
            1
        );
        eprintln!(
            "runtime root relocation: {}",
            serde_json::json!({
                "device": moved.dev(), "inode": moved.ino(), "old_root_absent": true,
                "witness_preserved": true, "first": first_receipt, "second": second_receipt,
            })
        );
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
async fn a_linked_storage_parent_refuses_before_starting_chrome() {
    use std::os::unix::fs::symlink;
    if browser_binary().is_none() {
        println!("SKIP: no browser binary — worker storage refusal unqualified");
        return;
    }
    let fixture = Fixture::new();
    let result = AssertUnwindSafe(async {
        let target = fixture.path.join("linked-target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("witness"), b"untouched target").unwrap();
        symlink(&target, fixture.path.join(".project-data")).unwrap();
        let request = serde_json::json!({"url":"about:blank","steps":[{"action":"navigate","url":"about:blank"}],
            "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":5}});
        let output = fixture.worker(&request).await.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(response["error"]["kind"], "browser_storage_failed", "{response}");
        assert!(!output.group_stop_requested);
        assert!(!output.stderr.contains("browser ownership:"));
        assert!(std::fs::symlink_metadata(fixture.path.join(".project-data")).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read(target.join("witness")).unwrap(), b"untouched target");
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
        eprintln!("linked storage refusal: {}", serde_json::json!({"kind":response["error"]["kind"],"target_unchanged":true,"browser_ownership_emitted":false}));
    }).catch_unwind().await;
    fixture.finish(result.is_ok()).unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

/// The slow-host leg of the response contract (`SIGNOFF-REPAIR.11.4.3.1.2.27`).
///
/// A render refusal and an unconfirmed cleanup are two independent facts, and a
/// slow host produces both at once. Waiting for a slow host to produce them is
/// not a control: the condition is load-dependent and did not reproduce on a
/// quiet machine. This injects the exact shape the desktop-runtime diagnosis
/// proved instead — a process that ESCAPES the browser's group and keeps the
/// inherited stderr write end open, so no EOF arrives and stderr completion
/// cannot be confirmed while the group itself exits normally.
#[tokio::test(flavor = "multi_thread")]
async fn a_render_refusal_survives_an_unconfirmed_cleanup() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let program = fixture.path.join("escaping-browser");
    // The parent is the "browser": it never publishes a DevTools endpoint, so
    // the render budget trips while launch is still waiting. Its forked child
    // leaves the process group with setsid and holds the inherited stderr.
    std::fs::write(
        &program,
        "#!/bin/sh\nexec python3 -B -c '\nimport os, sys, time\nif os.fork() == 0:\n    os.setsid()\n    open(\"escaped.pid\", \"w\").write(str(os.getpid()))\n    time.sleep(25)\n    os._exit(0)\ntime.sleep(60)\n'\n",
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let request = serde_json::json!({"url":"about:blank","steps":[{"action":"navigate","url":"about:blank"}],
        "limits":{"max_steps":1,"max_output_bytes":1024,"time_budget_secs":1}});
    let result = AssertUnwindSafe(async {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        command.env("R3_BROWSER_BIN", &program);
        let started = Instant::now();
        let output = fixture
            .command(
                command,
                request.to_string().as_bytes(),
                Duration::from_secs(45),
            )
            .await
            .unwrap();
        // The owned browser group is gone; only the escaped writer remains.
        fixture.verify_browser_groups().await.unwrap();
        assert!(!output.group_stop_requested);
        let response: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        eprintln!(
            "unconfirmed-cleanup refusal: {}",
            serde_json::json!({"error": response["error"], "elapsed_ms": started.elapsed().as_millis()})
        );
        // The render's OWN outcome is what the caller must act on, and an
        // unconfirmed cleanup does not overwrite it.
        assert_eq!(
            response["error"]["kind"], "time_budget_exceeded",
            "{response}"
        );
        // The cleanup fact travels beside it, explicit and machine-readable.
        assert_eq!(response["error"]["cleanup_confirmed"], false, "{response}");
        assert!(
            response["error"]["cleanup_error"]
                .as_str()
                .is_some_and(|detail| detail.contains("stderr completion unconfirmed")),
            "{response}"
        );
        // The worker's own receipt agrees with the response it returned.
        let receipt = completion(&output.stderr);
        assert_eq!(receipt["cleanup_confirmed"], false, "{receipt}");
        assert_eq!(receipt["render_succeeded"], false, "{receipt}");
    })
    .catch_unwind()
    .await;
    // The escaped writer is this control's own residue: reap it and prove it.
    let escaped = std::fs::read_to_string(fixture.path.join("escaped.pid"))
        .ok()
        .and_then(|raw| raw.trim().parse::<i32>().ok())
        .and_then(rustix::process::Pid::from_raw);
    if let Some(pid) = escaped {
        let _ = rustix::process::kill_process(pid, rustix::process::Signal::KILL);
        let deadline = Instant::now() + Duration::from_secs(5);
        while rustix::process::test_kill_process(pid).is_ok() {
            assert!(
                Instant::now() < deadline,
                "the escaped writer outlived its control"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    fixture.finish(result.is_ok()).unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

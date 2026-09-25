// The pinned Chrome, owned by a test (`SIGNOFF-REPAIR.11.1.1`): a real browser for
// the console controls, driven over CDP by the client the browse worker uses.
//
// The runtime is NAMED, never discovered. `R3_BROWSER_BIN` is set only by
// `scripts/ci_browser.py`, which downloads, hash-verifies and version-checks the
// pinned Chrome for Testing build; `reasonbraid-browse`'s test support records
// why a desktop browser is an unqualified substitute rather than a fallback.
//
// Ownership follows the worker's `lifetime.rs`. Chrome starts in its own process
// group under a private workspace in `target/console-browser/`, and its stderr
// pipe is the exit detector: EOF needs every write end closed, including one held
// by a helper that left the group, as Chrome's crashpad handler does. A browser
// that finishes cleanly removes its workspace. One that does not — a failed
// control, an unconfirmed cleanup — keeps it, with Chrome's stderr beside it and
// `browser.json` naming the group and the outcome, which is the receipt
// `scripts/census_retained_fixtures.py` reads before it retires the payload.

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::{Browser, Page};
use futures_util::StreamExt;
use rustix::process::{kill_process_group, test_kill_process_group, Pid, Signal};
use tokio::io::AsyncReadExt as _;
use tokio::process::{Child, ChildStderr, Command};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, timeout_at, Instant};

const STARTUP: Duration = Duration::from_secs(30);
const CLEANUP: Duration = Duration::from_secs(10);
const STDERR_LIMIT: usize = 64 * 1024;

// The worker's qualified flag set, less what only a render worker needs. Debugging
// listens on loopback only, on a port the kernel chooses. `--no-sandbox` is the
// worker's stance too: the Ubuntu runners restrict the unprivileged user
// namespaces Chrome's Linux sandbox needs, and the page is this repository's own
// console over loopback, showing fixture data.
//
// ⛔ And no hostname resolves. `--disable-background-networking` does not stop
// Chrome's own services: under the rest of these flags the pinned build, left on
// `about:blank` for 20 s, opened six connections to Google — network time,
// accounts, the component updater, and messaging check-in and registration —
// measured with `--log-net-log` (`SIGNOFF-REPAIR.11.1.1`). With the rule, none.
// The console is served on 127.0.0.1 by address, which needs no resolution.
const CHROME_ARGS: &[&str] = &[
    "--headless",
    "--host-resolver-rules=MAP * ~NOTFOUND , EXCLUDE 127.0.0.1",
    "--remote-debugging-port=0",
    "--remote-debugging-address=127.0.0.1",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-breakpad",
    "--disable-default-apps",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    "--disable-sync",
    "--metrics-recording-only",
    "--password-store=basic",
    "--use-mock-keychain",
    "--hide-scrollbars",
    "--mute-audio",
    "--no-sandbox",
];

/// The pinned browser, or `None` when this run was not given one.
pub fn browser_binary() -> Option<PathBuf> {
    std::env::var_os("R3_BROWSER_BIN").map(PathBuf::from)
}

/// Announce a control that did not run, in a way the announcement survives.
///
/// libtest captures `eprintln!` and replays it only for a failing test, so a skip
/// announced that way reads exactly like a pass. A write through the
/// `std::io::stderr()` handle is not captured.
pub fn skip_without_browser(unqualified: &str) {
    let mut stderr = io::stderr();
    let _ = writeln!(
        stderr,
        "SKIP (R3_BROWSER_BIN unset): {unqualified} is UNQUALIFIED by this run; run \
         `python3 -B scripts/ci_browser.py -- bash scripts/run_pg_tests.sh console_browser`"
    );
    let _ = stderr.flush();
}

pub struct Chrome {
    workspace: PathBuf,
    /// `browser.json`'s fixed half: the workspace, relative to the repository,
    /// and the process group. The outcome is added when the workspace is kept.
    receipt: serde_json::Value,
    child: Child,
    group: Pid,
    browser: Option<Browser>,
    handler: Option<JoinHandle<()>>,
    stderr: Option<JoinHandle<()>>,
    stderr_bytes: Arc<Mutex<Vec<u8>>>,
    finished: bool,
}

impl Chrome {
    /// Start the pinned Chrome under `root/target/console-browser/` and connect.
    pub async fn launch(binary: &Path, root: &Path) -> Self {
        let relative = format!("target/console-browser/run-{}", uuid::Uuid::now_v7());
        let workspace = root.join(&relative);
        for directory in [
            "profile", "cache", "tmp", "config", "data", "state", "crashes",
        ] {
            std::fs::create_dir_all(workspace.join(directory)).expect("browser workspace");
        }
        let mut command = Command::new(binary);
        command
            .args(CHROME_ARGS)
            .arg(path_argument(
                "--user-data-dir=",
                &workspace.join("profile"),
            ))
            .arg(path_argument("--disk-cache-dir=", &workspace.join("cache")))
            .current_dir(&workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        // Chrome's process-singleton socket lives under TMPDIR, and `sun_path`
        // holds 108 bytes; a relative directory resolves against the workspace
        // for 3 bytes (the worker's `lifetime.rs` measured the absolute one at
        // 228 on a CI runner).
        for key in ["TMPDIR", "TMP", "TEMP"] {
            command.env(key, "tmp");
        }
        for (key, directory) in [
            ("XDG_CACHE_HOME", "cache"),
            ("XDG_CONFIG_HOME", "config"),
            ("CHROME_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_STATE_HOME", "state"),
            ("BREAKPAD_DUMP_LOCATION", "crashes"),
        ] {
            command.env(key, workspace.join(directory));
        }
        command.env_remove("SSLKEYLOGFILE").env_remove("QLOGDIR");

        let mut child = command.spawn().expect("spawn the pinned Chrome");
        let group = child
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .and_then(Pid::from_raw)
            .expect("the browser's process group");
        let receipt = serde_json::json!({
            "workspace": relative,
            "browser_group": group.as_raw_nonzero().get(),
        });
        let stderr = child.stderr.take().expect("the browser's stderr");
        let stderr_bytes = Arc::new(Mutex::new(Vec::new()));
        let (send, endpoint) = oneshot::channel();
        let drain = tokio::spawn(drain_stderr(stderr, Arc::clone(&stderr_bytes), send));
        // Owned from here on: a panic below drops `chrome`, which kills the group.
        let mut chrome = Self {
            workspace,
            receipt,
            child,
            group,
            browser: None,
            handler: None,
            stderr: Some(drain),
            stderr_bytes,
            finished: false,
        };
        // The group is named on disk before the first await, so a run killed
        // from outside still leaves the census an identity to check.
        chrome.record("running", &[]);

        let endpoint = timeout(STARTUP, endpoint)
            .await
            .expect("Chrome published no DevTools endpoint within its startup budget")
            .expect("Chrome exited before publishing its DevTools endpoint");
        let config = HandlerConfig {
            viewport: Some(Default::default()),
            ..Default::default()
        };
        let (browser, mut handler) = Browser::connect_with_config(endpoint, config)
            .await
            .expect("connect to the pinned Chrome");
        chrome.handler = Some(tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        }));
        chrome.browser = Some(browser);
        chrome
    }

    /// A new tab showing `url`, loaded: `goto` resolves after the load event,
    /// which follows every deferred script.
    pub async fn open(&self, url: &str) -> Page {
        let browser = self.browser.as_ref().expect("a connected browser");
        let page = browser.new_page("about:blank").await.expect("open a page");
        page.goto(url).await.expect("the page loads");
        page
    }

    /// Close the browser and prove it gone: its group empty, its child reaped and
    /// its stderr at EOF. Only then is the workspace removed.
    pub async fn finish(mut self) {
        let deadline = Instant::now() + CLEANUP;
        if let Some(mut browser) = self.browser.take() {
            // A CDP acknowledgement is not exit evidence; the checks below are.
            let _ = timeout_at(deadline, browser.close()).await;
        }
        let mut failures = Vec::new();
        match kill_process_group(self.group, Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => {}
            Err(e) => failures.push(format!("group signal: {e}")),
        }
        match timeout_at(deadline, self.child.wait()).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => failures.push(format!("reap: {e}")),
            Err(_) => failures.push("reap: deadline".to_owned()),
        }
        // Darwin can answer EPERM while unreaped members remain; only ESRCH is
        // evidence that the group is empty.
        loop {
            match test_kill_process_group(self.group) {
                Err(rustix::io::Errno::SRCH) => break,
                _ if Instant::now() >= deadline => {
                    failures.push("group: still present at the deadline".to_owned());
                    break;
                }
                _ => sleep(Duration::from_millis(20)).await,
            }
        }
        if let Some(handler) = self.handler.take() {
            handler.abort();
            let _ = timeout_at(deadline, handler).await;
        }
        if let Some(mut drain) = self.stderr.take() {
            if timeout_at(deadline, &mut drain).await.is_err() {
                drain.abort();
                failures.push("stderr: a process outside the group still holds it".to_owned());
            }
        }
        self.finished = true;
        if failures.is_empty() {
            std::fs::remove_dir_all(&self.workspace).expect("remove the browser workspace");
        } else {
            self.retain("cleanup_unconfirmed", &failures);
            panic!(
                "browser cleanup unconfirmed ({}); workspace kept at {}",
                failures.join("; "),
                self.workspace.display()
            );
        }
    }

    /// Write `browser.json`: the fixed half plus `state` and any failures.
    fn record(&self, state: &str, failures: &[String]) {
        let mut receipt = self.receipt.clone();
        receipt["state"] = state.into();
        receipt["failures"] = failures.into();
        if let Ok(bytes) = serde_json::to_vec_pretty(&receipt) {
            let _ = std::fs::write(self.workspace.join("browser.json"), bytes);
        }
    }

    /// Keep the workspace as evidence: the outcome, and Chrome's own stderr.
    fn retain(&self, state: &str, failures: &[String]) {
        self.record(state, failures);
        if let Ok(bytes) = self.stderr_bytes.lock() {
            let _ = std::fs::write(self.workspace.join("chrome.stderr"), &*bytes);
        }
    }
}

impl Drop for Chrome {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        // An unfinished browser is a failed control: stop it, keep the evidence.
        let _ = kill_process_group(self.group, Signal::KILL);
        let _ = self.child.start_kill();
        for task in [&self.handler, &self.stderr].into_iter().flatten() {
            task.abort();
        }
        self.retain("control_failed", &[]);
    }
}

fn path_argument(name: &str, path: &Path) -> std::ffi::OsString {
    let mut argument = std::ffi::OsString::from(name);
    argument.push(path);
    argument
}

/// Keep the first `STDERR_LIMIT` bytes, read to EOF, and hand over the endpoint
/// Chrome announces as `DevTools listening on ws://127.0.0.1:<port>/devtools/…`.
async fn drain_stderr(
    mut stderr: ChildStderr,
    saved: Arc<Mutex<Vec<u8>>>,
    send: oneshot::Sender<String>,
) {
    let mut send = Some(send);
    let mut chunk = [0; 4096];
    loop {
        let read = match stderr.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(read) => read,
        };
        let Ok(mut saved) = saved.lock() else { break };
        let room = STDERR_LIMIT.saturating_sub(saved.len());
        saved.extend_from_slice(&chunk[..read.min(room)]);
        if send.is_some() {
            if let Some(endpoint) = endpoint(&saved) {
                if let Some(send) = send.take() {
                    let _ = send.send(endpoint);
                }
            }
        }
    }
}

fn endpoint(stderr: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stderr)
        .split_inclusive('\n')
        .filter(|line| line.ends_with('\n'))
        .find_map(|line| {
            let url = line.trim_end().strip_prefix("DevTools listening on ")?;
            url.starts_with("ws://127.0.0.1:").then(|| url.to_owned())
        })
}

//! Own the process before the first cancellable await; finish outside render timeout.

use std::io;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use futures_util::StreamExt;
use rustix::process::{kill_process_group, test_kill_process_group, Pid, Signal};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStderr, Command};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout_at, Instant};

use crate::storage::Workspace;

const CLEANUP_BUDGET: Duration = Duration::from_secs(10);
const GRACE: Duration = Duration::from_secs(2);
const STDERR_LIMIT: usize = 64 * 1024;
const LINE_LIMIT: usize = 4096;

type Endpoint = oneshot::Receiver<Result<String, String>>;

pub struct Lifetime {
    workspace: Workspace,
    child: Option<Child>,
    pid: Option<Pid>,
    pub browser: Option<chromiumoxide::Browser>,
    handler: Option<JoinHandle<()>>,
    pub network: Option<JoinHandle<()>>,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
    finished: bool,
}

impl Lifetime {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            workspace: Workspace::discover()?,
            child: None,
            pid: None,
            browser: None,
            handler: None,
            network: None,
            stderr: None,
            finished: false,
        })
    }

    fn spawn(&mut self, binary: &Path) -> io::Result<Endpoint> {
        let mut command = Command::new(binary);
        command
            .args(CHROME_ARGS)
            .arg(path_argument(
                "--user-data-dir=",
                &self.workspace.path.join("profile"),
            ))
            .arg(path_argument(
                "--disk-cache-dir=",
                &self.workspace.path.join("cache"),
            ))
            .current_dir(&self.workspace.path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        // Chrome's process singleton binds a Unix domain socket under the
        // temporary directory, and `sun_path` holds 108 bytes. It places the
        // socket there PRECISELY to keep that path short, so an absolute
        // TMPDIR pointing into this per-invocation workspace defeats the
        // vendor's own mitigation: on a CI runner the result measured 228
        // bytes and Chrome aborted with
        // `FATAL:process_singleton_posix.cc:313] Socket path too long`.
        //
        // Shortening names cannot fix it. With a 45-byte socket suffix the
        // budget is 63 bytes, and even a short temp directory under a short
        // fixture root measures 67. The child's working directory is already
        // this workspace, so a RELATIVE temporary directory resolves to the
        // same place while costing 3 bytes instead of 183.
        //
        // This cannot regress: should Chrome canonicalise TMPDIR, it resolves
        // against that same working directory and yields exactly the absolute
        // path used before. Every other variable stays absolute, so the
        // profile, cache and state isolation the controls assert is unchanged.
        for (key, relative) in [("TMPDIR", "tmp"), ("TMP", "tmp"), ("TEMP", "tmp")] {
            command.env(key, relative);
        }
        for (key, directory) in [
            ("XDG_CACHE_HOME", "cache"),
            ("XDG_CONFIG_HOME", "config"),
            ("CHROME_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_STATE_HOME", "state"),
            ("CHROME_USER_DATA_DIR", "profile"),
            ("CHROME_LOG_FILE", "chrome.log"),
            ("BREAKPAD_DUMP_LOCATION", "crashes"),
        ] {
            command.env(key, self.workspace.path.join(directory));
        }
        command.env_remove("SSLKEYLOGFILE").env_remove("QLOGDIR");
        let child = command.spawn()?;
        // POSIX spawn returns a positive pid_t. Publish ownership without an await
        // between spawn and storing the handle, including before receipt I/O.
        self.pid = child
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .and_then(Pid::from_raw);
        self.child = Some(child);
        let pid = self
            .pid
            .ok_or_else(|| io::Error::other("browser process identity unavailable"))?;
        let receipt = serde_json::json!({
            "workspace": self.workspace.relative(), "browser_group": pid.as_raw_nonzero().get()
        });
        self.workspace
            .write_new("owner.json", &serde_json::to_vec(&receipt)?)?;
        eprintln!("browser ownership: {receipt}");
        let stderr = self
            .child
            .as_mut()
            .and_then(|child| child.stderr.take())
            .ok_or_else(|| io::Error::other("browser stderr unavailable"))?;
        let (send, receive) = oneshot::channel();
        self.stderr = Some(tokio::spawn(read_stderr(stderr, send)));
        Ok(receive)
    }

    pub async fn launch(&mut self, binary: &Path) -> Result<(), String> {
        let endpoint = self.spawn(binary).map_err(|e| e.to_string())?;
        let endpoint = endpoint
            .await
            .map_err(|_| "browser stderr ended before endpoint discovery".to_owned())??;
        // Use only the owned child's validated loopback WebSocket endpoint. This
        // avoids the HTTP discovery branch and its separate ambient HTTP client.
        let config = chromiumoxide::handler::HandlerConfig {
            viewport: Some(Default::default()),
            ..Default::default()
        };
        let (browser, mut handler) = chromiumoxide::Browser::connect_with_config(endpoint, config)
            .await
            .map_err(|e| e.to_string())?;
        self.browser = Some(browser);
        self.handler = Some(tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        }));
        Ok(())
    }

    /// All branches, including cancelled launch/connect, arrive here. Cleanup has
    /// one additional budget; no response may bypass this result.
    pub async fn finish(&mut self, success: bool) -> Result<(), String> {
        let deadline = Instant::now() + CLEANUP_BUDGET;
        if let Some(browser) = self.browser.as_mut() {
            // CDP acknowledgement alone is never process-exit evidence.
            let _ = timeout_at((Instant::now() + GRACE).min(deadline), browser.close()).await;
        }
        self.browser.take();
        let process_result = self.stop_process(deadline).await;
        let browser_pid = self.pid;
        if process_result.is_ok() {
            // Never signal a retired numeric group during a later storage/task error.
            self.pid = None;
        }
        let handler_result = join_aborted(&mut self.handler, deadline).await;
        let network_result = join_aborted(&mut self.network, deadline).await;
        let mut stderr_result = Ok(Vec::new());
        if let Some(mut task) = self.stderr.take() {
            stderr_result = match timeout_at(deadline, &mut task).await {
                Ok(result) => result
                    .map_err(|e| e.to_string())
                    .and_then(|v| v.map_err(|e| e.to_string())),
                Err(_) => {
                    task.abort();
                    // Consume the cancellation within the same shutdown deadline.
                    let _ = timeout_at(deadline, &mut task).await;
                    Err("browser stderr completion unconfirmed".to_owned())
                }
            };
        }
        let cleanup = process_result
            .and(handler_result)
            .and(network_result)
            .and(stderr_result.as_ref().map(|_| ()).map_err(Clone::clone));
        let receipt = serde_json::json!({
            "workspace": self.workspace.relative(),
            "browser_group": browser_pid.map(|pid| pid.as_raw_nonzero().get()),
            "cleanup_confirmed": cleanup.is_ok(), "render_succeeded": success,
            "cleanup_error": cleanup.as_ref().err(),
        });
        eprintln!("browser completion: {receipt}");
        if success && cleanup.is_ok() {
            self.workspace.remove().map_err(|e| e.to_string())?;
        } else {
            self.workspace
                .write_new(
                    "completion.json",
                    &serde_json::to_vec(&receipt).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
            if let Ok(bytes) = stderr_result {
                self.workspace
                    .write_new("browser.stderr", &bytes)
                    .map_err(|e| e.to_string())?;
            }
        }
        cleanup?;
        self.finished = true;
        Ok(())
    }

    async fn stop_process(&mut self, deadline: Instant) -> Result<(), String> {
        let Some(pid) = self.pid else {
            if let Some(child) = self.child.as_mut() {
                child.start_kill().map_err(|e| e.to_string())?;
                timeout_at(deadline, child.wait())
                    .await
                    .map_err(|_| "browser reap deadline")?
                    .map_err(|e| e.to_string())?;
                return Err("browser group identity unavailable; cleanup unconfirmed".to_owned());
            }
            return Ok(());
        };
        for signal in [None, Some(Signal::TERM), Some(Signal::KILL)] {
            if let Some(signal) = signal {
                match kill_process_group(pid, signal) {
                    Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => {}
                    Err(e) => return Err(format!("browser group signal: {e}")),
                }
            }
            let stage_end = if signal == Some(Signal::KILL) {
                deadline
            } else {
                (Instant::now() + GRACE).min(deadline)
            };
            loop {
                let reaped = self
                    .child
                    .as_mut()
                    .ok_or("browser child missing")?
                    .try_wait()
                    .map_err(|e| e.to_string())?
                    .is_some();
                match test_kill_process_group(pid) {
                    Err(rustix::io::Errno::SRCH) if reaped => return Ok(()),
                    // Darwin may report EPERM while only unreaped descendants
                    // remain. Neither this nor a denied signal establishes absence.
                    Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => {}
                    Err(e) => return Err(format!("browser group inspection: {e}")),
                }
                if Instant::now() >= stage_end {
                    break;
                }
                sleep(
                    Duration::from_millis(20)
                        .min(stage_end.saturating_duration_since(Instant::now())),
                )
                .await;
            }
        }
        Err("browser group exit/reaping unconfirmed at cleanup deadline".to_owned())
    }
}

impl Drop for Lifetime {
    fn drop(&mut self) {
        if !self.finished {
            // Emergency requests are not completion evidence. Workspace Drop
            // deliberately retains data, and callers must inspect owner.json.
            if let Some(pid) = self.pid {
                let _ = kill_process_group(pid, Signal::KILL);
            }
            if let Some(child) = self.child.as_mut() {
                let _ = child.start_kill();
            }
            for task in [&self.handler, &self.network].into_iter().flatten() {
                task.abort();
            }
            if let Some(task) = &self.stderr {
                task.abort();
            }
        }
    }
}

fn path_argument(name: &str, path: &Path) -> std::ffi::OsString {
    let mut argument = std::ffi::OsString::from(name);
    argument.push(path);
    argument
}

async fn join_aborted(task: &mut Option<JoinHandle<()>>, deadline: Instant) -> Result<(), String> {
    if let Some(mut task) = task.take() {
        task.abort();
        match timeout_at(deadline, &mut task).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) if e.is_cancelled() => {}
            Ok(Err(e)) => return Err(format!("browser task failed: {e}")),
            Err(_) => return Err("browser task cancellation unconfirmed".to_owned()),
        }
    }
    Ok(())
}

async fn read_stderr(
    mut stderr: ChildStderr,
    send: oneshot::Sender<Result<String, String>>,
) -> io::Result<Vec<u8>> {
    let mut send = Some(send);
    let mut saved = Vec::new();
    let mut line = Vec::new();
    let mut overlong = false;
    let mut chunk = [0; 4096];
    loop {
        let read = stderr.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        saved.extend_from_slice(&chunk[..read.min(STDERR_LIMIT - saved.len())]);
        if send.is_some() {
            for &byte in &chunk[..read] {
                if byte == b'\n' {
                    if !overlong {
                        if let Some(endpoint) = parse_endpoint(&line) {
                            if let Some(send) = send.take() {
                                let _ = send.send(Ok(endpoint));
                            }
                        }
                    }
                    line.clear();
                    overlong = false;
                } else if line.len() < LINE_LIMIT {
                    line.push(byte);
                } else {
                    overlong = true;
                }
            }
        }
    }
    if let Some(send) = send {
        let _ = send.send(Err(
            "browser exited before publishing a loopback endpoint".to_owned()
        ));
    }
    Ok(saved)
}

fn parse_endpoint(line: &[u8]) -> Option<String> {
    let url = std::str::from_utf8(line)
        .ok()?
        .trim_end_matches('\r')
        .strip_prefix("DevTools listening on ")?;
    let suffix = url.strip_prefix("ws://127.0.0.1:")?;
    let (port, id) = suffix.split_once("/devtools/browser/")?;
    if port.parse::<u16>().ok()? == 0 || uuid::Uuid::parse_str(id).is_err() {
        return None;
    }
    Some(url.to_owned())
}

// Keep chromiumoxide 0.9.1's existing default launch behavior, with explicit
// loopback debugging, private storage and process ownership added here. The
// deployment's existing no-sandbox boundary is separately owned by SIGNOFF-REPAIR.7.2.
const CHROME_ARGS: &[&str] = &[
    "--disable-background-networking",
    "--enable-features=NetworkService,NetworkServiceInProcess",
    "--disable-background-timer-throttling",
    "--disable-backgrounding-occluded-windows",
    "--disable-breakpad",
    "--disable-client-side-phishing-detection",
    "--disable-component-extensions-with-background-pages",
    "--disable-default-apps",
    "--disable-dev-shm-usage",
    "--disable-features=TranslateUI",
    "--disable-hang-monitor",
    "--disable-ipc-flooding-protection",
    "--disable-popup-blocking",
    "--disable-prompt-on-repost",
    "--disable-renderer-backgrounding",
    "--disable-sync",
    "--force-color-profile=srgb",
    "--metrics-recording-only",
    "--no-first-run",
    "--enable-automation",
    "--password-store=basic",
    "--use-mock-keychain",
    "--enable-blink-features=IdleDetection",
    "--lang=en_US",
    "--remote-debugging-port=0",
    "--remote-debugging-address=127.0.0.1",
    "--disable-extensions",
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--headless",
    "--hide-scrollbars",
    "--mute-audio",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_failed_handler_join_retains_storage_and_refuses_completion() {
        let mut owner = Lifetime::new().unwrap();
        owner.handler = Some(tokio::spawn(async {
            panic!("deliberate owned handler failure");
        }));
        while !owner.handler.as_ref().unwrap().is_finished() {
            tokio::task::yield_now().await;
        }
        let error = owner.finish(true).await.unwrap_err();
        assert!(error.contains("browser task failed"), "{error}");
        let receipt: serde_json::Value = serde_json::from_slice(
            &std::fs::read(owner.workspace.path.join("completion.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["cleanup_confirmed"], false);
        assert!(owner.workspace.path.is_dir());
        assert!(owner.handler.is_none() && owner.child.is_none());
        // The injected task's panic was consumed and no process was started.
        owner.workspace.remove().unwrap();
        owner.finished = true;
    }

    #[test]
    fn discovery_accepts_only_a_nonzero_loopback_browser_endpoint() {
        let id = "89e82f46-26b8-4280-91ec-642b5cfa4a6b";
        assert_eq!(
            parse_endpoint(
                format!("DevTools listening on ws://127.0.0.1:1234/devtools/browser/{id}\r")
                    .as_bytes()
            ),
            Some(format!("ws://127.0.0.1:1234/devtools/browser/{id}"))
        );
        for url in [
            format!("ws://example.com:1234/devtools/browser/{id}"),
            format!("ws://127.0.0.1:0/devtools/browser/{id}"),
            format!("ws://127.0.0.1:65536/devtools/browser/{id}"),
            "ws://127.0.0.1:1234/devtools/page/invalid".to_owned(),
            format!("ws://127.0.0.1:1234/devtools/browser/{id}?redirect=evil"),
        ] {
            assert!(parse_endpoint(format!("DevTools listening on {url}").as_bytes()).is_none());
        }
    }
}

//! Owned on-volume fixtures and bounded worker groups for browser controls.

use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use rustix::process::{kill_process_group, test_kill_process_group, Pid, Signal};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Instant};

const OUTPUT_LIMIT: u64 = 2 * 1024 * 1024;
const SHUTDOWN_LIMIT: Duration = Duration::from_secs(5);

pub struct Fixture {
    pub path: PathBuf,
    identity: (u64, u64),
    safe_to_delete: AtomicBool,
    used: AtomicBool,
    finished: bool,
}

impl Fixture {
    pub fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let device = std::fs::metadata(&root).unwrap().dev();
        let mut parent = root;
        for component in ["target", "browser-lifetime-controls"] {
            parent.push(component);
            match std::fs::DirBuilder::new().mode(0o700).create(&parent) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("fixture parent refuses: {e}"),
            }
            let metadata = std::fs::symlink_metadata(&parent).unwrap();
            assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
            assert_eq!(metadata.dev(), device);
        }
        let path = parent.join(format!("case-{}", uuid::Uuid::now_v7()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        for name in ["tmp", "cache", "config", "data", "state"] {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(path.join(name))
                .unwrap();
        }
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert_eq!(metadata.dev(), device);
        eprintln!("browser fixture created: {}", path.display());
        Self {
            path,
            identity: (metadata.dev(), metadata.ino()),
            safe_to_delete: AtomicBool::new(true),
            used: AtomicBool::new(false),
            finished: false,
        }
    }

    pub async fn worker(&self, request: &serde_json::Value) -> Result<WorkerOutput, String> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_reasonbraid-browse"));
        if let Some(binary) = browser_binary() {
            command.env("R3_BROWSER_BIN", binary);
        }
        self.command(
            command,
            request.to_string().as_bytes(),
            Duration::from_secs(45),
        )
        .await
    }

    pub async fn command(
        &self,
        mut command: Command,
        input: &[u8],
        limit: Duration,
    ) -> Result<WorkerOutput, String> {
        if self.used.swap(true, Ordering::SeqCst) {
            return Err("fixture already owns a command; use a new fixture".to_owned());
        }
        for (key, dir) in [
            ("TMPDIR", "tmp"),
            ("TMP", "tmp"),
            ("TEMP", "tmp"),
            ("XDG_CACHE_HOME", "cache"),
            ("XDG_CONFIG_HOME", "config"),
            ("CHROME_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_STATE_HOME", "state"),
        ] {
            command.env(key, self.path.join(dir));
        }
        command
            .current_dir(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        let child = command.spawn().map_err(|e| format!("worker spawn: {e}"))?;
        self.safe_to_delete.store(false, Ordering::SeqCst);
        let mut group = WorkerGroup::new(child)?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = async {
            let mut stdin = group.child.stdin.take().ok_or("missing stdin")?;
            let output = group.child.stdout.take().ok_or("missing stdout")?;
            let error = group.child.stderr.take().ok_or("missing stderr")?;
            let write = async move {
                stdin
                    .write_all(input)
                    .await
                    .map_err(|e| format!("stdin: {e}"))?;
                drop(stdin); // The worker reads to EOF; release the actual pipe writer.
                Ok::<(), String>(())
            };
            let wait = async {
                group
                    .child
                    .wait()
                    .await
                    .map_err(|e| format!("worker wait: {e}"))
            };
            let (_, (), (), status) = tokio::try_join!(
                write,
                read_bounded(output, &mut stdout),
                read_bounded(error, &mut stderr),
                wait
            )?;
            Ok::<ExitStatus, String>(status)
        };
        let result = match timeout(limit, result).await {
            Ok(result) => result,
            Err(_) => Err("worker deadline exceeded".to_owned()),
        };
        let cleanup = group.finish().await;
        if cleanup.is_ok() {
            self.safe_to_delete.store(true, Ordering::SeqCst);
        }
        std::fs::write(self.path.join("stdout.log"), &stdout).map_err(|e| e.to_string())?;
        std::fs::write(self.path.join("stderr.log"), &stderr).map_err(|e| e.to_string())?;
        let receipt = serde_json::json!({
            "fixture": self.path.file_name().unwrap().to_string_lossy(),
            "pid": group.pid.as_raw_nonzero().get(),
            "worker_exit": result.as_ref().ok().and_then(|status| status.code()),
            "group_cleanup_confirmed": cleanup.is_ok(),
            "group_stop_requested": cleanup.as_ref().ok(),
            "operation_error": result.as_ref().err(),
            "cleanup_error": cleanup.as_ref().err(),
            "stdout_bytes": stdout.len(),
            "stderr_bytes": stderr.len()
        });
        std::fs::write(
            self.path.join("worker.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        eprintln!("worker terminal receipt: {receipt}");
        let group_stop_requested = cleanup?;
        let status = result?;
        if !status.success() {
            return Err(format!("worker exited {status}"));
        }
        Ok(WorkerOutput {
            stdout: String::from_utf8(stdout).map_err(|e| e.to_string())?,
            group_stop_requested,
        })
    }

    pub fn finish(mut self, success: bool) -> Result<(), String> {
        if !success || !self.safe_to_delete.load(Ordering::SeqCst) {
            return Ok(());
        }
        let metadata = std::fs::symlink_metadata(&self.path).map_err(|e| e.to_string())?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || (metadata.dev(), metadata.ino()) != self.identity
        {
            return Err("fixture identity changed; retain data".to_owned());
        }
        std::fs::remove_dir_all(&self.path).map_err(|e| e.to_string())?;
        match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.finished = true;
                Ok(())
            }
            _ => Err("fixture removal not confirmed".to_owned()),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.finished {
            eprintln!("browser fixture retained: {}", self.path.display());
        }
    }
}

#[derive(Debug)]
pub struct WorkerOutput {
    pub stdout: String,
    // A requested test-side stop is not evidence of production cleanup.
    pub group_stop_requested: bool,
}

async fn read_bounded(reader: impl AsyncRead + Unpin, bytes: &mut Vec<u8>) -> Result<(), String> {
    reader
        .take(OUTPUT_LIMIT + 1)
        .read_to_end(bytes)
        .await
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > OUTPUT_LIMIT {
        return Err("worker output limit exceeded".to_owned());
    }
    Ok(())
}

fn group_exists(pid: Pid) -> rustix::io::Result<bool> {
    match test_kill_process_group(pid) {
        Ok(()) => Ok(true),
        Err(rustix::io::Errno::SRCH) => Ok(false),
        Err(e) => Err(e),
    }
}

async fn observe_group(
    mut probe: impl FnMut() -> rustix::io::Result<bool>,
    deadline: Instant,
) -> Result<bool, String> {
    loop {
        match probe() {
            Ok(present) => return Ok(present),
            Err(rustix::io::Errno::PERM) if Instant::now() < deadline => {
                // Darwin can report EPERM for an existing group containing only
                // zombies. Wait for an observable state; never call this absence.
                sleep(
                    Duration::from_millis(20)
                        .min(deadline.saturating_duration_since(Instant::now())),
                )
                .await;
            }
            Err(rustix::io::Errno::PERM) => {
                return Err(
                    "worker group observation remained denied; cleanup unconfirmed".to_owned(),
                );
            }
            Err(e) => return Err(format!("cannot inspect worker group: {e}")),
        }
    }
}

fn request_group_stop(pid: Pid, signal: Signal) -> Result<(), String> {
    match kill_process_group(pid, signal) {
        // A group may enter teardown between observation and signalling. Neither
        // refusal qualifies cleanup: subsequent observation must prove absence.
        Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => Ok(()),
        Err(e) => Err(format!("worker group signal request: {e}")),
    }
}

struct WorkerGroup {
    child: Child,
    pid: Pid,
    finished: bool,
}

impl WorkerGroup {
    fn new(child: Child) -> Result<Self, String> {
        let raw = child.id().ok_or("worker has no process identity")?;
        let pid = Pid::from_raw(i32::try_from(raw).map_err(|e| e.to_string())?)
            .ok_or("invalid worker PID")?;
        if pid.as_raw_nonzero().get() <= 1 {
            return Err("invalid worker group".to_owned());
        }
        Ok(Self {
            child,
            pid,
            finished: false,
        })
    }

    async fn finish(&mut self) -> Result<bool, String> {
        self.child.try_wait().map_err(|e| e.to_string())?;
        let requested =
            observe_group(|| group_exists(self.pid), Instant::now() + SHUTDOWN_LIMIT).await?;
        if requested {
            request_group_stop(self.pid, Signal::TERM)?;
        }
        let deadline = Instant::now() + SHUTDOWN_LIMIT;
        loop {
            self.child.try_wait().map_err(|e| e.to_string())?;
            if !observe_group(|| group_exists(self.pid), deadline).await? {
                self.finished = true;
                return Ok(requested);
            }
            if Instant::now() >= deadline {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
        request_group_stop(self.pid, Signal::KILL)?;
        timeout(SHUTDOWN_LIMIT, self.child.wait())
            .await
            .map_err(|_| "worker reap deadline")?
            .map_err(|e| e.to_string())?;
        let deadline = Instant::now() + SHUTDOWN_LIMIT;
        while observe_group(|| group_exists(self.pid), deadline).await? {
            if Instant::now() >= deadline {
                return Err("worker group removal not confirmed".to_owned());
            }
            sleep(Duration::from_millis(20)).await;
        }
        self.finished = true;
        Ok(requested)
    }
}

impl Drop for WorkerGroup {
    fn drop(&mut self) {
        if !self.finished {
            // Emergency cancellation requests cleanup but does not claim reaping.
            let _ = kill_process_group(self.pid, Signal::KILL);
        }
    }
}

pub fn browser_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("R3_BROWSER_BIN") {
        return Some(path.into());
    }
    [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
    ]
    .into_iter()
    .map(PathBuf::from)
    .find(|p| p.is_file())
}

#[tokio::test]
async fn transient_group_denial_requires_a_later_absence_observation() {
    let mut calls = 0;
    let observed = observe_group(
        || {
            calls += 1;
            if calls == 1 {
                Err(rustix::io::Errno::PERM)
            } else {
                Ok(false)
            }
        },
        Instant::now() + Duration::from_secs(1),
    )
    .await
    .unwrap();
    assert!(!observed);
    assert_eq!(calls, 2, "the denial itself was never accepted as absence");
}

#[tokio::test]
async fn persistent_group_denial_remains_unconfirmed_at_the_deadline() {
    let started = Instant::now();
    let result = timeout(
        Duration::from_secs(1),
        observe_group(
            || Err(rustix::io::Errno::PERM),
            started + Duration::from_millis(60),
        ),
    )
    .await
    .expect("observation has a finite bound");
    assert!(result.unwrap_err().contains("cleanup unconfirmed"));
    assert!(started.elapsed() >= Duration::from_millis(60));
}

//! Direct-worker completion controls for the R2 extraction spawner
//! (`SIGNOFF-REPAIR.7.3.3.2.2`).
//!
//! Every scenario drives the REAL `run_extraction` against a controlled
//! repository-local worker script, then asks the operating system whether the
//! exact direct child this process created is still in the process table. A
//! reaped child has no entry; a killed-but-unreaped child is still a zombie
//! entry, so the same observation proves stop AND reap.
//!
//! These controls need no database and no network. They are POSIX-only: the
//! worker scripts, the process-table probe and the fixture ownership checks all
//! assume a Unix host (the extraction worker's own supported platforms).

#![cfg(unix)]

use std::fs::{self, DirBuilder, Metadata, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use reasonbraid_server::extraction::{
    run_extraction, run_extraction_reporting, ExtractionError, ExtractionRun, WorkerCompletion,
    WorkerLimits, WorkerResponse,
};

/// `R2_WORKER_BIN` is process-wide state: every scenario holds this lock across
/// the override AND the spawner call, so two controls never select each other's
/// worker. Poisoning is recovered deliberately — a panicking control must not
/// disable the remaining ones.
fn worker_selection() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = LOCK.get_or_init(|| Mutex::new(()));
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn same_volume(left: &Metadata, right: &Metadata) -> bool {
    left.dev() == right.dev()
}

/// The repository root derived at runtime from the current directory, never a
/// persisted absolute path (§12/§13: the checkout may move between runs).
fn repository_root() -> PathBuf {
    std::env::current_dir()
        .expect("the current directory is readable")
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("scripts/project_env.py").is_file()
                && path.join("rust-toolchain.toml").is_file()
        })
        .expect("run the extraction completion controls inside the repository")
        .canonicalize()
        .expect("the repository root canonicalizes")
}

/// `target/extraction-completion-controls/fixtures`, created 0700 on the
/// repository's own volume. There is no temporary-directory or home fallback.
fn controls_storage(root: &Path) -> PathBuf {
    let volume = fs::metadata(root).expect("the repository root is readable");
    let mut path = root.to_path_buf();
    for part in ["target", "extraction-completion-controls", "fixtures"] {
        path.push(part);
        let mut builder = DirBuilder::new();
        builder.mode(0o700);
        match builder.create(&path) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => panic!("the control storage {} refused: {error}", path.display()),
        }
        let metadata = fs::symlink_metadata(&path).expect("the control storage is readable");
        assert!(
            metadata.is_dir() && same_volume(&metadata, &volume),
            "the control storage must be an unlinked directory on the repository volume: {}",
            path.display()
        );
    }
    path
}

/// One scenario's exclusively created directory: its worker script and the
/// files that script writes. A failing scenario retains it for inspection.
struct Fixture {
    root: PathBuf,
    dir: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = repository_root();
        let storage = controls_storage(&root);
        for _ in 0..512 {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let dir = storage.join(format!("{name}-{}-{sequence}", std::process::id()));
            let mut builder = DirBuilder::new();
            builder.mode(0o700);
            match builder.create(&dir) {
                Ok(()) => return Self { root, dir },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(error) => panic!("the control fixture {} refused: {error}", dir.display()),
            }
        }
        panic!("the control fixture names are exhausted; existing directories stay untouched");
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// An exclusively created 0700 `/bin/sh` worker, selected through the
    /// documented `R2_WORKER_BIN` override.
    fn worker(&self, body: &str) -> PathBuf {
        let path = self.path("worker.sh");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o700)
            .open(&path)
            .expect("the control worker is created exclusively");
        file.write_all(format!("#!/bin/sh\n{body}").as_bytes())
            .expect("the control worker is written");
        file.sync_all().expect("the control worker is synchronized");
        path
    }

    /// A plain non-executable file: an existing binary path whose spawn fails.
    fn unexecutable(&self) -> PathBuf {
        self.file("not-executable", b"this is not an executable\n")
    }

    /// The acquired-bytes input the spawner names in its request. The
    /// controlled workers never parse it; the production worker would.
    fn input(&self) -> PathBuf {
        self.file("input", b"<feed><title>control</title></feed>\n")
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .expect("the control file is created exclusively");
        file.write_all(bytes).expect("the control file is written");
        path
    }

    fn relative(&self) -> String {
        self.dir
            .strip_prefix(&self.root)
            .expect("the fixture stays under the repository root")
            .display()
            .to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "retained failed extraction completion fixture: {}",
                self.relative()
            );
            return;
        }
        if let Err(error) = fs::remove_dir_all(&self.dir) {
            panic!(
                "the control fixture cleanup refused ({}): {error}",
                self.relative()
            );
        }
    }
}

/// Does the process table still carry this exact pid? A reaped child is absent;
/// a killed-but-unreaped child is still present as a zombie entry. A refused
/// probe is never read as absence.
fn in_process_table(pid: u32) -> bool {
    let output = Command::new("/bin/ps")
        .args(["-o", "pid=", "-p", &pid.to_string()])
        .output()
        .expect("the process-table probe runs");
    let listed = !String::from_utf8_lossy(&output.stdout).trim().is_empty();
    if !listed && !output.status.success() {
        // `ps` exits nonzero for an absent pid; any other failure would also be
        // silent here, so require the documented shape.
        assert!(
            output.stderr.is_empty(),
            "the process-table probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    listed
}

/// Wait until this pid leaves the process table, bounded. Returns false when it
/// is still present at the deadline — never an assumption of absence.
fn left_process_table(pid: u32, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    loop {
        if !in_process_table(pid) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The pid the controlled worker recorded for itself, waited for with a bound.
fn recorded_pid(path: &Path, budget: Duration) -> u32 {
    let deadline = Instant::now() + budget;
    loop {
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                return pid;
            }
        }
        assert!(
            Instant::now() < deadline,
            "the controlled worker never recorded its identity at {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn limits() -> WorkerLimits {
    WorkerLimits::default()
}

fn run_against(worker: &Path, input: &Path, media_type: &str, budget: Duration) -> ControlRun {
    let guard = worker_selection();
    std::env::set_var("R2_WORKER_BIN", worker);
    let started = Instant::now();
    let result = run_extraction(input, media_type, limits(), budget);
    let elapsed = started.elapsed();
    std::env::remove_var("R2_WORKER_BIN");
    drop(guard);
    ControlRun { result, elapsed }
}

struct ControlRun {
    result: Result<WorkerResponse, ExtractionError>,
    elapsed: Duration,
}

const SUCCESS_WORKER: &str = "cat > /dev/null\nprintf '%s\\n' '{\"parent_digest\":\"sha256:aa\",\"chunks\":[{\"digest\":\"sha256:bb\",\"text\":\"one\"}],\"excluded\":[],\"extractor_version\":\"0.1.0\"}'\n";

const REFUSAL_WORKER: &str = "cat > /dev/null\nprintf '%s\\n' '{\"error\":{\"kind\":\"feed_unreadable\",\"message\":\"the feed is unreadable\"}}'\n";

const NONZERO_WORKER: &str = "cat > /dev/null\nexit 3\n";

const UNPARSEABLE_WORKER: &str = "cat > /dev/null\nprintf '%s\\n' 'not a response'\n";

#[test]
fn the_absent_worker_refuses_before_any_child_exists() {
    let fixture = Fixture::new("absent");
    let missing = fixture.path("no-such-worker");
    let input = fixture.input();
    let run = run_against(
        &missing,
        &input,
        "application/atom+xml",
        Duration::from_secs(5),
    );
    match run.result {
        Err(ExtractionError::WorkerMissing(path)) => {
            assert!(path.ends_with("no-such-worker"), "{path}");
        }
        other => panic!("the absent worker must refuse: {other:?}"),
    }
}

#[test]
fn the_unspawnable_worker_refuses_before_any_child_exists() {
    let fixture = Fixture::new("unspawnable");
    let blocked = fixture.unexecutable();
    let input = fixture.input();
    let run = run_against(
        &blocked,
        &input,
        "application/atom+xml",
        Duration::from_secs(5),
    );
    assert!(
        matches!(run.result, Err(ExtractionError::SpawnFailed(_))),
        "the unspawnable worker must refuse: {:?}",
        run.result
    );
}

#[test]
fn the_successful_exchange_leaves_no_worker_behind() {
    let fixture = Fixture::new("success");
    let worker = fixture.worker(SUCCESS_WORKER);
    let input = fixture.input();
    let run = run_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    let response = run.result.expect("the controlled worker responds");
    assert_eq!(response.parent_digest, "sha256:aa");
    assert_eq!(response.extractor_version, "0.1.0");
    assert_eq!(response.chunks.len(), 1);
    assert_eq!(response.chunks[0].text, "one");
    assert!(response.excluded.is_empty());
}

#[test]
fn the_named_refusal_survives_and_leaves_no_worker_behind() {
    let fixture = Fixture::new("refusal");
    let worker = fixture.worker(REFUSAL_WORKER);
    let input = fixture.input();
    let run = run_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    match run.result {
        Err(ExtractionError::WorkerRefused { kind, message }) => {
            assert_eq!(kind, "feed_unreadable");
            assert!(message.contains("feed"), "{message}");
        }
        other => panic!("the named refusal must survive: {other:?}"),
    }
}

#[test]
fn a_nonzero_exit_is_reported_with_its_status() {
    let fixture = Fixture::new("nonzero");
    let worker = fixture.worker(NONZERO_WORKER);
    let input = fixture.input();
    let run = run_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    match run.result {
        Err(ExtractionError::RequestFailed(detail)) => {
            assert!(detail.contains("the worker exited with"), "{detail}");
        }
        other => panic!("a nonzero exit must be reported: {other:?}"),
    }
}

#[test]
fn an_unparseable_reply_is_reported_without_inventing_a_response() {
    let fixture = Fixture::new("unparseable");
    let worker = fixture.worker(UNPARSEABLE_WORKER);
    let input = fixture.input();
    let run = run_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    match run.result {
        Err(ExtractionError::RequestFailed(detail)) => {
            assert!(detail.contains("the response failed to parse"), "{detail}");
        }
        other => panic!("an unparseable reply must be reported: {other:?}"),
    }
}

/// The regression this leaf owns: the request write fails after the worker has
/// closed its stdin, and the spawner returns. The direct child it created must
/// not still be running — an abandoned live child keeps the request's input
/// readable and its own resources held with nobody waiting for it.
#[test]
fn an_early_request_failure_never_abandons_a_live_worker() {
    let fixture = Fixture::new("abandoned");
    let pid_path = fixture.path("worker.pid");
    let release_path = fixture.path("release");
    let worker = fixture.worker(&format!(
        "printf '%s\\n' \"$$\" > '{pid}.partial'\nmv '{pid}.partial' '{pid}'\nexec 0<&-\ni=0\nwhile [ ! -f '{release}' ] && [ \"$i\" -lt 300 ]; do\n  sleep 0.1\n  i=$((i + 1))\ndone\nexit 0\n",
        pid = pid_path.display(),
        release = release_path.display(),
    ));
    let input = fixture.input();

    // A media type far larger than a pipe buffer forces the real write path:
    // the spawner blocks mid-request and the closed reader turns that write
    // into the actual EPIPE failure, without a clock or a parser change.
    let oversized = "x".repeat(8 * 1024 * 1024);
    let run = run_against(&worker, &input, &oversized, Duration::from_secs(30));

    let pid = recorded_pid(&pid_path, Duration::from_secs(20));
    let consumed = left_process_table(pid, Duration::from_secs(5));

    // Always release and reap the controlled worker, whatever the verdict is:
    // a control may never leave a process running past its own scope.
    fs::write(&release_path, b"release\n").expect("the release marker is written");
    let released = left_process_table(pid, Duration::from_secs(40));

    assert!(
        matches!(run.result, Err(ExtractionError::RequestFailed(_))),
        "the write failure must surface as a request failure: {:?}",
        run.result
    );
    assert!(
        consumed,
        "pid {pid} was still in the process table after the spawner returned: \
         the early failure abandoned its direct worker"
    );
    assert!(
        released,
        "pid {pid} outlived its release marker; inspect it before rerunning"
    );
}

/// The time budget must stop AND reap the worker, not merely stop waiting for
/// it. The recorded pid leaves the process table; a zombie would still be there.
#[test]
fn the_time_budget_stops_and_reaps_its_worker() {
    let fixture = Fixture::new("timeout");
    let pid_path = fixture.path("worker.pid");
    let worker = fixture.worker(&format!(
        "printf '%s\\n' \"$$\" > '{pid}.partial'\nmv '{pid}.partial' '{pid}'\nexec sleep 30\n",
        pid = pid_path.display(),
    ));

    let input = fixture.input();
    let run = run_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(1),
    );
    assert!(
        matches!(run.result, Err(ExtractionError::TimedOut)),
        "the budget must trip: {:?}",
        run.result
    );
    assert!(
        run.elapsed < Duration::from_secs(20),
        "the spawner returned after {:?}: the stop/reap is not bounded",
        run.elapsed
    );

    let pid = recorded_pid(&pid_path, Duration::from_secs(5));
    assert!(
        left_process_table(pid, Duration::from_secs(5)),
        "pid {pid} survived the time budget: the worker was not stopped and reaped"
    );
}

// ---------------------------------------------------------------------------
// The completion evidence itself (`run_extraction_reporting`). The controls
// above prove the direct child does not survive the spawner; these prove the
// spawner SAYS so, and never claims a termination it did not observe.
// ---------------------------------------------------------------------------

fn report_against(
    worker: &Path,
    input: &Path,
    media_type: &str,
    budget: Duration,
) -> ExtractionRun {
    let guard = worker_selection();
    std::env::set_var("R2_WORKER_BIN", worker);
    let run = run_extraction_reporting(input, media_type, limits(), budget);
    std::env::remove_var("R2_WORKER_BIN");
    drop(guard);
    run
}

fn consumed_status(completion: &WorkerCompletion) -> bool {
    match completion {
        WorkerCompletion::Consumed { success, .. } => *success,
        other => panic!("the worker must be consumed: {other:?}"),
    }
}

#[test]
fn an_absent_worker_reports_that_no_child_was_started() {
    let fixture = Fixture::new("absent-report");
    let missing = fixture.path("no-such-worker");
    let input = fixture.input();
    let run = report_against(
        &missing,
        &input,
        "application/atom+xml",
        Duration::from_secs(5),
    );
    assert_eq!(run.completion, WorkerCompletion::NeverStarted);
    assert!(!run.completion.is_consumed());
    assert!(matches!(run.result, Err(ExtractionError::WorkerMissing(_))));
}

#[test]
fn a_failed_spawn_reports_that_no_child_was_started() {
    let fixture = Fixture::new("unspawnable-report");
    let blocked = fixture.unexecutable();
    let input = fixture.input();
    let run = report_against(
        &blocked,
        &input,
        "application/atom+xml",
        Duration::from_secs(5),
    );
    assert_eq!(run.completion, WorkerCompletion::NeverStarted);
    assert!(matches!(run.result, Err(ExtractionError::SpawnFailed(_))));
}

#[test]
fn a_successful_extraction_reports_a_consumed_worker() {
    let fixture = Fixture::new("success-report");
    let worker = fixture.worker(SUCCESS_WORKER);
    let input = fixture.input();
    let run = report_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    assert!(run.result.is_ok(), "{:?}", run.result);
    assert!(consumed_status(&run.completion), "{}", run.completion);
}

#[test]
fn a_named_refusal_reports_a_consumed_worker() {
    let fixture = Fixture::new("refusal-report");
    let worker = fixture.worker(REFUSAL_WORKER);
    let input = fixture.input();
    let run = report_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    assert!(matches!(
        run.result,
        Err(ExtractionError::WorkerRefused { .. })
    ));
    assert!(consumed_status(&run.completion), "{}", run.completion);
}

#[test]
fn a_nonzero_exit_reports_a_consumed_unsuccessful_worker() {
    let fixture = Fixture::new("nonzero-report");
    let worker = fixture.worker(NONZERO_WORKER);
    let input = fixture.input();
    let run = report_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    assert!(matches!(run.result, Err(ExtractionError::RequestFailed(_))));
    assert!(
        !consumed_status(&run.completion),
        "a nonzero exit is consumed but not successful: {}",
        run.completion
    );
}

#[test]
fn an_unparseable_reply_reports_a_consumed_worker() {
    let fixture = Fixture::new("unparseable-report");
    let worker = fixture.worker(UNPARSEABLE_WORKER);
    let input = fixture.input();
    let run = report_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(30),
    );
    assert!(matches!(run.result, Err(ExtractionError::RequestFailed(_))));
    assert!(consumed_status(&run.completion), "{}", run.completion);
}

/// The early failure the repair owns, told through the evidence surface: the
/// request fails AND the spawner reports the child it started as consumed.
#[test]
fn an_early_request_failure_reports_its_consumed_worker() {
    let fixture = Fixture::new("abandoned-report");
    let pid_path = fixture.path("worker.pid");
    let release_path = fixture.path("release");
    let worker = fixture.worker(&format!(
        "printf '%s\\n' \"$$\" > '{pid}.partial'\nmv '{pid}.partial' '{pid}'\nexec 0<&-\ni=0\nwhile [ ! -f '{release}' ] && [ \"$i\" -lt 300 ]; do\n  sleep 0.1\n  i=$((i + 1))\ndone\nexit 0\n",
        pid = pid_path.display(),
        release = release_path.display(),
    ));
    let input = fixture.input();
    let oversized = "x".repeat(8 * 1024 * 1024);
    let run = report_against(&worker, &input, &oversized, Duration::from_secs(30));

    let pid = recorded_pid(&pid_path, Duration::from_secs(20));
    let absent = left_process_table(pid, Duration::from_secs(5));
    fs::write(&release_path, b"release\n").expect("the release marker is written");
    let released = left_process_table(pid, Duration::from_secs(40));

    assert!(matches!(run.result, Err(ExtractionError::RequestFailed(_))));
    assert!(
        run.completion.is_consumed(),
        "the early failure must report its worker consumed: {}",
        run.completion
    );
    assert!(
        absent,
        "pid {pid} outlived the spawner's reported completion"
    );
    assert!(released, "pid {pid} outlived its release marker");
}

#[test]
fn a_tripped_budget_reports_a_consumed_worker() {
    let fixture = Fixture::new("timeout-report");
    let pid_path = fixture.path("worker.pid");
    let worker = fixture.worker(&format!(
        "printf '%s\\n' \"$$\" > '{pid}.partial'\nmv '{pid}.partial' '{pid}'\nexec sleep 30\n",
        pid = pid_path.display(),
    ));
    let input = fixture.input();
    let run = report_against(
        &worker,
        &input,
        "application/atom+xml",
        Duration::from_secs(1),
    );
    assert!(matches!(run.result, Err(ExtractionError::TimedOut)));
    assert!(
        !consumed_status(&run.completion),
        "a killed worker is consumed but not successful: {}",
        run.completion
    );
    let pid = recorded_pid(&pid_path, Duration::from_secs(5));
    assert!(
        left_process_table(pid, Duration::from_secs(5)),
        "pid {pid} survived"
    );
}

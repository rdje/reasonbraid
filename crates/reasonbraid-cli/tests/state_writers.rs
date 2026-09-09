//! Real-process writer controls against an owned loopback HTTP fixture.
//! Request counts describe actual dispatch, not authoritative server effects.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};
use reasonbraid_cli::{StateFile, StoredPrincipal, StoredThread};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const TENANT: &str = "ten_00000000-0000-7000-8000-000000000001";
const ALICE: &str = "hpr_00000000-0000-7000-8000-000000000001";
const BOB: &str = "hpr_00000000-0000-7000-8000-000000000002";
const OLD_THREAD: &str = "thr_00000000-0000-7000-8000-000000000001";
const NEW_THREAD: &str = "thr_00000000-0000-7000-8000-000000000002";

struct Fixture {
    root: PathBuf,
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let root = cwd
            .ancestors()
            .find(|p| p.join("crates/reasonbraid-cli/Cargo.toml").is_file())
            .unwrap()
            .to_path_buf();
        let device = std::fs::metadata(&root).unwrap().dev();
        let base = root.join("target/cli-writer-controls");
        let meta = std::fs::symlink_metadata(root.join("target")).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        match std::fs::create_dir(&base) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(error) => panic!("owned fixture base creation failed: {error}"),
        }
        let meta = std::fs::symlink_metadata(&base).unwrap();
        assert!(meta.is_dir() && !meta.file_type().is_symlink());
        assert_eq!(meta.dev(), device);
        let dir = base.join(format!("writer-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir(&dir).unwrap();
        let fixture = Self { root, dir };
        let mut state = StateFile {
            version: 1,
            ..StateFile::default()
        };
        state.principals.insert(
            "alice".into(),
            StoredPrincipal {
                kind: "human".into(),
                id: ALICE.into(),
                tenant: TENANT.into(),
            },
        );
        state.threads.insert(
            OLD_THREAD.into(),
            StoredThread {
                tenant_id: TENANT.into(),
                subject: "preserve original".into(),
            },
        );
        state.save(&fixture.state_dir()).unwrap();
        fixture
    }

    fn state_dir(&self) -> PathBuf {
        self.dir.join("state")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.dir) {
            if std::thread::panicking() {
                eprintln!("owned writer fixture cleanup failed: {error}");
            } else {
                panic!("owned writer fixture cleanup failed: {error}");
            }
        }
    }
}

#[derive(Clone)]
struct Observation {
    operation: &'static str,
    body: Value,
    principal: Option<String>,
}

type Observed = Arc<Mutex<Vec<Observation>>>;

#[derive(Default)]
struct Gate {
    entered: Notify,
    release: Notify,
}

#[derive(Clone)]
struct Probe {
    requests: Arc<AtomicUsize>,
    gate: Option<Arc<Gate>>,
    observed: Observed,
}

impl Probe {
    async fn record(&self, operation: &'static str, body: &Value, headers: &HeaderMap) {
        let ordinal = self.requests.fetch_add(1, Ordering::SeqCst);
        let principal = headers
            .get(reasonbraid_cli::PRINCIPAL_HEADER)
            .map(|value| value.to_str().unwrap().to_owned());
        self.observed.lock().unwrap().push(Observation {
            operation,
            body: body.clone(),
            principal,
        });
        if let Some(gate) = self.gate.as_ref().filter(|_| ordinal == 0) {
            gate.entered.notify_one();
            gate.release.notified().await;
        }
    }
}

struct Server {
    url: String,
    requests: Arc<AtomicUsize>,
    gate: Option<Arc<Gate>>,
    observed: Observed,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

async fn enroll(
    State(probe): State<Probe>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    probe.record("enroll", &body, &headers).await;
    Json(json!({
        "kind":body["kind"], "name":body["name"], "principal_id":BOB,
        "tenant_id":TENANT, "boundary_id":format!("bnd_{TENANT}"),
        "grant_id":format!("grt_{BOB}"), "replayed":false
    }))
}

async fn create_thread(
    State(probe): State<Probe>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Json<Value> {
    probe.record("create", &body, &headers).await;
    Json(json!({"thread_id":NEW_THREAD,"thread_state":"open"}))
}

impl Server {
    async fn start() -> Self {
        Self::with_gate(false).await
    }

    async fn with_gate(gated: bool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(AtomicUsize::new(0));
        let gate = gated.then(|| Arc::new(Gate::default()));
        let observed = Arc::new(Mutex::new(Vec::new()));
        let router = Router::new()
            .route("/v1/enrollments", post(enroll))
            .route("/v1/threads", post(create_thread))
            .with_state(Probe {
                requests: requests.clone(),
                gate: gate.clone(),
                observed: observed.clone(),
            });
        let (stop, stopped) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
                .unwrap();
        });
        Self {
            url,
            requests,
            gate,
            observed,
            stop: Some(stop),
            task: Some(task),
        }
    }

    async fn finish(mut self) {
        self.release();
        let _ = self.stop.take().unwrap().send(());
        let mut task = self.task.take().unwrap();
        let result = timeout(Duration::from_secs(10), &mut task).await;
        if result.is_err() {
            task.abort();
            let _ = task.await;
        }
        result
            .expect("owned HTTP server shutdown deadline")
            .unwrap();
    }

    fn release(&self) {
        if let Some(gate) = &self.gate {
            gate.release.notify_one();
        }
    }

    async fn entered(&self) -> bool {
        timeout(
            Duration::from_secs(90),
            self.gate.as_ref().unwrap().entered.notified(),
        )
        .await
        .is_ok()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.release();
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

struct Rb {
    child: Child,
    stdout: Option<JoinHandle<Vec<u8>>>,
    stderr: Option<JoinHandle<Vec<u8>>>,
}

impl Rb {
    fn start(fixture: &Fixture, server: &Server, args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rb"))
            .args(args)
            .current_dir(&fixture.root)
            .env("REASONBRAID_CLI_STATE", fixture.state_dir())
            .env("REASONBRAID_SERVER", &server.url)
            .env("NO_PROXY", "127.0.0.1,localhost")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        eprintln!("owned rb pid={:?}, args={args:?}", child.id());
        let mut stdout = child.stdout.take().unwrap().take(65_537);
        let mut stderr = child.stderr.take().unwrap().take(65_537);
        Self {
            child,
            stdout: Some(tokio::spawn(async move {
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes).await.unwrap();
                bytes
            })),
            stderr: Some(tokio::spawn(async move {
                let mut bytes = Vec::new();
                stderr.read_to_end(&mut bytes).await.unwrap();
                bytes
            })),
        }
    }

    async fn finish(mut self) -> (ExitStatus, String, String) {
        let waited = timeout(Duration::from_secs(90), self.child.wait()).await;
        let completed = waited.is_ok();
        let status = match waited {
            Ok(status) => status.unwrap(),
            Err(_) => {
                let _ = self.child.start_kill();
                self.child.wait().await.unwrap()
            }
        };
        let stdout = self.stdout.take().unwrap().await.unwrap();
        let stderr = self.stderr.take().unwrap().await.unwrap();
        assert!(stdout.len() <= 65_536 && stderr.len() <= 65_536);
        let stdout = String::from_utf8(stdout).unwrap();
        let stderr = String::from_utf8(stderr).unwrap();
        assert!(
            completed,
            "owned rb deadline; stdout={stdout}, stderr={stderr}"
        );
        (status, stdout, stderr)
    }

    async fn kill(mut self) -> (ExitStatus, String, String) {
        self.child.start_kill().unwrap();
        self.finish().await
    }
}

impl Drop for Rb {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
        for reader in [&self.stdout, &self.stderr].into_iter().flatten() {
            reader.abort();
        }
    }
}

#[tokio::test]
async fn both_cli_writers_refuse_a_held_state_lock_before_http() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let original = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
    let mut holder = Command::new("python3")
        .args(["-B", "-c", "import fcntl,os,sys; fd=os.open(sys.argv[1],os.O_RDWR); fcntl.flock(fd,fcntl.LOCK_EX); print('locked',flush=True); sys.stdin.buffer.read(1)"])
        .arg(fixture.state_dir().join("state.lock"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .kill_on_drop(true).spawn().unwrap();
    let mut output = BufReader::new(holder.stdout.take().unwrap());
    let mut ready = String::new();
    let readiness = timeout(Duration::from_secs(10), output.read_line(&mut ready)).await;
    if !matches!(readiness, Ok(Ok(_))) || ready.trim() != "locked" {
        let _ = holder.start_kill();
        let _ = holder.wait().await;
        server.finish().await;
        panic!("owned lock holder readiness failed: {readiness:?}, {ready:?}");
    }
    let enroll = Rb::start(&fixture, &server, &["enroll", "human", "bob", "--json"])
        .finish()
        .await;
    let thread = Rb::start(
        &fixture,
        &server,
        &[
            "thread",
            "create",
            "--subject",
            "new",
            "--objective",
            "test exclusion",
            "--as",
            "alice",
            "--json",
        ],
    )
    .finish()
    .await;
    let count = server.requests.load(Ordering::SeqCst);
    holder.start_kill().unwrap();
    let status = holder.wait().await.unwrap();
    server.finish().await;
    assert!(!status.success());
    assert_eq!(
        std::fs::read(fixture.state_dir().join("state.json")).unwrap(),
        original
    );
    eprintln!(
        "held-lock CLI observations: enrollment_ok={}, thread_ok={}, HTTP requests={count}",
        enroll.0.success(),
        thread.0.success()
    );
    assert!(!enroll.0.success() && !thread.0.success());
    assert!(
        enroll.2.contains("another local state writer")
            && thread.2.contains("another local state writer")
    );
    assert_eq!(
        count, 0,
        "a local refusal must precede any HTTP effect request"
    );
}

fn enrollment_args() -> Vec<&'static str> {
    vec!["enroll", "human", "bob", "--json"]
}

fn thread_args() -> Vec<&'static str> {
    vec![
        "thread",
        "create",
        "--subject",
        "new",
        "--objective",
        "test exclusion",
        "--as",
        "alice",
        "--json",
    ]
}

// This test process is independent of rb. Its nonblocking probe closes its own
// descriptor before returning, including an incorrectly acquired lock.
fn lock_is_held(fixture: &Fixture) -> bool {
    let file = std::fs::File::open(fixture.state_dir().join("state.lock")).unwrap();
    match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
        Err(rustix::io::Errno::WOULDBLOCK) => true,
        Ok(()) => false,
        Err(error) => panic!("owned lock observation failed: {error}"),
    }
}

#[tokio::test]
async fn both_writer_orders_hold_exclusion_until_publication_and_merge_fresh_state() {
    for enroll_first in [true, false] {
        let fixture = Fixture::new();
        let server = Server::with_gate(true).await;
        let original = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        let (first_args, second_args) = if enroll_first {
            (enrollment_args(), thread_args())
        } else {
            (thread_args(), enrollment_args())
        };
        let first = Rb::start(&fixture, &server, &first_args);
        if !server.entered().await {
            let result = first.kill().await;
            server.finish().await;
            panic!("first writer did not reach owned HTTP gate: {result:?}");
        }
        let held = lock_is_held(&fixture);
        let before = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        let second = Rb::start(&fixture, &server, &second_args).finish().await;
        let pending_requests = server.requests.load(Ordering::SeqCst);
        server.release();
        let first = first.finish().await;
        let retry = Rb::start(&fixture, &server, &second_args).finish().await;
        let total_requests = server.requests.load(Ordering::SeqCst);
        server.finish().await;
        let state = StateFile::load(&fixture.state_dir()).unwrap();
        eprintln!("overlap: enroll_first={enroll_first}, lock_held={held}, pending_requests={pending_requests}, total={total_requests}");
        assert!(held);
        assert_eq!(before, original);
        assert!(
            !second.0.success() && second.2.contains("another local state writer"),
            "{second:?}"
        );
        assert!(first.0.success(), "{first:?}");
        assert!(retry.0.success(), "{retry:?}");
        assert_eq!((pending_requests, total_requests), (1, 2));
        assert_eq!(state.principals.len(), 2);
        assert_eq!(state.principals["alice"].id, ALICE);
        assert_eq!(state.principals["bob"].id, BOB);
        assert_eq!(state.threads.len(), 2);
        assert_eq!(state.threads[OLD_THREAD].subject, "preserve original");
        assert_eq!(state.threads[NEW_THREAD].subject, "new");
    }
}

#[tokio::test]
async fn killing_either_http_writer_preserves_state_and_releases_exclusion() {
    for enroll_first in [true, false] {
        let fixture = Fixture::new();
        let server = Server::with_gate(true).await;
        let original = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        let args = if enroll_first {
            enrollment_args()
        } else {
            thread_args()
        };
        let writer = Rb::start(&fixture, &server, &args);
        if !server.entered().await {
            let result = writer.kill().await;
            server.finish().await;
            panic!("writer did not reach owned HTTP gate: {result:?}");
        }
        let held = lock_is_held(&fixture);
        let killed = writer.kill().await;
        let released = !lock_is_held(&fixture);
        let observed = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        server.release();
        // This different deliberate operation checks local recovery, not the
        // killed request's server outcome or its replay safety.
        let other = if enroll_first {
            thread_args()
        } else {
            enrollment_args()
        };
        let recovered = Rb::start(&fixture, &server, &other).finish().await;
        server.finish().await;
        eprintln!("crash release: enroll_first={enroll_first}, held={held}, released={released}");
        assert!(held && released);
        assert!(!killed.0.success());
        assert_eq!(observed, original);
        assert!(recovered.0.success(), "{recovered:?}");
        let state = StateFile::load(&fixture.state_dir()).unwrap();
        assert_eq!(state.principals["alice"].id, ALICE);
        assert_eq!(state.threads[OLD_THREAD].subject, "preserve original");
    }
}

#[tokio::test]
async fn named_actor_uses_current_mapping_and_explicit_tenant_override() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let mut state = StateFile::load(&fixture.state_dir()).unwrap();
    state.principals.get_mut("alice").unwrap().id = BOB.into();
    state.save(&fixture.state_dir()).unwrap();
    let tenant_override = "ten_00000000-0000-7000-8000-000000000002";
    let mut args = thread_args();
    args.extend(["--tenant", tenant_override]);
    let result = Rb::start(&fixture, &server, &args).finish().await;
    let observed = server.observed.lock().unwrap().clone();
    server.finish().await;
    assert!(result.0.success(), "{result:?}");
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].operation, "create");
    assert_eq!(observed[0].principal.as_deref(), Some(BOB));
    assert_eq!(observed[0].body["body"]["tenant_id"], tenant_override);
    let state = StateFile::load(&fixture.state_dir()).unwrap();
    assert_eq!(state.threads[NEW_THREAD].tenant_id, tenant_override);
    assert_eq!(state.principals["alice"].tenant, TENANT);
}

#[tokio::test]
async fn explicit_principal_entrypoint_obeys_exclusion_and_releases_usage_failures() {
    use reasonbraid_cli::{run_thread_create, BudgetArgs, Config, CreateProfileArgs, PrincipalRef};

    let fixture = Fixture::new();
    let server = Server::start().await;
    let cfg = Config {
        server_base: server.url.clone(),
        state_dir: fixture.state_dir(),
    };
    let explicit = PrincipalRef {
        id: BOB.into(),
        kind: "human",
        tenant: Some(TENANT.into()),
    };
    let budget = BudgetArgs::default();
    let profile = CreateProfileArgs::default();
    let held = std::fs::File::open(fixture.state_dir().join("state.lock")).unwrap();
    rustix::fs::flock(&held, rustix::fs::FlockOperation::NonBlockingLockExclusive).unwrap();
    let busy = run_thread_create(
        &cfg, &explicit, "new", "test", &budget, &profile, None, None, true,
    )
    .await;
    let blocked_requests = server.requests.load(Ordering::SeqCst);
    drop(held);
    let invalid = PrincipalRef {
        id: ALICE.into(),
        kind: "human",
        tenant: None,
    };
    let usage = run_thread_create(
        &cfg, &invalid, "new", "test", &budget, &profile, None, None, true,
    )
    .await;
    let released = !lock_is_held(&fixture);
    let result = run_thread_create(
        &cfg, &explicit, "new", "test", &budget, &profile, None, None, true,
    )
    .await;
    let observed = server.observed.lock().unwrap().clone();
    server.finish().await;
    assert!(busy
        .unwrap_err()
        .to_string()
        .contains("another local state writer"));
    assert_eq!(blocked_requests, 0);
    assert!(usage.unwrap_err().to_string().contains("no tenant"));
    assert!(released);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(observed.len(), 1);
    assert_eq!(observed[0].principal.as_deref(), Some(BOB));
    let state = StateFile::load(&fixture.state_dir()).unwrap();
    assert_eq!(state.principals["alice"].id, ALICE);
    assert_eq!(state.threads[NEW_THREAD].tenant_id, TENANT);
}

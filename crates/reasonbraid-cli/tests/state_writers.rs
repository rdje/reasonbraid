//! Real-process writer controls against an owned loopback HTTP fixture.
//! Request counts describe actual dispatch, not authoritative server effects.
#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
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

#[derive(Clone, Default)]
enum Reply {
    #[default]
    Normal,
    Status(u16),
    DuplicateKey,
    WrongIdentity,
    MissingGrant,
    Replay,
}

#[derive(Clone)]
struct Probe {
    requests: Arc<AtomicUsize>,
    gate: Option<Arc<Gate>>,
    observed: Observed,
    reply: Arc<Mutex<Reply>>,
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
    reply: Arc<Mutex<Reply>>,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

async fn enroll(
    State(probe): State<Probe>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    probe.record("enroll", &body, &headers).await;
    let mut response = json!({
        "kind":body["kind"], "name":body["name"], "principal_id":BOB,
        "tenant_id":TENANT, "boundary_id":format!("bnd_{TENANT}"),
        "grant_id":format!("grt_{BOB}"), "replayed":false
    });
    if let Some(key) = body.get("bootstrap_request_id") {
        response["bootstrap_request_id"] = key.clone();
    }
    match probe.reply.lock().unwrap().clone() {
        Reply::Normal => Json(response).into_response(),
        Reply::Status(status) => (
            StatusCode::from_u16(status).unwrap(),
            Json(json!({
                "code":"dependency_unavailable", "message":"owned uncertain bootstrap response"
            })),
        )
            .into_response(),
        Reply::DuplicateKey => {
            let raw = serde_json::to_string(&response).unwrap();
            let duplicate = format!(
                "{{\"bootstrap_request_id\":{},{}",
                body["bootstrap_request_id"],
                &raw[1..]
            );
            (
                StatusCode::OK,
                [("content-type", "application/json")],
                duplicate,
            )
                .into_response()
        }
        Reply::WrongIdentity => {
            response["principal_id"] = json!(ALICE);
            Json(response).into_response()
        }
        Reply::MissingGrant => {
            response.as_object_mut().unwrap().remove("grant_id");
            Json(response).into_response()
        }
        Reply::Replay => {
            response["replayed"] = json!(true);
            Json(response).into_response()
        }
    }
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
        let reply = Arc::new(Mutex::new(Reply::Normal));
        let router = Router::new()
            .route("/v1/enrollments", post(enroll))
            .route("/v1/threads", post(create_thread))
            .with_state(Probe {
                requests: requests.clone(),
                gate: gate.clone(),
                observed: observed.clone(),
                reply: reply.clone(),
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
            reply,
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
    fn command(fixture: &Fixture, server: &Server, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rb"));
        command
            .args(args)
            .current_dir(&fixture.root)
            .env("REASONBRAID_CLI_STATE", fixture.state_dir())
            .env("REASONBRAID_SERVER", &server.url)
            .env("NO_PROXY", "127.0.0.1,localhost")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        command
    }

    fn start(fixture: &Fixture, server: &Server, args: &[&str]) -> Self {
        let mut child = Self::command(fixture, server, args).spawn().unwrap();
        eprintln!(
            "owned rb pid={:?}, verb={:?}, argument bytes={}",
            child.id(),
            args.first(),
            args.iter().map(|arg| arg.len()).sum::<usize>()
        );
        let mut stdout = child.stdout.take().unwrap().take(1_048_577);
        let mut stderr = child.stderr.take().unwrap().take(1_048_577);
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
        assert!(stdout.len() <= 1_048_576 && stderr.len() <= 1_048_576);
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

#[tokio::test]
async fn bootstrap_dispatch_has_a_durable_matching_pending_request() {
    let fixture = Fixture::new();
    let server = Server::with_gate(true).await;
    let writer = Rb::start(&fixture, &server, &enrollment_args());
    if !server.entered().await {
        let result = writer.kill().await;
        server.finish().await;
        panic!("bootstrap did not reach owned HTTP gate: {result:?}");
    }
    let at_dispatch = StateFile::load(&fixture.state_dir()).unwrap();
    let observed = server.observed.lock().unwrap().clone();
    let held = lock_is_held(&fixture);
    server.release();
    let result = writer.finish().await;
    server.finish().await;
    eprintln!(
        "bootstrap dispatch: version={}, recovery={}, request_key={}, lock_held={held}",
        at_dispatch.version,
        at_dispatch.bootstrap.is_some(),
        observed[0].body.get("bootstrap_request_id").is_some()
    );
    assert!(result.0.success(), "{result:?}");
    assert!(held);
    let pending = at_dispatch.bootstrap.unwrap().pending.unwrap();
    assert_eq!(at_dispatch.version, 2);
    assert_eq!(pending.name, "bob");
    assert_eq!(pending.request_id, observed[0].body["bootstrap_request_id"]);
    assert_eq!(at_dispatch.principals.len(), 1);
    assert_eq!(at_dispatch.threads.len(), 1);
    let final_state = StateFile::load(&fixture.state_dir()).unwrap();
    let recovery = final_state.bootstrap.unwrap();
    assert!(recovery.pending.is_none());
    assert_eq!(recovery.completed.unwrap().request, pending);
    assert_eq!(final_state.principals["bob"].id, BOB);
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
        if enroll_first {
            let before: Value = serde_json::from_slice(&before).unwrap();
            let original: Value = serde_json::from_slice(&original).unwrap();
            assert_eq!(before["principals"], original["principals"]);
            assert_eq!(before["threads"], original["threads"]);
            assert_eq!(before["bootstrap"]["pending"]["name"], "bob");
        } else {
            assert_eq!(before, original);
        }
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
        // Bootstrap interruption must recover its saved key before a different
        // writer. The fixture observes dispatch identity, not a database commit.
        let bootstrap_recovery = if enroll_first {
            Some(
                Rb::start(&fixture, &server, &enrollment_args())
                    .finish()
                    .await,
            )
        } else {
            None
        };
        let other = if enroll_first {
            thread_args()
        } else {
            enrollment_args()
        };
        let recovered = Rb::start(&fixture, &server, &other).finish().await;
        let requests = server.observed.lock().unwrap().clone();
        server.finish().await;
        eprintln!("crash release: enroll_first={enroll_first}, held={held}, released={released}");
        assert!(held && released);
        assert!(!killed.0.success());
        if let Some(result) = bootstrap_recovery {
            assert!(result.0.success(), "{result:?}");
            let observed: Value = serde_json::from_slice(&observed).unwrap();
            let original: Value = serde_json::from_slice(&original).unwrap();
            assert_eq!(observed["principals"], original["principals"]);
            assert_eq!(observed["threads"], original["threads"]);
            assert_eq!(
                requests[0].body["bootstrap_request_id"],
                requests[1].body["bootstrap_request_id"]
            );
            assert_eq!(
                observed["bootstrap"]["pending"]["request_id"],
                requests[0].body["bootstrap_request_id"]
            );
        } else {
            assert_eq!(observed, original);
        }
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

#[tokio::test]
async fn a_different_pending_bootstrap_blocks_ordinary_writers_before_http() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let mut state = StateFile::load(&fixture.state_dir()).unwrap();
    state.version = 2;
    state.bootstrap = Some(reasonbraid_cli::BootstrapRecovery {
        pending: Some(reasonbraid_cli::BootstrapRequest {
            request_id: "req_00000000-0000-7000-8000-000000000001".into(),
            server: server.url.clone(),
            name: "alice".into(),
            actions: None,
        }),
        completed: None,
    });
    state.save(&fixture.state_dir()).unwrap();
    let original = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
    let enroll = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    let thread = Rb::start(&fixture, &server, &thread_args()).finish().await;
    let count = server.requests.load(Ordering::SeqCst);
    server.finish().await;
    assert!(!enroll.0.success() && !thread.0.success());
    assert!(enroll.2.contains("pending") && thread.2.contains("pending"));
    assert_eq!(count, 0);
    assert_eq!(
        std::fs::read(fixture.state_dir().join("state.json")).unwrap(),
        original
    );
}

#[tokio::test]
async fn failed_bootstrap_replies_preserve_one_key_and_the_original_request() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let original = StateFile::load(&fixture.state_dir()).unwrap();
    let mut saved = None;
    for reply in [
        Reply::Status(500),
        Reply::DuplicateKey,
        Reply::WrongIdentity,
        Reply::MissingGrant,
    ] {
        *server.reply.lock().unwrap() = reply;
        let args = if saved.is_none() {
            vec![
                "enroll",
                "human",
                "bob",
                "--actions",
                "original_ignored_action",
                "--json",
            ]
        } else {
            vec![
                "enroll",
                "human",
                "bob",
                "--actions",
                "different_ignored_action",
                "--resume-bootstrap",
                "--json",
            ]
        };
        let result = Rb::start(&fixture, &server, &args).finish().await;
        assert!(!result.0.success(), "{result:?}");
        assert!(!lock_is_held(&fixture));
        let bytes = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        if let Some(previous) = &saved {
            assert_eq!(&bytes, previous);
        } else {
            saved = Some(bytes);
        }
        let state = StateFile::load(&fixture.state_dir()).unwrap();
        assert_eq!(
            serde_json::to_value(&state.principals).unwrap(),
            serde_json::to_value(&original.principals).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&state.threads).unwrap(),
            serde_json::to_value(&original.threads).unwrap()
        );
        assert!(state.bootstrap.unwrap().completed.is_none());
    }
    *server.reply.lock().unwrap() = Reply::Replay;
    let result = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    let observed = server.observed.lock().unwrap().clone();
    server.finish().await;
    assert!(result.0.success(), "{result:?}");
    let output: Value = serde_json::from_str(&result.1).unwrap();
    assert_eq!(output["replayed"], true);
    assert_eq!(output["recovery_source"], "server");
    assert_eq!(observed.len(), 5);
    for request in &observed {
        assert_eq!(request.body, observed[0].body);
        assert_eq!(request.body["actions"], json!(["original_ignored_action"]));
    }
    let recovery = StateFile::load(&fixture.state_dir())
        .unwrap()
        .bootstrap
        .unwrap();
    assert!(recovery.pending.is_none());
    let completed = recovery.completed.unwrap();
    assert_eq!(
        completed.request.request_id,
        observed[0].body["bootstrap_request_id"]
    );
    assert_eq!(completed.outcome.principal_id, BOB);
    assert!(completed.outcome.replayed);
}

#[tokio::test]
async fn explicit_resume_uses_a_historical_receipt_and_normal_invocation_is_fresh() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let missing = Rb::start(
        &fixture,
        &server,
        &["enroll", "human", "bob", "--resume-bootstrap"],
    )
    .finish()
    .await;
    assert!(!missing.0.success() && missing.2.contains("no matching"));
    assert_eq!(server.requests.load(Ordering::SeqCst), 0);
    let first = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    assert!(first.0.success(), "{first:?}");
    let first: Value = serde_json::from_str(&first.1).unwrap();
    let mut state = StateFile::load(&fixture.state_dir()).unwrap();
    // An ordinary later name mapping must not silently replace the historical receipt.
    state.principals.get_mut("bob").unwrap().id = ALICE.into();
    state.save(&fixture.state_dir()).unwrap();
    *server.reply.lock().unwrap() = Reply::Status(500);
    let resumed = Rb::start(
        &fixture,
        &server,
        &["enroll", "human", "bob", "--resume-bootstrap", "--json"],
    )
    .finish()
    .await;
    assert!(resumed.0.success(), "{resumed:?}");
    let resumed: Value = serde_json::from_str(&resumed.1).unwrap();
    assert_eq!(resumed["recovery_source"], "local_receipt");
    assert_eq!(
        resumed["bootstrap_request_id"],
        first["bootstrap_request_id"]
    );
    assert_eq!(resumed["replayed"], first["replayed"]);
    assert_eq!(
        StateFile::load(&fixture.state_dir()).unwrap().principals["bob"].id,
        BOB
    );
    assert_eq!(server.requests.load(Ordering::SeqCst), 1);
    // Recreate the valid completion-before-cleanup phase through the public store.
    let mut state = StateFile::load(&fixture.state_dir()).unwrap();
    let recovery = state.bootstrap.as_mut().unwrap();
    recovery.pending = Some(recovery.completed.as_ref().unwrap().request.clone());
    state.save(&fixture.state_dir()).unwrap();
    let cleanup = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    assert!(cleanup.0.success(), "{cleanup:?}");
    assert_eq!(
        serde_json::from_str::<Value>(&cleanup.1).unwrap()["recovery_source"],
        "local_receipt"
    );
    assert_eq!(server.requests.load(Ordering::SeqCst), 1);
    *server.reply.lock().unwrap() = Reply::Normal;
    let fresh = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    let observed = server.observed.lock().unwrap().clone();
    server.finish().await;
    assert!(fresh.0.success(), "{fresh:?}");
    assert_eq!(observed.len(), 2);
    assert_ne!(
        observed[0].body["bootstrap_request_id"],
        observed[1].body["bootstrap_request_id"]
    );
}

#[tokio::test]
async fn invalid_recovery_modes_and_endpoint_conflicts_do_not_dispatch_or_change_state() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let other = Server::start().await;
    *server.reply.lock().unwrap() = Reply::Status(500);
    let first = Rb::start(&fixture, &server, &enrollment_args())
        .finish()
        .await;
    assert!(!first.0.success());
    let before = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
    for args in [
        vec![
            "enroll",
            "role",
            "bob",
            "--tenant",
            TENANT,
            "--resume-bootstrap",
        ],
        vec![
            "enroll",
            "human",
            "bob",
            "--tenant",
            TENANT,
            "--resume-bootstrap",
        ],
        vec!["enroll", "human", "different", "--resume-bootstrap"],
        vec![
            "enroll",
            "human",
            "bob",
            "--resume-bootstrap",
            "--server",
            &other.url,
        ],
        vec![
            "enroll",
            "human",
            "bob",
            "--server",
            "http://SECRET:password@127.0.0.1:1",
        ],
    ] {
        let result = Rb::start(&fixture, &server, &args).finish().await;
        assert!(!result.0.success(), "{result:?}");
        assert!(!result.2.contains("SECRET") && !result.2.contains("password"));
        assert_eq!(
            std::fs::read(fixture.state_dir().join("state.json")).unwrap(),
            before
        );
        assert!(!lock_is_held(&fixture));
    }
    let counts = (
        server.requests.load(Ordering::SeqCst),
        other.requests.load(Ordering::SeqCst),
    );
    server.finish().await;
    other.finish().await;
    assert_eq!(counts, (1, 0));
}

#[tokio::test]
async fn losing_the_process_with_unread_output_recovers_the_completed_receipt() {
    let fixture = Fixture::new();
    let server = Server::start().await;
    let name = "x".repeat(96 * 1024);
    let mut child = Rb::command(&fixture, &server, &["enroll", "human", &name, "--json"])
        .spawn()
        .unwrap();
    eprintln!("owned unread-output rb pid={:?}", child.id());
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    // Deliberately leave stdout unread. Observe both a completed state snapshot
    // and queued pipe bytes before interrupting this exact owned process.
    let ready = timeout(Duration::from_secs(90), async {
        loop {
            let state = StateFile::load(&fixture.state_dir()).unwrap();
            if state.bootstrap.as_ref().is_some_and(|recovery| {
                recovery.pending.is_none()
                    && recovery
                        .completed
                        .as_ref()
                        .is_some_and(|done| done.request.name == name)
            }) {
                let queued = rustix::io::ioctl_fionread(&stdout).unwrap();
                if queued > 0 {
                    break (state, queued);
                }
            }
            if child.try_wait().unwrap().is_some() {
                panic!("owned output process exited before the unread-output observation");
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let alive = child.try_wait().unwrap().is_none();
    let queued_after = rustix::io::ioctl_fionread(&stdout).unwrap();
    if alive {
        child.start_kill().unwrap();
    }
    let status = timeout(Duration::from_secs(10), child.wait())
        .await
        .expect("owned interrupted output process reap deadline")
        .unwrap();
    let mut partial = Vec::new();
    stdout
        .take(1_048_577)
        .read_to_end(&mut partial)
        .await
        .unwrap();
    let mut errors = Vec::new();
    stderr
        .take(1_048_577)
        .read_to_end(&mut errors)
        .await
        .unwrap();
    assert!(partial.len() <= 1_048_576 && errors.len() <= 1_048_576);
    let result = Rb::start(
        &fixture,
        &server,
        &["enroll", "human", &name, "--resume-bootstrap", "--json"],
    )
    .finish()
    .await;
    let count = server.requests.load(Ordering::SeqCst);
    server.finish().await;
    let (published, queued_before) = ready.expect("owned output observation deadline");
    eprintln!("unread output: queued before={queued_before}, after={queued_after}, process alive={alive}, partial bytes={}, HTTP count={count}", partial.len());
    assert!(alive && !status.success());
    assert!(queued_after >= queued_before && queued_after < name.len() as u64);
    assert!(
        serde_json::from_slice::<Value>(&partial).is_err(),
        "the owned reader did not receive a complete JSON result"
    );
    assert!(result.0.success(), "recovery failed: {}", result.2);
    let output: Value = serde_json::from_str(&result.1).unwrap();
    let completed = published.bootstrap.unwrap().completed.unwrap();
    assert_eq!(output["recovery_source"], "local_receipt");
    assert_eq!(output["bootstrap_request_id"], completed.request.request_id);
    assert_eq!(output["principal_id"], completed.outcome.principal_id);
    assert_eq!(output["name"], name);
    assert_eq!(count, 1);
}

#[tokio::test]
async fn bootstrap_checks_completion_capacity_before_dispatch_at_the_exact_bound() {
    use reasonbraid_cli::{
        BootstrapOutcome, BootstrapRecovery, BootstrapRequest, CompletedBootstrap,
    };

    const LIMIT: usize = 8 * 1024 * 1024;
    let mut observations = Vec::new();
    for (excess, existing_pending) in [(1, false), (1, true), (0, false)] {
        let fixture = Fixture::new();
        let server = Server::start().await;
        let mut original = StateFile::load(&fixture.state_dir()).unwrap();
        original
            .threads
            .get_mut(OLD_THREAD)
            .unwrap()
            .subject
            .clear();
        let request = BootstrapRequest {
            request_id: "req_00000000-0000-7000-8000-000000000001".into(),
            server: server.url.clone(),
            name: "bob".into(),
            actions: None,
        };
        let outcome = BootstrapOutcome {
            bootstrap_request_id: request.request_id.clone(),
            kind: "human".into(),
            name: "bob".into(),
            principal_id: BOB.into(),
            tenant_id: TENANT.into(),
            boundary_id: format!("bnd_{TENANT}"),
            grant_id: format!("grt_{BOB}"),
            replayed: false,
        };
        let mut completion = original.clone();
        completion.version = 2;
        completion.principals.insert(
            "bob".into(),
            StoredPrincipal {
                kind: "human".into(),
                id: BOB.into(),
                tenant: TENANT.into(),
            },
        );
        completion.bootstrap = Some(BootstrapRecovery {
            pending: Some(request.clone()),
            completed: Some(CompletedBootstrap {
                request: request.clone(),
                outcome,
            }),
        });
        let payload = LIMIT + excess - serde_json::to_vec_pretty(&completion).unwrap().len();
        original.threads.get_mut(OLD_THREAD).unwrap().subject = "x".repeat(payload);
        completion.threads.get_mut(OLD_THREAD).unwrap().subject = "x".repeat(payload);
        assert_eq!(
            serde_json::to_vec_pretty(&completion).unwrap().len(),
            LIMIT + excess
        );
        let mut pending = original.clone();
        pending.version = 2;
        pending.bootstrap = Some(BootstrapRecovery {
            pending: Some(request),
            completed: None,
        });
        let pending_bytes = serde_json::to_vec_pretty(&pending).unwrap().len();
        assert!(pending_bytes < LIMIT);
        original.save(&fixture.state_dir()).unwrap();
        if existing_pending {
            pending.save(&fixture.state_dir()).unwrap();
        }
        let before = std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        let result = Rb::start(&fixture, &server, &enrollment_args())
            .finish()
            .await;
        let count = server.requests.load(Ordering::SeqCst);
        server.finish().await;
        let preserved = before == std::fs::read(fixture.state_dir().join("state.json")).unwrap();
        let state = StateFile::load(&fixture.state_dir()).unwrap();
        let maps_intact = state.principals["alice"].id == ALICE
            && state.threads[OLD_THREAD].subject == original.threads[OLD_THREAD].subject;
        let released = !lock_is_held(&fixture);
        if excess > 0 {
            assert!(
                result.2.contains("preflight refused before HTTP"),
                "{}",
                result.2
            );
        }
        eprintln!("capacity: completion_excess={excess}, existing_pending={existing_pending}, pending_bytes={pending_bytes}, HTTP={count}, success={}, original_preserved={preserved}, maps_intact={maps_intact}, lock_released={released}, error={}", result.0.success(), result.2.trim());
        observations.push((
            excess,
            existing_pending,
            count,
            result.0.success(),
            preserved,
            maps_intact,
            released,
        ));
    }
    // All process/server lifetimes and all unique fixture cleanups complete
    // before the desired baseline assertion, including the fitting control.
    assert_eq!(
        observations,
        vec![
            (1, false, 0, false, true, true, true),
            (1, true, 0, false, true, true, true),
            (0, false, 1, true, false, true, true)
        ]
    );
}

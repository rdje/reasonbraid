//! The inspection console in a real browser (`SIGNOFF-REPAIR.11.1.1`).
//!
//! The console's other controls (`src/ui.rs`) read `app.js` as TEXT: which
//! routes it names, that it never writes `innerHTML`, that its inbox query parses
//! with the route's own extractor. None of them RUNS it, and so the Timeline view
//! threw for every thread that had an event — `el()` appended the numeric
//! `aggregate_version` as though it were a DOM node — while every check passed.
//!
//! These controls serve the real API and console over a disposable PostgreSQL
//! and drive the pinned Chrome through the page the way an operator does: save
//! an identity, open a thread, show a view. What the page shows is compared with
//! what the server returned for the same read.
//!
//! Run with
//! `python3 -B scripts/ci_browser.py -- bash scripts/run_pg_tests.sh console_browser`.
//! Without `R3_BROWSER_BIN` they skip and say so; those that need a server's
//! data skip without `DATABASE_URL` too, as every database suite does. Each of
//! those enrolls its own tenant and reads only inside it (the inbox control
//! seeds one row, in its own tenant), so the suite asserts over no shared table
//! and needs no cleanup plan.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/browser.rs"]
mod browser_support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use browser_support::{browser_binary, skip_without_browser, Chrome};
use chromiumoxide::Page;
use reasonbraid_core::{ClientContext, CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::ca::ensure_server_ca;
use reasonbraid_server::{api_router, node_router, ui_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::sync::oneshot;

/// A view is given this long to finish rendering after its button is clicked.
const RENDER: Duration = Duration::from_secs(15);

static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    Some(pool)
}

/// The console and the routes it reads, on one loopback origin — what
/// `rb-server` serves at `/`, `/v1/` and `/v1/nodes/presence`.
struct Console {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
    hold: Hold,
}

impl Console {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().expect("local address");
        let ca = Arc::new(ensure_server_ca(pool).await.expect("server CA"));
        let hold = Hold::default();
        let app = api_router(pool.clone())
            .merge(node_router(pool.clone(), ca))
            .merge(ui_router())
            .layer(from_fn_with_state(hold.clone(), held));
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        Self { addr, handle, hold }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// One request the test holds back, so a slow answer is produced on demand
/// rather than hoped for: the next request whose path and query contain the
/// armed text announces itself and waits until the test releases it.
#[derive(Clone, Default)]
struct Hold(Arc<Mutex<Option<Armed>>>);

struct Armed {
    needle: String,
    parked: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

/// The test's two ends of a held request.
struct Held {
    parked: oneshot::Receiver<()>,
    release: oneshot::Sender<()>,
}

impl Hold {
    fn arm(&self, needle: &str) -> Held {
        let (parked, parked_rx) = oneshot::channel();
        let (release_tx, release) = oneshot::channel();
        *self.0.lock().expect("the hold") = Some(Armed {
            needle: needle.to_owned(),
            parked,
            release,
        });
        Held {
            parked: parked_rx,
            release: release_tx,
        }
    }
}

impl Held {
    async fn parked(&mut self) {
        tokio::time::timeout(RENDER, &mut self.parked)
            .await
            .expect("the held request arrives")
            .expect("the hold announces it");
    }
}

async fn held(State(hold): State<Hold>, request: Request, next: Next) -> Response {
    let target = request
        .uri()
        .path_and_query()
        .map_or_else(String::new, |target| target.as_str().to_owned());
    let armed = {
        let mut slot = hold.0.lock().expect("the hold");
        match slot.as_ref() {
            Some(armed) if target.contains(&armed.needle) => slot.take(),
            _ => None,
        }
    };
    if let Some(armed) = armed {
        let _ = armed.parked.send(());
        let _ = armed.release.await;
    }
    next.run(request).await
}

impl Drop for Console {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// An operator: a fresh tenant, its administrator, and one of its threads.
struct Operator {
    principal: String,
    tenant: String,
    thread: String,
}

async fn operator(base: &str, name: &str, subject: &str, objective: &str) -> Operator {
    let client = reqwest::Client::new();
    let enrolled: Value = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({ "kind": "human", "name": name }))
        .send()
        .await
        .expect("enroll")
        .json()
        .await
        .expect("enrollment json");
    let principal = enrolled["principal_id"]
        .as_str()
        .unwrap_or_else(|| panic!("an enrolled principal: {enrolled}"))
        .to_owned();
    let tenant = enrolled["tenant_id"]
        .as_str()
        .expect("an enrolled tenant")
        .to_owned();
    let envelope = CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: "thread.create".to_string(),
        request_id: RequestId::new(),
        idempotency_key: format!("console-{name}"),
        expected_aggregate_version: None,
        body: json!({ "tenant_id": tenant, "subject": subject, "objective": objective }),
        authority_context: None,
        client_context: ClientContext::default(),
    };
    let created: Value = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &principal)
        .json(&envelope)
        .send()
        .await
        .expect("create a thread")
        .json()
        .await
        .expect("thread json");
    let thread = created["thread_id"]
        .as_str()
        .unwrap_or_else(|| panic!("a created thread: {created}"))
        .to_owned();
    Operator {
        principal,
        tenant,
        thread,
    }
}

/// The server's own answer to one of the reads the console makes.
async fn read(base: &str, who: &Operator, path: &str) -> Value {
    let response = reqwest::Client::new()
        .get(format!("{base}{path}?tenant_id={}", who.tenant))
        .header(PRINCIPAL_HEADER, &who.principal)
        .send()
        .await
        .expect("read");
    assert_eq!(response.status().as_u16(), 200, "GET {path}");
    response.json().await.expect("read json")
}

/// What `#output` holds once a view has rendered.
#[derive(Debug, serde::Deserialize)]
struct Rendered {
    headings: Vec<String>,
    tables: usize,
    errors: Vec<String>,
    rows: Vec<Vec<String>>,
    text: String,
    /// Elements a DATUM would have become had any view parsed data as markup.
    markup: usize,
    /// Set only by the markup the inertness control plants in thread content.
    ran: Option<String>,
}

/// Wait until `#output` shows `heading` and something beside it, then read it.
///
/// Every view appends its heading before its one `await` and everything else
/// after it, in one synchronous run, and `render()` appends its error the same
/// way. So a second child beside the expected heading means the view has
/// finished, whichever way it ended. The count is taken in the heading's own
/// parent, which is `#output` itself or the view's container inside it.
async fn rendered(page: &Page, heading: &str) -> Rendered {
    const SNAPSHOT: &str = r#"(() => {
        const out = document.getElementById("output");
        const h2 = out.querySelector("h2");
        return JSON.stringify({
            heading: h2 ? h2.textContent : "",
            children: h2 ? h2.parentElement.children.length : 0,
            headings: [...out.querySelectorAll("h2")].map((h) => h.textContent),
            tables: out.querySelectorAll("table").length,
            errors: [...out.querySelectorAll(".error")].map((e) => e.textContent),
            rows: [...out.querySelectorAll("tbody tr")].map((tr) =>
                [...tr.cells].map((td) => td.textContent)),
            text: out.textContent,
            markup: out.querySelectorAll("img, script, iframe, object, embed").length,
            ran: window.__rbRan === undefined ? null : String(window.__rbRan),
        });
    })()"#;
    let deadline = tokio::time::Instant::now() + RENDER;
    loop {
        let snapshot: String = page
            .evaluate(SNAPSHOT)
            .await
            .expect("read the page")
            .into_value()
            .expect("a snapshot string");
        let value: Value = serde_json::from_str(&snapshot).expect("snapshot json");
        if value["heading"] == heading && value["children"].as_u64() > Some(1) {
            return serde_json::from_value(value).expect("a rendered view");
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "`{heading}` did not render within {RENDER:?}; the page shows {value}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Wait until the page has RECEIVED the response to a request naming `needle`
/// (its resource-timing entry is complete), then give its handlers a moment.
///
/// ⭐ This is what makes a stale-view control able to fail: reading the page
/// before the late answer arrives would pass whether or not it is dropped. The
/// wait is proven long enough by the RED, where the same wait sees the stale
/// rows land.
async fn received(page: &Page, needle: &str) {
    let probe = format!(
        "performance.getEntriesByType('resource').some((e) => e.name.includes({}) && e.responseEnd > 0)",
        serde_json::to_string(needle).expect("a JS string")
    );
    let deadline = tokio::time::Instant::now() + RENDER;
    loop {
        let done: bool = page
            .evaluate(probe.as_str())
            .await
            .expect("read the page's resource timing")
            .into_value()
            .expect("a boolean");
        if done {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the page never received the response to `{needle}`"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
}

/// Replace a text field's content the way an operator does: clear, then type.
async fn refill(page: &Page, selector: &str, value: &str) {
    page.evaluate(format!(
        "document.querySelector({}).value = ''",
        serde_json::to_string(selector).expect("a JS string")
    ))
    .await
    .expect("clear the field");
    page.find_element(selector)
        .await
        .expect("the field")
        .click()
        .await
        .expect("focus the field")
        .type_str(value)
        .await
        .expect("type into the field");
}

/// Read the view under `heading` once its text contains `needle`.
async fn showing(page: &Page, heading: &str, needle: &str) -> Rendered {
    let deadline = tokio::time::Instant::now() + RENDER;
    loop {
        let view = rendered(page, heading).await;
        if view.text.contains(needle) {
            return view;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "`{heading}` never showed `{needle}`: {view:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn click(page: &Page, selector: &str) {
    page.find_element(selector)
        .await
        .unwrap_or_else(|e| panic!("find `{selector}`: {e}"))
        .click()
        .await
        .unwrap_or_else(|e| panic!("click `{selector}`: {e}"));
}

/// Save the identity through the form, as an operator types it, and open the
/// operator's thread from the Threads view.
async fn sign_in_and_open_thread(page: &Page, who: &Operator) -> Rendered {
    for (selector, value) in [("#principal", &who.principal), ("#tenant", &who.tenant)] {
        page.find_element(selector)
            .await
            .expect("an identity field")
            .click()
            .await
            .expect("focus the field")
            .type_str(value)
            .await
            .expect("type into the field");
    }
    click(page, "#auth-form button[type=submit]").await;
    let threads = rendered(page, "Threads").await;
    assert!(threads.errors.is_empty(), "{threads:?}");
    assert!(
        threads.rows.iter().any(|row| row[0] == who.thread),
        "the Threads view lists the operator's thread: {threads:?}"
    );
    click(page, "#output button.link").await;
    rendered(page, "Thread").await
}

/// The Timeline and the Audit, the two views the goal line names, render in a
/// real browser — and show, cell for cell, what the server returned.
#[tokio::test]
async fn the_timeline_and_audit_render_what_the_server_returned() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's Timeline and Audit views");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    let who = operator(
        &base,
        "console-timeline",
        "timeline probe",
        "render every view",
    )
    .await;
    let events = read(&base, &who, &format!("/v1/threads/{}/events", who.thread)).await;
    let events = events["events"].as_array().expect("an event list").clone();
    assert!(!events.is_empty(), "the control needs a thread WITH events");

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    let thread = sign_in_and_open_thread(&page, &who).await;
    assert!(thread.errors.is_empty(), "{thread:?}");

    click(&page, "button[data-view=events]").await;
    let timeline = rendered(&page, "Timeline (events)").await;
    assert!(
        timeline.errors.is_empty(),
        "the Timeline renders without error: {:?}",
        timeline.errors
    );
    assert_eq!(timeline.rows.len(), events.len(), "one row per event");
    for (row, event) in timeline.rows.iter().zip(&events) {
        assert_eq!(
            row[..3],
            [
                event["aggregate_version"].to_string(),
                event["event_type"].as_str().expect("a type").to_owned(),
                event["committed_at"].as_str().expect("a time").to_owned(),
            ],
            "the row shows the event's version, type and time"
        );
        let body: Value = serde_json::from_str(&row[3]).expect("the body cell is JSON");
        assert_eq!(body, event["body"], "the row shows the event's body");
    }

    click(&page, "button[data-view=audit]").await;
    let audit = rendered(&page, "Audit (authorization records)").await;
    assert!(
        audit.errors.is_empty(),
        "the Audit renders: {:?}",
        audit.errors
    );
    assert!(
        !audit.rows.is_empty(),
        "every read writes a record: {audit:?}"
    );
    // Reading the audit writes a record, so the server's later answer holds
    // every row the page showed, and more.
    let records = read(&base, &who, &format!("/v1/threads/{}/audit", who.thread)).await;
    let records = records["records"].as_array().expect("a record list");
    for row in &audit.rows {
        let record = records
            .iter()
            .find(|r| r["record_id"] == row[0].as_str())
            .unwrap_or_else(|| panic!("the page shows a record the server holds: {row:?}"));
        let expected: Vec<String> = ["record_id", "decision", "action", "actor", "policy_digest"]
            .iter()
            .map(|field| record[field].as_str().expect("a text field").to_owned())
            .collect();
        assert_eq!(row, &expected, "the row shows the record");
    }

    page.close().await.expect("close the page");
    chrome.finish().await;
}

/// ⭐ Why the census withdrew class 1, checked where it matters: thread content
/// that IS markup stays text in every view that shows it, and none of it runs.
#[tokio::test]
async fn markup_in_thread_content_renders_as_text_and_never_runs() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's inert rendering of thread content");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    // Unquoted so that the Timeline's JSON rendering shows them verbatim too. The
    // image fires `onerror` the moment a parser builds it; the script would run
    // only if inserted as an element, which `innerHTML` never does.
    let subject = "<img src=x onerror=window.__rbRan=document.title>";
    let objective = "<script>window.__rbRan='objective'</script>";
    let who = operator(&base, "console-inert", subject, objective).await;

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    let thread = sign_in_and_open_thread(&page, &who).await;
    click(&page, "button[data-view=threads]").await;
    let threads = rendered(&page, "Threads").await;
    click(&page, "#output button.link").await;
    let _ = rendered(&page, "Thread").await;
    click(&page, "button[data-view=events]").await;
    let timeline = rendered(&page, "Timeline (events)").await;

    for (view, shown, content) in [
        ("Threads", &threads, vec![subject]),
        ("Thread", &thread, vec![subject, objective]),
        ("Timeline", &timeline, vec![subject, objective]),
    ] {
        assert!(shown.errors.is_empty(), "{view}: {:?}", shown.errors);
        for text in content {
            assert!(
                shown.text.contains(text),
                "{view} shows the content as text: {text}"
            );
        }
        assert_eq!(shown.markup, 0, "{view} built no element from data");
        assert_eq!(shown.ran, None, "{view} ran nothing from data");
    }

    page.close().await.expect("close the page");
    chrome.finish().await;
}

/// The page's rendering helper turns EVERY value a view can hand it into text,
/// including the kinds no view passes today. The Timeline's number was one such
/// kind, and a structured value is the next: the helper renders it as its JSON,
/// where `String()` would show `[object Object]`.
///
/// No database: the helper is the page's own, served by `ui_router` alone, so
/// this control also runs where only the browser is available.
#[tokio::test]
async fn the_rendering_helper_turns_every_value_into_text() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's rendering helper");
        return;
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral loopback port");
    let base = format!("http://{}", listener.local_addr().expect("local address"));
    let server = tokio::spawn(async move {
        axum::serve(listener, ui_router()).await.expect("serve");
    });

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    // `el` is a top-level function of the page's classic script, so the page's
    // global scope is where it is found.
    const CELLS: &str = r#"JSON.stringify(
        [7, 0, -1.5, true, false, { a: [1, "<b>"] }, [1, 2], "<i>x</i>"].map((value) => {
            const cell = el("td", null, value);
            return [cell.textContent, cell.childElementCount];
        }))"#;
    let cells: String = page
        .evaluate(CELLS)
        .await
        .expect("run the helper")
        .into_value()
        .expect("a JSON string");
    let cells: Vec<(String, u64)> = serde_json::from_str(&cells).expect("cells");
    let expected = [
        "7",
        "0",
        "-1.5",
        "true",
        "false",
        r#"{"a":[1,"<b>"]}"#,
        "[1,2]",
        "<i>x</i>",
    ];
    assert_eq!(
        cells,
        expected.map(|text| (text.to_owned(), 0)),
        "each value is one text child, never an element"
    );

    page.close().await.expect("close the page");
    chrome.finish().await;
    server.abort();
}

// ── A slow answer never lands in a later view (`SIGNOFF-REPAIR.11.1.2`) ──────
//
// Each control HOLDS one of the page's requests at the server, makes the
// operator move on, then releases it and waits for the page to receive it. The
// late answer must change nothing on screen. Before the repair every view drew
// into the one `#output` element after its fetch returned, whichever view was
// showing by then.

/// The Timeline is slow; the operator opens the Audit; the Timeline's answer
/// arrives. The Audit is all that shows.
#[tokio::test]
async fn a_slow_view_never_lands_under_a_later_one() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's view switching");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    let who = operator(&base, "console-switch", "switch probe", "hold the timeline").await;

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    sign_in_and_open_thread(&page, &who).await;

    let mut timeline = console.hold.arm("/events?");
    click(&page, "button[data-view=events]").await;
    timeline.parked().await;
    click(&page, "button[data-view=audit]").await;
    let audit = rendered(&page, "Audit (authorization records)").await;
    assert!(audit.errors.is_empty(), "{audit:?}");
    assert!(!audit.rows.is_empty(), "{audit:?}");

    let _ = timeline.release.send(());
    received(&page, "/events?").await;
    let after = rendered(&page, "Audit (authorization records)").await;
    assert_eq!(
        after.headings,
        ["Audit (authorization records)"],
        "{after:?}"
    );
    assert_eq!(after.tables, 1, "only the Audit's table shows: {after:?}");
    assert_eq!(
        after.rows, audit.rows,
        "the late Timeline changed the Audit"
    );

    page.close().await.expect("close the page");
    chrome.finish().await;
}

/// Tenant A's thread list is slow; the operator switches to tenant B's
/// identity; A's list arrives. Only B's threads show.
#[tokio::test]
async fn a_changed_identity_never_shows_the_old_identitys_threads() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's identity change");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    let first = operator(&base, "console-identity-a", "tenant A's thread", "held").await;
    let second = operator(&base, "console-identity-b", "tenant B's thread", "shown").await;

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;

    let held_list = format!("tenant_id={}", first.tenant);
    let mut threads = console.hold.arm(&held_list);
    refill(&page, "#principal", &first.principal).await;
    refill(&page, "#tenant", &first.tenant).await;
    click(&page, "#auth-form button[type=submit]").await;
    threads.parked().await;
    refill(&page, "#principal", &second.principal).await;
    refill(&page, "#tenant", &second.tenant).await;
    click(&page, "#auth-form button[type=submit]").await;
    let shown = showing(&page, "Threads", &second.thread).await;
    assert!(shown.errors.is_empty(), "{shown:?}");

    let _ = threads.release.send(());
    received(&page, &held_list).await;
    let after = rendered(&page, "Threads").await;
    assert_eq!(after.headings, ["Threads"], "{after:?}");
    assert!(
        !after.text.contains(&first.thread),
        "tenant A's thread shows under tenant B's identity: {after:?}"
    );
    assert_eq!(after.rows, shown.rows, "the late list changed the view");

    page.close().await.expect("close the page");
    chrome.finish().await;
}

/// Two presence checks: the first is slow, the second answers, then the first
/// arrives. The panel keeps the answer to the check the operator made last.
/// The server's refusal names the node it was asked about, which is what tells
/// the two answers apart.
#[tokio::test]
async fn two_presence_checks_show_only_the_later_answer() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's presence panel");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    let who = operator(&base, "console-presence", "presence probe", "two checks").await;

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    sign_in_and_open_thread(&page, &who).await;
    click(&page, "button[data-view=presence]").await;
    rendered(&page, "Node presence").await;

    let mut earlier = console.hold.arm("node_id=nod-earlier-check");
    refill(&page, "#output input", "nod-earlier-check").await;
    click(&page, "#output button").await;
    earlier.parked().await;
    refill(&page, "#output input", "nod-later-check").await;
    click(&page, "#output button").await;
    let shown = showing(&page, "Node presence", "nod-later-check").await;

    let _ = earlier.release.send(());
    received(&page, "node_id=nod-earlier-check").await;
    let after = rendered(&page, "Node presence").await;
    assert!(
        after.text.contains("nod-later-check") && !after.text.contains("nod-earlier-check"),
        "the panel shows the earlier check's answer: {after:?}"
    );
    assert_eq!(after.text, shown.text, "the late answer changed the panel");

    page.close().await.expect("close the page");
    chrome.finish().await;
}

/// The inbox panel under the same race, with answers told apart by content: the
/// held check is for a node with one queued command, the later one for a node
/// with none. Then the queued command, checked on its own, renders.
#[tokio::test]
async fn two_inbox_checks_show_only_the_later_answer() {
    let Some(binary) = browser_binary() else {
        skip_without_browser("the console's inbox panel");
        return;
    };
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let console = Console::start(&pool).await;
    let base = console.base();
    let who = operator(&base, "console-inbox", "inbox probe", "two checks").await;
    // One queued command for a node of the operator's own tenant, seeded as the
    // inbox suites seed theirs.
    let queued = format!("nod-console-{}", uuid::Uuid::now_v7());
    sqlx::query(
        "INSERT INTO node_inbox (node_id, cursor, command_id, tenant_id, thread_id, payload) \
         VALUES ($1, 1, 'cmd-console-queued', $2, $3, '{}'::jsonb)",
    )
    .bind(&queued)
    .bind(&who.tenant)
    .bind(&who.thread)
    .execute(&pool)
    .await
    .expect("seed a queued command");

    let root = pg_test_support::repository_root().expect("the repository root");
    let chrome = Chrome::launch(&binary, &root).await;
    let page = chrome.open(&base).await;
    sign_in_and_open_thread(&page, &who).await;
    click(&page, "button[data-view=inbox]").await;
    let heading = "Node inbox (tenant_admin)";
    rendered(&page, heading).await;

    let held_check = format!("node_id={queued}");
    let mut earlier = console.hold.arm(&held_check);
    refill(&page, "#output input", &queued).await;
    click(&page, "#output button").await;
    earlier.parked().await;
    refill(&page, "#output input", "nod-console-empty").await;
    click(&page, "#output button").await;
    // The column header appears once the later answer's table is drawn.
    let shown = showing(&page, heading, "quarantine").await;
    assert!(
        shown.rows.is_empty(),
        "the later node has no commands: {shown:?}"
    );

    let _ = earlier.release.send(());
    received(&page, &held_check).await;
    let after = rendered(&page, heading).await;
    assert_eq!(after.tables, 1, "{after:?}");
    assert!(
        after.rows.is_empty(),
        "the panel shows the earlier check's answer: {after:?}"
    );

    // Checked on its own, the queued command renders with its delivery state.
    let inbox: Value = reqwest::Client::new()
        .get(format!(
            "{base}/v1/nodes/inbox?node_id={queued}&tenant_id={}",
            who.tenant
        ))
        .header(PRINCIPAL_HEADER, &who.principal)
        .send()
        .await
        .expect("read the inbox")
        .json()
        .await
        .expect("inbox json");
    let state = inbox["rows"][0]["delivery_state"]
        .as_str()
        .unwrap_or_else(|| panic!("the seeded row: {inbox}"))
        .to_owned();
    refill(&page, "#output input", &queued).await;
    click(&page, "#output button").await;
    let queued_view = showing(&page, heading, "cmd-console-queued").await;
    assert_eq!(
        queued_view.rows,
        [["cmd-console-queued", state.as_str(), "—", "—"]],
        "the panel shows the server's row"
    );

    page.close().await.expect("close the page");
    chrome.finish().await;
}

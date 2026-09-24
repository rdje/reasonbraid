//! The `.1.6.2` static inspection console — a vanilla HTML/JS/CSS page served by
//! `rb-server` at `/` (`docs/decisions/2026-09-06_ui-embedding.md`).
//!
//! Embedded at COMPILE time (`include_str!`): one binary, no runtime paths, no
//! frontend build pipeline (§12; the `ui-direction` decision). READ-ONLY: the
//! page renders the existing GET surfaces through its own same-origin fetches
//! (the dev-profile `x-reasonbraid-principal` header + the `tenant_id` query,
//! exactly as the CLI sends them) — this module adds NO API route, NO grant,
//! NO write path. Page-side rendering is text-safe by construction (the app
//! never uses `innerHTML` with data), enforced by the offline tests below.

use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;

const INDEX_HTML: &str = include_str!("../web/index.html");
const APP_JS: &str = include_str!("../web/app.js");
const STYLE_CSS: &str = include_str!("../web/style.css");

/// The console's static routes. State-free: the page talks to the API like any
/// other client — the shell carries no pool and no authority of its own.
pub fn ui_router() -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/app.js", get(serve_app_js))
        .route("/style.css", get(serve_style_css))
}

async fn serve_index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

fn typed(media_type: &'static str, bytes: &'static str) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, media_type)], bytes).into_response()
}

async fn serve_app_js() -> impl IntoResponse {
    typed("text/javascript; charset=utf-8", APP_JS)
}

async fn serve_style_css() -> impl IntoResponse {
    typed("text/css; charset=utf-8", STYLE_CSS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page's data contract is mechanical: it references ONLY the documented
    /// read surfaces, renders through `textContent` (never `innerHTML`), and
    /// performs no write (no POST) — a read-only shell by construction.
    #[test]
    fn the_page_consumes_only_the_documented_read_surfaces() {
        for surface in [
            "/v1/threads?",
            "/v1/threads/",
            "/events?",
            "/audit?",
            "/budget?",
            "/v1/nodes/presence?node_id=",
            "/v1/nodes/inbox?node_id=",
        ] {
            assert!(
                APP_JS.contains(surface),
                "app.js must consume the documented surface `{surface}`"
            );
        }
        assert!(
            !APP_JS.contains("innerHTML"),
            "the page renders through textContent — never innerHTML with data"
        );
        assert!(
            !APP_JS.to_ascii_uppercase().contains("POST"),
            "the page is read-only — no write verb"
        );
        assert!(
            INDEX_HTML.contains("/app.js") && INDEX_HTML.contains("/style.css"),
            "the shell loads its assets from the same origin"
        );
    }

    /// The inbox panel speaks the SERVER's contract, checked against the
    /// server's own types rather than a list of strings (`SIGNOFF-REPAIR.4.4.2.2`).
    ///
    /// The panel sent `?node=` for two years of commits while the route required
    /// `node_id`, and rendered `r.state`, a field no inbox row has had; the
    /// surface list above pinned the broken prefix, so the check agreed with the
    /// bug. Here the query the panel builds is parsed by the route's own
    /// extractor, and every row field it reads must be a field the server
    /// serializes.
    #[test]
    fn the_inbox_panel_speaks_the_servers_contract() {
        let start = APP_JS
            .find("\"/v1/nodes/inbox?")
            .expect("the panel requests the inbox");
        let panel_end = start
            + APP_JS[start..]
                .find("\n}\n")
                .expect("the panel's function ends");
        let panel = &APP_JS[start..panel_end];
        let request = &panel[..panel.find(");").expect("the request expression ends")];

        // The query's parameter names, as the panel writes them: every `name=`
        // after `?` or `&` in the request, plus `tenant_id` when it appends
        // `tenantQuery()`, whose own literal is checked to be that name.
        assert!(APP_JS.contains("return \"tenant_id=\" + encodeURIComponent(state.tenant);"));
        let mut query = Vec::new();
        for (i, _) in request.match_indices('=') {
            let before = &request[..i];
            let name_start = before.rfind(['?', '&', '"']).map_or(0, |p| p + 1);
            let name = &before[name_start..];
            if !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                query.push(format!("{name}=nod_00000000-0000-7000-8000-000000000001"));
            }
        }
        if request.contains("tenantQuery()") {
            query.push("tenant_id=ten_00000000-0000-7000-8000-000000000000".to_string());
        }
        let uri: axum::http::Uri = format!("/v1/nodes/inbox?{}", query.join("&"))
            .parse()
            .expect("a uri");
        if let Err(e) =
            axum::extract::Query::<crate::api::InboxInspectionParams>::try_from_uri(&uri)
        {
            panic!("the route refuses the panel's query `{uri}`: {e}");
        }

        // Every row field the panel reads is one the server writes.
        let row = serde_json::to_value(crate::api::InboxRow {
            cursor: 1,
            command_id: String::new(),
            thread_id: String::new(),
            payload: serde_json::Value::Null,
            acknowledged_at: None,
            quarantined_at: None,
            quarantine_reason: None,
            delivery_state: String::new(),
            result_refusal: None,
        })
        .expect("a row serializes");
        let fields = row.as_object().expect("a row is an object");
        let mut read = Vec::new();
        for (i, _) in panel.match_indices("r.") {
            let preceded_by_ident = panel[..i]
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
            if preceded_by_ident {
                continue;
            }
            let name: String = panel[i + 2..]
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || *c == '_')
                .collect();
            if !name.is_empty() {
                read.push(name);
            }
        }
        assert!(!read.is_empty(), "the panel reads its rows");
        for name in &read {
            assert!(
                fields.contains_key(name.as_str()),
                "the panel reads `r.{name}`, which no inbox row has (rows carry {:?})",
                fields.keys().collect::<Vec<_>>()
            );
        }
    }

    /// The shell serves over the real listener with typed content types —
    /// the offline proof that the single binary carries the whole console.
    #[tokio::test]
    async fn the_console_serves_from_the_router() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, ui_router()).await.expect("serve");
        });
        let client = reqwest::Client::new();

        let index = client
            .get(format!("http://{addr}/"))
            .send()
            .await
            .expect("GET /");
        assert_eq!(index.status().as_u16(), 200, "the shell serves at /");
        let content_type = index
            .headers()
            .get("content-type")
            .expect("typed content type")
            .to_str()
            .expect("ascii header");
        assert!(
            content_type.starts_with("text/html"),
            "index content-type: {content_type}"
        );
        let html = index.text().await.expect("html body");
        assert!(
            html.contains("inspection console"),
            "the shell marker is served"
        );

        let app_js = client
            .get(format!("http://{addr}/app.js"))
            .send()
            .await
            .expect("GET /app.js");
        assert_eq!(app_js.status().as_u16(), 200);
        let content_type = app_js
            .headers()
            .get("content-type")
            .expect("typed content type")
            .to_str()
            .expect("ascii header");
        assert!(
            content_type.starts_with("text/javascript"),
            "app.js content-type: {content_type}"
        );
        assert!(
            app_js
                .text()
                .await
                .expect("js body")
                .contains("x-reasonbraid-principal"),
            "the page presents the dev-profile principal header, same as the CLI"
        );

        let style = client
            .get(format!("http://{addr}/style.css"))
            .send()
            .await
            .expect("GET /style.css");
        assert_eq!(style.status().as_u16(), 200);
        let content_type = style
            .headers()
            .get("content-type")
            .expect("typed content type")
            .to_str()
            .expect("ascii header");
        assert!(
            content_type.starts_with("text/css"),
            "style.css content-type: {content_type}"
        );

        handle.abort();
    }
}

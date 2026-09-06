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
            "/v1/nodes/inbox?node=",
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

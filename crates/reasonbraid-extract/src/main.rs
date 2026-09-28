//! The R2 extraction worker (PHASE-4.4.2): the stdio JSON protocol, the
//! per-format parsers, and the `.4.1` contract's refusal vocabulary — every
//! refusal names its kind; the extraction is ALWAYS a Derivation (the
//! parent digest + the derived chunks, each with its own ADR-011 digest).
//!
//! The process is the quarantine: ONE request in, ONE response out, exit.

use serde::{Deserialize, Serialize};

/// The worker's wire request (the `.4.1` contract's shape): a temp path to
/// the acquired bytes + the declared media type + the ceilings.
#[derive(Debug, Deserialize)]
pub struct ExtractRequest {
    pub input_path: String,
    pub media_type: String,
    pub limits: ExtractLimits,
}

#[derive(Debug, Deserialize)]
pub struct ExtractLimits {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_chunks: usize,
    pub max_entry_bytes: u64,
    pub max_decompression_ratio: f64,
}

/// The Derivation chunk: the derived text + its ADR-011 digest.
#[derive(Debug, Serialize)]
pub struct DerivedChunk {
    pub digest: String,
    pub text: String,
}

/// The worker's wire response.
#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub parent_digest: String,
    pub chunks: Vec<DerivedChunk>,
    pub excluded: Vec<String>,
    pub extractor_version: String,
}

/// The named refusal (the `.4.1` vocabulary). `kind` is the stable wire
/// name; the message names the offender.
#[derive(Debug, Serialize)]
pub struct ExtractError {
    pub kind: String,
    pub message: String,
}

pub const EXTRACTOR_VERSION: &str = "0.1.0";

/// The internal error: the refusal kind + the detail.
#[derive(Debug)]
pub struct Refusal {
    pub kind: &'static str,
    pub message: String,
}

impl Refusal {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// The ADR-011 digest over the bytes (shared with the receipts).
pub fn digest_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn main() {
    // ONE request on stdin (the server writes a single line), ONE response
    // on stdout, then exit — nothing persists.
    let mut input = String::new();
    let request: ExtractRequest =
        match std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)
            .map_err(|error| error.to_string())
            .and_then(|_| serde_json::from_str(&input).map_err(|error| error.to_string()))
        {
            Ok(request) => request,
            Err(message) => {
                respond_error(&ExtractError {
                    kind: "request_unreadable".to_owned(),
                    message,
                });
                return;
            }
        };
    match extract(&request) {
        Ok(response) => respond(&response),
        Err(refusal) => respond_error(&ExtractError {
            kind: refusal.kind.to_owned(),
            message: refusal.message,
        }),
    }
}

fn respond(payload: &impl Serialize) {
    if let Ok(json) = serde_json::to_string(payload) {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "{json}");
        let _ = stdout.flush();
    }
}

/// The refusal envelope: ALWAYS `{error: {kind, message}}` — the wire
/// contract the server's spawner matches on.
fn respond_error(error: &ExtractError) {
    respond(&serde_json::json!({ "error": error }));
}

/// The extraction: the media type dispatches to the parser; the ceilings
/// are enforced before and during; every refusal is named.
pub fn extract(request: &ExtractRequest) -> Result<ExtractResponse, Refusal> {
    let bytes = std::fs::read(&request.input_path)
        .map_err(|e| Refusal::new("input_unreadable", format!("the input failed: {e}")))?;
    if bytes.len() as u64 > request.limits.max_input_bytes {
        return Err(Refusal::new(
            "input_too_large",
            format!(
                "the input ({}) exceeds the {}-byte ceiling",
                bytes.len(),
                request.limits.max_input_bytes
            ),
        ));
    }
    let parent_digest = digest_sha256_hex(&bytes);
    let (mut chunks, excluded) = match request.media_type.as_str() {
        "application/pdf" => extract_pdf(&bytes, request)?,
        "application/zip" => extract_zip(&bytes, request)?,
        "application/x-tar" => extract_tar(&bytes, request)?,
        "application/atom+xml" | "application/rss+xml" => extract_feed(&bytes, request)?,
        other => {
            return Err(Refusal::new(
                "media_type_unsupported",
                format!("the media type `{other}` is outside the R2 format set"),
            ));
        }
    };
    if chunks.len() > request.limits.max_chunks {
        return Err(Refusal::new(
            "chunk_count_exceeded",
            format!(
                "{} chunks exceeds the {}-chunk ceiling",
                chunks.len(),
                request.limits.max_chunks
            ),
        ));
    }
    let total: usize = chunks.iter().map(|c| c.text.len()).sum();
    if total as u64 > request.limits.max_output_bytes {
        return Err(Refusal::new(
            "output_too_large",
            format!(
                "the derived text ({total} bytes) exceeds the {}-byte ceiling",
                request.limits.max_output_bytes
            ),
        ));
    }
    for chunk in &mut chunks {
        chunk.digest = digest_sha256_hex(chunk.text.as_bytes());
    }
    Ok(ExtractResponse {
        parent_digest,
        chunks,
        excluded,
        extractor_version: EXTRACTOR_VERSION.to_owned(),
    })
}

fn push_chunk(
    chunks: &mut Vec<DerivedChunk>,
    text: String,
    request: &ExtractRequest,
) -> Result<(), Refusal> {
    if text.is_empty() {
        return Ok(());
    }
    if text.len() as u64 > request.limits.max_output_bytes {
        return Err(Refusal::new(
            "output_too_large",
            format!(
                "a derived chunk ({}) exceeds the {}-byte ceiling",
                text.len(),
                request.limits.max_output_bytes
            ),
        ));
    }
    chunks.push(DerivedChunk {
        digest: String::new(),
        text,
    });
    Ok(())
}

// ── PDF ──────────────────────────────────────────────────────────────────

fn extract_pdf(
    bytes: &[u8],
    request: &ExtractRequest,
) -> Result<(Vec<DerivedChunk>, Vec<String>), Refusal> {
    let document = lopdf::Document::load_mem(bytes)
        .map_err(|e| Refusal::new("pdf_unreadable", format!("the PDF failed to load: {e}")))?;
    // The refusal list, checked BEFORE any text extraction: an encrypted
    // document and a JS-bearing catalog are named, never parsed.
    if document.is_encrypted() {
        return Err(Refusal::new(
            "pdf_encrypted",
            "the PDF is encrypted (the R2 policy refuses encrypted documents)",
        ));
    }
    if js_bearing(&document) {
        return Err(Refusal::new(
            "pdf_javascript",
            "the PDF carries JavaScript (the R2 policy refuses it)",
        ));
    }
    let mut chunks = Vec::new();
    let pages = document.get_pages();
    for (index, _) in pages.values().enumerate() {
        let text = document
            .extract_text(&[(index + 1) as u32])
            .map_err(|e| Refusal::new("pdf_unreadable", format!("the page text failed: {e}")))?;
        push_chunk(&mut chunks, text, request)?;
    }
    Ok((chunks, Vec::new()))
}

/// The JS-bearing check: the catalog's (and any page's `AA` action
/// dictionary's) keys naming `JS`/`JavaScript` anywhere.
fn js_bearing(document: &lopdf::Document) -> bool {
    let mut stack: Vec<lopdf::Object> = Vec::new();
    if let Ok(catalog) = document
        .trailer
        .get(b"Root")
        .and_then(|root| root.as_reference())
        .and_then(|id| document.get_object(id))
    {
        stack.push(catalog.clone());
    }
    while let Some(object) = stack.pop() {
        if let lopdf::Object::Dictionary(dict) = object {
            for (key, value) in dict {
                let key_upper = String::from_utf8_lossy(&key).to_ascii_uppercase();
                if key_upper.contains("JS") || key_upper.contains("JAVASCRIPT") {
                    return true;
                }
                match value {
                    lopdf::Object::Dictionary(_) => stack.push(value.clone()),
                    lopdf::Object::Array(items) => {
                        for item in items {
                            if matches!(item, lopdf::Object::Dictionary(_)) {
                                stack.push(item.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    false
}

// ── the archives ─────────────────────────────────────────────────────────

fn entry_name_safe(name: &str) -> Result<(), Refusal> {
    if name.starts_with('/') || name.contains('\\') || name.split('/').any(|part| part == "..") {
        return Err(Refusal::new(
            "path_traversal",
            format!("the entry name `{name}` traverses (refused)"),
        ));
    }
    Ok(())
}

fn is_archive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".zip")
        || lower.ends_with(".tar")
        || lower.ends_with(".tgz")
        || lower.ends_with(".gz")
}

/// An archive recognised by its CONTENT (`SIGNOFF-REPAIR.11.54`): the entry name
/// alone let a nested archive under any other name through as an excluded
/// binary, where `deployment.md` says R2 refuses one by name. The signatures:
/// zip, gzip, tar (`ustar` at 257), bzip2 (`BZh[1-9]` then the block magic),
/// xz and 7z.
fn is_archive_bytes(data: &[u8]) -> bool {
    data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
        || data.starts_with(&[0x1f, 0x8b])
        || data.get(257..262) == Some(b"ustar".as_slice())
        || (data.starts_with(b"BZh")
            && data.get(3).is_some_and(|b| (b'1'..=b'9').contains(b))
            && data.get(4..10) == Some(b"1AY&SY".as_slice()))
        || data.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00])
        || data.starts_with(&[b'7', b'z', 0xbc, 0xaf, 0x27, 0x1c])
}

fn nested_by_content(name: &str) -> Refusal {
    Refusal::new(
        "nested_archive",
        format!("the entry `{name}` is an archive by its content (one level, then refused)"),
    )
}

fn text_eligible(name: &str, data: &[u8], request: &ExtractRequest) -> bool {
    if is_archive_name(name) {
        return false; // the nested-archive refusal happens at the caller
    }
    data.len() as u64 <= request.limits.max_entry_bytes
        && std::str::from_utf8(data).is_ok()
        && data
            .iter()
            .all(|b| !b.is_ascii_control() || *b == b'\t' || *b == b'\n' || *b == b'\r')
}

fn decompression_check(
    compressed: u64,
    decompressed: u64,
    request: &ExtractRequest,
    name: &str,
) -> Result<(), Refusal> {
    if compressed > 0 {
        let ratio = decompressed as f64 / compressed as f64;
        if ratio > request.limits.max_decompression_ratio {
            return Err(Refusal::new(
                "decompression_ratio_exceeded",
                format!(
                    "the entry `{name}` decoded to {decompressed} bytes from a {compressed}-byte envelope"
                ),
            ));
        }
    }
    Ok(())
}

fn extract_zip(
    bytes: &[u8],
    request: &ExtractRequest,
) -> Result<(Vec<DerivedChunk>, Vec<String>), Refusal> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| Refusal::new("archive_unreadable", format!("the zip failed: {e}")))?;
    let mut chunks = Vec::new();
    let mut excluded = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|e| {
            Refusal::new("archive_unreadable", format!("the zip entry failed: {e}"))
        })?;
        let name = entry.name().to_owned();
        entry_name_safe(&name)?;
        if entry.is_dir() {
            continue;
        }
        if is_archive_name(&name) {
            return Err(Refusal::new(
                "nested_archive",
                format!("the entry `{name}` is an archive (one level, then refused)"),
            ));
        }
        let declared = entry.size();
        let compressed = entry.compressed_size();
        if declared > request.limits.max_entry_bytes {
            return Err(Refusal::new(
                "entry_too_large",
                format!(
                    "the entry `{name}` declares {declared} bytes (the ceiling is {})",
                    request.limits.max_entry_bytes
                ),
            ));
        }
        let mut data =
            Vec::with_capacity(declared.min(request.limits.max_entry_bytes) as usize + 1);
        use std::io::Read;
        entry
            .take(request.limits.max_entry_bytes + 1)
            .read_to_end(&mut data)
            .map_err(|e| {
                Refusal::new(
                    "archive_unreadable",
                    format!("the zip entry read failed: {e}"),
                )
            })?;
        // An archive is named as one before the ratio brake judges it: its
        // bytes are already read (bounded by `take`), and a tar compresses so
        // well that the brake would otherwise answer first.
        if is_archive_bytes(&data) {
            return Err(nested_by_content(&name));
        }
        decompression_check(compressed, data.len() as u64, request, &name)?;
        if data.len() as u64 > request.limits.max_entry_bytes {
            return Err(Refusal::new(
                "entry_too_large",
                format!(
                    "the entry `{name}` decoded to {} bytes (the ceiling is {})",
                    data.len(),
                    request.limits.max_entry_bytes
                ),
            ));
        }
        if text_eligible(&name, &data, request) {
            push_chunk(
                &mut chunks,
                String::from_utf8_lossy(&data).into_owned(),
                request,
            )?;
        } else {
            excluded.push(name);
        }
    }
    Ok((chunks, excluded))
}

fn extract_tar(
    bytes: &[u8],
    request: &ExtractRequest,
) -> Result<(Vec<DerivedChunk>, Vec<String>), Refusal> {
    let mut archive = tar::Archive::new(bytes);
    let mut chunks = Vec::new();
    let mut excluded = Vec::new();
    for entry in archive
        .entries()
        .map_err(|e| Refusal::new("archive_unreadable", format!("the tar failed: {e}")))?
    {
        let entry = entry.map_err(|e| {
            Refusal::new("archive_unreadable", format!("the tar entry failed: {e}"))
        })?;
        let name = entry
            .path()
            .map_err(|e| Refusal::new("path_traversal", format!("the tar entry name failed: {e}")))?
            .to_string_lossy()
            .into_owned();
        entry_name_safe(&name)?;
        if is_archive_name(&name) {
            return Err(Refusal::new(
                "nested_archive",
                format!("the entry `{name}` is an archive (one level, then refused)"),
            ));
        }
        let mut data = Vec::new();
        use std::io::Read;
        entry
            .take(request.limits.max_entry_bytes + 1)
            .read_to_end(&mut data)
            .map_err(|e| {
                Refusal::new(
                    "archive_unreadable",
                    format!("the tar entry read failed: {e}"),
                )
            })?;
        if data.len() as u64 > request.limits.max_entry_bytes {
            return Err(Refusal::new(
                "entry_too_large",
                format!(
                    "the entry `{name}` is {} bytes (the ceiling is {})",
                    data.len(),
                    request.limits.max_entry_bytes
                ),
            ));
        }
        if is_archive_bytes(&data) {
            return Err(nested_by_content(&name));
        }
        if text_eligible(&name, &data, request) {
            push_chunk(
                &mut chunks,
                String::from_utf8_lossy(&data).into_owned(),
                request,
            )?;
        } else {
            excluded.push(name);
        }
    }
    Ok((chunks, excluded))
}

// ── the feeds ────────────────────────────────────────────────────────────

fn extract_feed(
    bytes: &[u8],
    request: &ExtractRequest,
) -> Result<(Vec<DerivedChunk>, Vec<String>), Refusal> {
    // 🔴 `SIGNOFF-REPAIR.11.54`: R2 advertises `application/rss+xml`, and every
    // feed went to the Atom parser, which refuses an `<rss>` root, so no RSS
    // feed ever extracted. The document's own root element decides, whichever
    // of the two types was declared.
    if root_element(bytes)?.as_deref() == Some(b"rss".as_slice()) {
        return extract_rss(bytes, request);
    }
    let feed = atom_syndication::Feed::read_from(bytes)
        .map_err(|e| Refusal::new("feed_unreadable", format!("the feed failed to parse: {e}")))?;
    let mut chunks = Vec::new();
    let mut title = feed.title().value.clone();
    if let Some(subtitle) = feed.subtitle() {
        title.push('\n');
        title.push_str(&subtitle.value);
    }
    push_chunk(&mut chunks, title, request)?;
    for entry in feed.entries() {
        let mut text = entry.title().value.clone();
        if let Some(summary) = entry.summary() {
            text.push('\n');
            text.push_str(&summary.value);
        }
        if let Some(content) = entry.content() {
            if let Some(value) = &content.value {
                text.push('\n');
                text.push_str(value);
            }
        }
        push_chunk(&mut chunks, text, request)?;
    }
    Ok((chunks, Vec::new()))
}

fn feed_unreadable(error: impl std::fmt::Display) -> Refusal {
    Refusal::new(
        "feed_unreadable",
        format!("the feed failed to parse: {error}"),
    )
}

/// The local name of the document's first element.
fn root_element(bytes: &[u8]) -> Result<Option<Vec<u8>>, Refusal> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(feed_unreadable)? {
            Event::Start(e) | Event::Empty(e) => return Ok(Some(e.local_name().as_ref().to_vec())),
            Event::Eof => return Ok(None),
            _ => buf.clear(),
        }
    }
}

/// RSS 2.0: the channel's title and description, then each item's title and
/// description, as the Atom path gives a feed's title and each entry's. Text,
/// CDATA, character references and the five predefined entities are read; any
/// other entity refuses, since an RSS document declares none of its own.
fn extract_rss(
    bytes: &[u8],
    request: &ExtractRequest,
) -> Result<(Vec<DerivedChunk>, Vec<String>), Refusal> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut buf = Vec::new();
    let mut path: Vec<Vec<u8>> = Vec::new();
    let (mut channel_title, mut channel_description) = (String::new(), String::new());
    let mut item: Option<(String, String)> = None;
    let mut items: Vec<(String, String)> = Vec::new();
    loop {
        let text = match reader.read_event_into(&mut buf).map_err(feed_unreadable)? {
            Event::Start(e) => {
                let name = e.local_name().as_ref().to_vec();
                if name == b"item" {
                    item = Some((String::new(), String::new()));
                }
                path.push(name);
                None
            }
            Event::End(_) => {
                if path.last().map(Vec::as_slice) == Some(b"item".as_slice()) {
                    if let Some(done) = item.take() {
                        items.push(done);
                    }
                }
                path.pop();
                None
            }
            Event::Text(t) => Some(t.decode().map_err(feed_unreadable)?.into_owned()),
            Event::CData(c) => Some(c.decode().map_err(feed_unreadable)?.into_owned()),
            Event::GeneralRef(r) => {
                if r.is_char_ref() {
                    r.resolve_char_ref()
                        .map_err(feed_unreadable)?
                        .map(String::from)
                } else {
                    let name = r.decode().map_err(feed_unreadable)?;
                    let resolved =
                        quick_xml::escape::resolve_predefined_entity(&name).ok_or_else(|| {
                            feed_unreadable(format!("the entity `&{name};` is undeclared"))
                        })?;
                    Some(resolved.to_owned())
                }
            }
            Event::Eof => break,
            _ => None,
        };
        if let Some(text) = text {
            let leaf = path.last().map(Vec::as_slice);
            let parent = path.len().checked_sub(2).map(|i| path[i].as_slice());
            let target = match (&mut item, parent, leaf) {
                (Some((title, _)), Some(b"item"), Some(b"title")) => Some(title),
                (Some((_, description)), Some(b"item"), Some(b"description")) => Some(description),
                (None, Some(b"channel"), Some(b"title")) => Some(&mut channel_title),
                (None, Some(b"channel"), Some(b"description")) => Some(&mut channel_description),
                _ => None,
            };
            if let Some(target) = target {
                target.push_str(&text);
            }
        }
        buf.clear();
    }
    let joined = |title: &str, description: &str| {
        let (title, description) = (title.trim(), description.trim());
        if description.is_empty() {
            title.to_owned()
        } else {
            format!("{title}\n{description}")
        }
    };
    let mut chunks = Vec::new();
    push_chunk(
        &mut chunks,
        joined(&channel_title, &channel_description),
        request,
    )?;
    for (title, description) in &items {
        push_chunk(&mut chunks, joined(title, description), request)?;
    }
    Ok((chunks, Vec::new()))
}

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod input_fixture;

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> ExtractLimits {
        ExtractLimits {
            max_input_bytes: 16 * 1024 * 1024,
            max_output_bytes: 4 * 1024 * 1024,
            max_chunks: 256,
            max_entry_bytes: 4 * 1024 * 1024,
            max_decompression_ratio: 10.0,
        }
    }

    fn request_for(path: &std::path::Path, media_type: &str) -> ExtractRequest {
        ExtractRequest {
            input_path: path.display().to_string(),
            media_type: media_type.to_owned(),
            limits: limits(),
        }
    }

    fn write_input(bytes: &[u8]) -> input_fixture::Input {
        input_fixture::Input::new(bytes)
    }

    #[test]
    fn the_inputs_remain_independent_until_their_owner_finishes() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(32));
        let writers: Vec<_> = (0..32)
            .map(|id| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let bytes = format!("independent input {id}").into_bytes();
                    barrier.wait();
                    (write_input(&bytes), bytes)
                })
            })
            .collect();
        // Consume every thread even if one writer failed before returning an input.
        let outcomes: Vec<_> = writers.into_iter().map(|w| w.join()).collect();
        let inputs: Vec<_> = outcomes.into_iter().map(Result::unwrap).collect();
        let paths: std::collections::HashSet<_> = inputs
            .iter()
            .map(|(input, _)| input.path().to_path_buf())
            .collect();
        assert_eq!(paths.len(), inputs.len());
        for (input, bytes) in &inputs {
            assert_eq!(std::fs::read(input.path()).unwrap(), *bytes);
        }
        for (input, bytes) in inputs {
            assert_eq!(std::fs::read(input.path()).unwrap(), bytes);
            let path = input.path().to_path_buf();
            drop(input);
            assert!(!path.try_exists().unwrap());
        }
    }

    #[cfg(unix)]
    #[test]
    fn the_input_allocator_preserves_existing_files_and_links() {
        use std::os::unix::fs::PermissionsExt;
        let input = write_input(b"existing owner");
        assert_eq!(
            std::fs::metadata(input.path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let name = input.path().file_name().unwrap().to_str().unwrap();
        let refused = input_fixture::Input::create_named(name, b"would truncate");
        assert_eq!(
            refused.err().unwrap().kind(),
            std::io::ErrorKind::AlreadyExists
        );

        let link_name = format!("link-{name}");
        let link = input.path().with_file_name(&link_name);
        std::os::unix::fs::symlink(input.path(), &link).unwrap();
        let refused = input_fixture::Input::create_named(&link_name, b"would follow");
        assert_eq!(
            refused.err().unwrap().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(input.path()).unwrap(), b"existing owner");
        assert_eq!(std::fs::read_link(&link).unwrap(), input.path());
        assert!(input_fixture::Input::create_named("../escape", b"no").is_err());
        std::fs::remove_file(link).unwrap();
    }

    #[test]
    fn failed_and_replaced_inputs_are_retained() {
        let input = write_input(b"failed assertion evidence");
        let path = input.path().to_path_buf();
        let result = std::panic::catch_unwind(move || {
            let _owner = input;
            panic!("deliberate assertion failure");
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"failed assertion evidence");
        std::fs::remove_file(path).unwrap();

        let input = write_input(b"original identity");
        let path = input.path().to_path_buf();
        let retained = path.with_extension("retained");
        std::fs::hard_link(&path, &retained).unwrap();
        std::fs::remove_file(&path).unwrap();
        let mut successor = std::fs::File::create_new(&path).unwrap();
        use std::io::Write;
        successor.write_all(b"successor identity").unwrap();
        let result = std::panic::catch_unwind(move || drop(input));
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"successor identity");
        assert_eq!(std::fs::read(&retained).unwrap(), b"original identity");
        drop(successor);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(retained).unwrap();
    }

    /// A minimal PDF (one page, one text stream), built with lopdf's own
    /// writer so the cross-reference table is valid.
    fn minimal_pdf() -> Vec<u8> {
        let mut document = lopdf::Document::with_version("1.4");
        let pages_id = document.new_object_id();
        let page_id = document.new_object_id();
        let font_id = document.new_object_id();
        let content_id = document.new_object_id();
        let catalog_id = document.new_object_id();
        document.objects.insert(
            font_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Font".into()),
                ("Subtype", "Type1".into()),
                ("BaseFont", "Helvetica".into()),
            ])),
        );
        document.objects.insert(
            content_id,
            lopdf::Object::Stream(lopdf::Stream::new(
                lopdf::Dictionary::from_iter([("Length", (44i64).into())]),
                b"BT /F1 12 Tf 72 720 Td (Hello extraction) Tj ET".to_vec(),
            )),
        );
        document.objects.insert(
            page_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Page".into()),
                ("Parent", lopdf::Object::Reference(pages_id)),
                (
                    "MediaBox",
                    lopdf::Object::Array(vec![0.into(), 0.into(), 612.into(), 792.into()]),
                ),
                ("Contents", lopdf::Object::Reference(content_id)),
                (
                    "Resources",
                    lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([(
                        "Font",
                        lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([(
                            "F1",
                            lopdf::Object::Reference(font_id),
                        )])),
                    )])),
                ),
            ])),
        );
        document.objects.insert(
            pages_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Pages".into()),
                (
                    "Kids",
                    lopdf::Object::Array(vec![lopdf::Object::Reference(page_id)]),
                ),
                ("Count", 1i64.into()),
            ])),
        );
        document.objects.insert(
            catalog_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Catalog".into()),
                ("Pages", lopdf::Object::Reference(pages_id)),
            ])),
        );
        document
            .trailer
            .set("Root", lopdf::Object::Reference(catalog_id));
        let mut out = Vec::new();
        document.save_to(&mut out).expect("the fixture PDF saves");
        out
    }

    #[test]
    fn the_pdf_extracts_its_text() {
        let fixture = minimal_pdf();
        let path = write_input(&fixture);
        let response =
            extract(&request_for(path.path(), "application/pdf")).expect("the PDF extracts");
        assert_eq!(response.chunks.len(), 1);
        assert!(
            response.chunks[0].text.contains("Hello extraction"),
            "{:?}",
            response.chunks[0].text
        );
        assert!(response.chunks[0].digest.starts_with("sha256:"));
        assert_eq!(response.parent_digest, digest_sha256_hex(&fixture));
    }

    #[test]
    fn the_pdf_refusals_name_themselves() {
        // The JS-bearing catalog: the catalog dictionary carries /JavaScript.
        let mut document = lopdf::Document::with_version("1.4");
        let pages_id = document.new_object_id();
        let catalog_id = document.new_object_id();
        document.objects.insert(
            pages_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Pages".into()),
                ("Count", 0i64.into()),
            ])),
        );
        document.objects.insert(
            catalog_id,
            lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([
                ("Type", "Catalog".into()),
                ("Pages", lopdf::Object::Reference(pages_id)),
                (
                    "Names",
                    lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([(
                        "JavaScript",
                        lopdf::Object::Dictionary(lopdf::Dictionary::from_iter([(
                            "Names",
                            lopdf::Object::Array(vec!["evil".into()]),
                        )])),
                    )])),
                ),
            ])),
        );
        document
            .trailer
            .set("Root", lopdf::Object::Reference(catalog_id));
        let mut out = Vec::new();
        document.save_to(&mut out).expect("the fixture PDF saves");
        let path = write_input(&out);
        match extract(&request_for(path.path(), "application/pdf")) {
            Err(refusal) => assert_eq!(refusal.kind, "pdf_javascript", "{:?}", refusal.message),
            Ok(_) => panic!("the JS-bearing PDF must refuse"),
        }

        // An unsupported media type names itself.
        let path = write_input(b"whatever");
        match extract(&request_for(path.path(), "video/mp4")) {
            Err(refusal) => assert_eq!(refusal.kind, "media_type_unsupported"),
            Ok(_) => panic!("the unsupported type must refuse"),
        }
    }

    fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, data) in entries.iter().copied() {
            writer
                .start_file(name, zip::write::SimpleFileOptions::default())
                .expect("the entry starts");
            use std::io::Write;
            writer.write_all(data).expect("the entry writes");
        }
        writer.finish().expect("the zip finishes").into_inner()
    }

    #[test]
    fn the_zip_extracts_text_entries_and_excludes_binaries() {
        let bytes = zip_bytes(&[
            ("a.txt", b"hello zip"),
            ("data.bin", &[0u8, 1, 2, 3, 255]),
            ("dir/nested.txt", b"nested"),
        ]);
        let path = write_input(&bytes);
        let response =
            extract(&request_for(path.path(), "application/zip")).expect("the zip extracts");
        let texts: Vec<&str> = response.chunks.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"hello zip"));
        assert!(texts.contains(&"nested"));
        assert_eq!(response.excluded, vec!["data.bin".to_owned()]);
    }

    #[test]
    fn the_zip_refusals_name_themselves() {
        // The nested archive.
        let nested = zip_bytes(&[("inner.txt", b"x")]);
        let bytes = zip_bytes(&[("outer.zip", &nested)]);
        let path = write_input(&bytes);
        match extract(&request_for(path.path(), "application/zip")) {
            Err(refusal) => assert_eq!(refusal.kind, "nested_archive", "{:?}", refusal.message),
            Ok(_) => panic!("the nested archive must refuse"),
        }

        // The traversal entry name.
        let bytes = zip_bytes(&[("../evil.txt", b"x")]);
        let path = write_input(&bytes);
        match extract(&request_for(path.path(), "application/zip")) {
            Err(refusal) => assert_eq!(refusal.kind, "path_traversal"),
            Ok(_) => panic!("the traversal must refuse"),
        }

        // The bomb: a highly compressible entry trips the ratio brake.
        let bomb = zip_bytes(&[("bomb.txt", &vec![b'0'; 1_000_000])]);
        let path = write_input(&bomb);
        match extract(&request_for(path.path(), "application/zip")) {
            Err(refusal) => {
                assert_eq!(
                    refusal.kind, "decompression_ratio_exceeded",
                    "{:?}",
                    refusal.message
                )
            }
            Ok(_) => panic!("the bomb must trip the ratio brake"),
        }
    }

    fn tar_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, data) in entries.iter().copied() {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, name, data)
                .expect("the entry appends");
        }
        builder.into_inner().expect("the tar finishes")
    }

    #[test]
    fn the_tar_extracts_text_entries() {
        let bytes = tar_bytes(&[("a.txt", b"hello tar"), ("b.txt", b"second")]);
        let path = write_input(&bytes);
        let response =
            extract(&request_for(path.path(), "application/x-tar")).expect("the tar extracts");
        let texts: Vec<&str> = response.chunks.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"hello tar"));
        assert!(texts.contains(&"second"));
    }

    #[test]
    fn the_feed_extracts_title_and_entries() {
        let atom = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Feed</title>
  <entry>
    <title>First</title>
    <summary>the first entry</summary>
    <content type="text">full first content</content>
  </entry>
  <entry>
    <title>Second</title>
    <summary>the second entry</summary>
  </entry>
</feed>"#;
        let path = write_input(atom.as_bytes());
        let response =
            extract(&request_for(path.path(), "application/atom+xml")).expect("the feed extracts");
        assert!(response.chunks[0].text.contains("Example Feed"));
        let joined: String = response
            .chunks
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("First"));
        assert!(joined.contains("full first content"));
        assert!(joined.contains("Second"));
        // A malformed feed names itself.
        let path = write_input(b"<not-xml");
        match extract(&request_for(path.path(), "application/rss+xml")) {
            Err(refusal) => assert_eq!(refusal.kind, "feed_unreadable"),
            Ok(_) => panic!("the malformed feed must refuse"),
        }
    }

    /// 🔴 `SIGNOFF-REPAIR.11.54`: R2 advertises `application/rss+xml`, and an
    /// RSS 2.0 feed went to the Atom parser, which refuses an `<rss>` root, so
    /// every RSS feed ended `feed_unreadable`.
    #[test]
    fn an_rss_feed_extracts_its_channel_and_its_items() {
        let rss = br#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel><title>Release notes</title><description>What changed</description>
<item><title>Version 2</title><description>Faster &amp; safer</description></item>
<item><title>Version 1</title><description><![CDATA[The <b>first</b> one]]></description></item>
</channel></rss>"#;
        let path = write_input(rss);
        let response = extract(&request_for(path.path(), "application/rss+xml"))
            .expect("the RSS feed extracts");
        let texts: Vec<&str> = response.chunks.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "Release notes\nWhat changed",
                "Version 2\nFaster & safer",
                "Version 1\nThe <b>first</b> one"
            ]
        );
    }

    /// 🔴 `SIGNOFF-REPAIR.11.54`: an archive nested inside another was known by
    /// its entry's NAME only, so one under any other name was excluded as
    /// binary instead of being refused by name, as `deployment.md` says.
    #[test]
    fn a_nested_archive_is_refused_by_its_content_not_only_its_name() {
        let inner_zip = zip_bytes(&[("inner.txt", b"x")]);
        let gzip = vec![
            0x1f, 0x8b, 0x08, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let inner_tar = tar_bytes(&[("inner.txt", b"x")]);
        for (label, inner) in [("zip", &inner_zip), ("gzip", &gzip), ("tar", &inner_tar)] {
            for (outer, media_type) in [
                (
                    zip_bytes(&[("notes.bin", inner.as_slice())]),
                    "application/zip",
                ),
                (
                    tar_bytes(&[("notes.bin", inner.as_slice())]),
                    "application/x-tar",
                ),
            ] {
                let path = write_input(&outer);
                match extract(&request_for(path.path(), media_type)) {
                    Err(refusal) => assert_eq!(
                        refusal.kind, "nested_archive",
                        "a {label} inside a {media_type}: {:?}",
                        refusal.message
                    ),
                    Ok(response) => panic!(
                        "a {label} named notes.bin inside a {media_type} must refuse, not {:?}",
                        response.excluded
                    ),
                }
            }
        }
    }

    /// Every signature `is_archive_bytes` knows, each beside a near-miss that
    /// must stay text (`SIGNOFF-REPAIR.11.54`; `cargo mutants` found the six it
    /// had not been shown).
    #[test]
    fn every_archive_signature_is_known_and_near_misses_are_not() {
        let mut tar = vec![0u8; 512];
        tar[257..262].copy_from_slice(b"ustar");
        let archives: [(&str, Vec<u8>); 8] = [
            ("zip local header", b"PK\x03\x04rest".to_vec()),
            ("empty zip", b"PK\x05\x06rest".to_vec()),
            ("spanned zip", b"PK\x07\x08rest".to_vec()),
            ("gzip", vec![0x1f, 0x8b, 8, 0]),
            ("tar", tar),
            ("bzip2", b"BZh91AY&SYrest".to_vec()),
            ("xz", vec![0xfd, b'7', b'z', b'X', b'Z', 0x00, 1]),
            ("7z", vec![b'7', b'z', 0xbc, 0xaf, 0x27, 0x1c, 0]),
        ];
        for (label, bytes) in &archives {
            assert!(is_archive_bytes(bytes), "{label} is an archive");
        }
        let texts: [(&str, &[u8]); 6] = [
            ("text that begins BZh", b"BZh is how this sentence starts"),
            (
                "BZh with a digit, no block magic",
                b"BZh9 and then ordinary words",
            ),
            ("BZh then the magic, no digit", b"BZhx1AY&SY"),
            ("PK and other bytes", b"PK is two letters"),
            ("a short text", b"hello"),
            ("xz's first byte only", &[0xfd, b'x']),
        ];
        for (label, bytes) in texts {
            assert!(!is_archive_bytes(bytes), "{label} is not an archive");
        }
    }

    #[test]
    fn the_output_ceiling_names_itself() {
        let bytes = zip_bytes(&[("a.txt", b"x".repeat(2048).as_slice())]);
        let path = write_input(&bytes);
        let mut request = request_for(path.path(), "application/zip");
        request.limits.max_output_bytes = 100;
        request.limits.max_decompression_ratio = 10_000.0;
        match extract(&request) {
            Err(refusal) => assert_eq!(refusal.kind, "output_too_large"),
            Ok(_) => panic!("the output ceiling must trip"),
        }
    }
}

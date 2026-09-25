//! The stream plumbing both provider-CLI adapters share (`SIGNOFF-REPAIR.10.1.2`).
//!
//! [`crate::codex`] and [`crate::claude`] each carried their own copy of the
//! same two readers, and both copies had the same three defects:
//!
//! - the stdout reader and the stderr drain read to a NEWLINE with no limit, so
//!   a child that writes megabytes without one is buffered whole before any
//!   bound is consulted;
//! - the failure reason's stderr tail was `buf[buf.len() - 1024..]`, a BYTE
//!   slice of a `String`, which panics when the offset falls inside a character;
//! - the drain was `while let Ok(Some(line)) = lines.next_line()`, which ENDS at
//!   the first stderr line that is not UTF-8. Ending the loop drops the reader
//!   and CLOSES the pipe, so the child's next stderr write fails, and a process
//!   that does not ignore SIGPIPE is killed by it.
//!
//! One copy now, read as BYTES, so no content can end a drain early and no
//! bound depends on where a newline happens to fall.

use std::sync::Arc;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;

/// The longest stdout event line an adapter reads: 2 MiB.
///
/// ⭐ Chosen from the supervisor's bound, not beside it. The supervisor accepts
/// at most `reasonbraid-node::supervisor::MAX_OUTPUT_BYTES` (256 KiB) of output
/// per attempt, and one event line carries at most one chunk of it, JSON-escaped.
/// The worst escape (`\uXXXX`) is six bytes per byte, so a line carrying the
/// largest chunk the supervisor would accept is at most 1.5 MiB plus its
/// envelope. 2 MiB never cuts a line the supervisor could have taken, and it
/// bounds what a child can make the adapter allocate.
pub const MAX_LINE_BYTES: usize = 2 * 1024 * 1024;

/// The stderr bytes kept: the LAST 8 KiB of the stream, where an error is.
pub(crate) const STDERR_KEPT_BYTES: usize = 8 * 1024;

/// The stderr a failure reason carries: at most the last 1 KiB, in whole
/// characters.
pub(crate) const STDERR_TAIL_BYTES: usize = 1024;

/// One line of a child's stdout, as [`read_line_bounded`] found it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Line {
    /// A complete line, without its newline.
    Text(String),
    /// A line that is not UTF-8, so it cannot be a JSONL event.
    NotUtf8,
    /// A line longer than the bound. Its bytes were consumed and discarded,
    /// never held.
    TooLong,
    /// The stream ended.
    Eof,
}

/// Read one line, holding at most `max` bytes of it.
///
/// A line over `max` is still read to its end, so the next call starts at the
/// next line, but its bytes are dropped as they arrive rather than kept.
pub(crate) async fn read_line_bounded<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> std::io::Result<Line> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut read_any = false;
    let mut too_long = false;
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            break; // the stream ended, possibly mid-line
        }
        read_any = true;
        let (take, ends_line) = match available.iter().position(|b| *b == b'\n') {
            Some(newline) => (newline + 1, true),
            None => (available.len(), false),
        };
        if !too_long {
            let content = take - usize::from(ends_line);
            if bytes.len() + content > max {
                too_long = true;
                bytes = Vec::new(); // never hold an over-long line
            } else {
                bytes.extend_from_slice(&available[..content]);
            }
        }
        reader.consume(take);
        if ends_line {
            break;
        }
    }
    if !read_any {
        return Ok(Line::Eof);
    }
    if too_long {
        return Ok(Line::TooLong);
    }
    Ok(String::from_utf8(bytes).map_or(Line::NotUtf8, Line::Text))
}

/// Drain a child's stderr to its END, keeping the last [`STDERR_KEPT_BYTES`].
///
/// The returned handle MUST be awaited before the buffer is read: the task may
/// not have consumed the pipe's tail when stdout reaches EOF (`PHASE-1-MAINT-2`,
/// the race that left a reason's stderr tail EMPTY).
pub(crate) fn drain_stderr<R>(mut stderr: R) -> (Arc<Mutex<Vec<u8>>>, tokio::task::JoinHandle<()>)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let kept = Arc::new(Mutex::new(Vec::new()));
    let out = Arc::clone(&kept);
    let handle = tokio::spawn(async move {
        let mut chunk = [0u8; 4096];
        // ⛔ Only EOF or an I/O error ends this loop. No CONTENT can: bytes are
        // kept as bytes, and turned into text only when a reason is written.
        while let Ok(read) = stderr.read(&mut chunk).await {
            if read == 0 {
                break;
            }
            let mut buf = out.lock().await;
            buf.extend_from_slice(&chunk[..read]);
            if buf.len() > STDERR_KEPT_BYTES {
                let excess = buf.len() - STDERR_KEPT_BYTES;
                buf.drain(..excess);
            }
        }
    });
    (kept, handle)
}

/// The last [`STDERR_TAIL_BYTES`] of the kept stderr, cut on a character
/// boundary. Invalid UTF-8 is shown as U+FFFD rather than refused.
pub(crate) fn stderr_tail(kept: &[u8]) -> String {
    let text = String::from_utf8_lossy(kept);
    let mut start = text.len().saturating_sub(STDERR_TAIL_BYTES);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn lines(input: &[u8], max: usize) -> Vec<Line> {
        let mut reader = input;
        let mut out = Vec::new();
        loop {
            let line = read_line_bounded(&mut reader, max)
                .await
                .expect("a slice reads");
            let done = line == Line::Eof;
            out.push(line);
            if done {
                return out;
            }
        }
    }

    #[tokio::test]
    async fn lines_are_split_and_the_last_may_lack_a_newline() {
        assert_eq!(
            lines(b"one\ntwo\nthree", 16).await,
            vec![
                Line::Text("one".into()),
                Line::Text("two".into()),
                Line::Text("three".into()),
                Line::Eof
            ]
        );
        assert_eq!(lines(b"", 16).await, vec![Line::Eof]);
        assert_eq!(
            lines(b"\n", 16).await,
            vec![Line::Text(String::new()), Line::Eof]
        );
    }

    /// The bound is exact, and an over-long line does not swallow the next one.
    #[tokio::test]
    async fn the_bound_is_exact_and_an_over_long_line_is_skipped_whole() {
        assert_eq!(
            lines(b"12345\n123456\nok\n", 5).await,
            vec![
                Line::Text("12345".into()),
                Line::TooLong,
                Line::Text("ok".into()),
                Line::Eof
            ]
        );
        assert_eq!(lines(b"123456", 5).await, vec![Line::TooLong, Line::Eof]);
    }

    #[tokio::test]
    async fn a_line_that_is_not_utf8_is_named_and_the_next_one_read() {
        assert_eq!(
            lines(b"\xff\xfe\nok\n", 16).await,
            vec![Line::NotUtf8, Line::Text("ok".into()), Line::Eof]
        );
    }

    /// The drain keeps EXACTLY the last `STDERR_KEPT_BYTES`, and no content ends
    /// it: the input carries every byte value, invalid UTF-8 included.
    #[tokio::test]
    async fn the_drain_keeps_exactly_the_last_bytes_and_reads_to_the_end() {
        let input: Vec<u8> = (0..20_000u32).map(|i| (i % 256) as u8).collect();
        let (kept, drain) = drain_stderr(std::io::Cursor::new(input.clone()));
        drain.await.expect("the drain task ends");
        assert_eq!(
            *kept.lock().await,
            input[input.len() - STDERR_KEPT_BYTES..],
            "the last {STDERR_KEPT_BYTES} bytes, exactly"
        );
        let (kept, drain) = drain_stderr(std::io::Cursor::new(b"short".to_vec()));
        drain.await.expect("the drain task ends");
        assert_eq!(*kept.lock().await, b"short");
    }

    /// The retention is the documented 8 KiB. The drain test above compares
    /// against the constant itself, so it cannot notice the constant changing;
    /// this pins the value the book states.
    #[test]
    fn the_documented_retention_is_eight_kib() {
        assert_eq!(STDERR_KEPT_BYTES, 8192);
        assert_eq!(STDERR_TAIL_BYTES, 1024);
    }

    /// A tail that would begin inside a character begins at the next one.
    #[test]
    fn the_tail_is_cut_on_a_character_boundary() {
        // 700 two-byte characters and a newline: 1,401 bytes, so the 1,024-byte
        // cut would begin at byte 377, inside a character (the stub's shape).
        let text = format!("{}\n", "é".repeat(700));
        assert!(!text.is_char_boundary(text.len() - STDERR_TAIL_BYTES));
        let tail = stderr_tail(text.as_bytes());
        assert_eq!(tail.len(), 1023, "it begins at the next character");
        assert!(
            tail.trim_end().chars().all(|c| c == 'é'),
            "whole characters only"
        );
        assert_eq!(stderr_tail(b"short"), "short");
        assert_eq!(stderr_tail(b"\xffbad"), "\u{fffd}bad");
    }
}

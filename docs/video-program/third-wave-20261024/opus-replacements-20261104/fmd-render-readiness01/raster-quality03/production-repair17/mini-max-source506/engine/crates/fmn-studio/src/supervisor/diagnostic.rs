//! Bounded user-visible worker causes, including failures before handshake.
//! Crash envelopes keep their existing wire format; this is the human error
//! path, which previously discarded the captured stderr when a worker exited.

use super::*;

pub(super) fn display(error: &ChannelError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "worker channel {:?}: ", error.kind)?;
    if !visible(f, &error.detail, 2048)? {
        f.write_str(" [truncated]")?;
    }
    if !error.stderr_tail.is_empty() {
        f.write_str("\nworker stderr (tail):\n")?;
        let start = error.stderr_tail.len().saturating_sub(4096);
        let tail = String::from_utf8_lossy(&error.stderr_tail[start..]);
        if start > 0 {
            f.write_str("[earlier output omitted]\n")?;
        }
        if !visible(f, &tail, 4096)? {
            f.write_str(" [truncated]")?;
        }
    }
    Ok(())
}

// Bound emitted bytes as well as source bytes. A child cannot inflate a short
// control-character stream into an unbounded formatted diagnostic, or execute
// terminal escapes. Preserve ordinary line breaks for readable tracebacks.
fn visible(f: &mut fmt::Formatter<'_>, text: &str, budget: usize) -> Result<bool, fmt::Error> {
    let mut remaining = budget;
    for c in text.chars() {
        let escape = (c.is_control() && !matches!(c, '\n' | '\t'))
            || ('\u{202a}'..='\u{202e}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c);
        if escape {
            for safe in c.escape_default() {
                if remaining == 0 {
                    return Ok(false);
                }
                write!(f, "{safe}")?;
                remaining -= 1;
            }
        } else {
            if remaining < c.len_utf8() {
                return Ok(false);
            }
            write!(f, "{c}")?;
            remaining -= c.len_utf8();
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_worker_diagnostic_includes_authored_cause_without_terminal_controls() {
        let error = ChannelError {
            kind: ChannelFailureKind::Closed,
            detail: "worker IPC pipe closed".into(),
            stderr_tail: b"Python Studio worker failed: invalid updater\n\x1b[2J\x00\xff".to_vec(),
        };
        let text = error.to_string();
        assert!(text.starts_with("worker channel Closed: worker IPC pipe closed"));
        assert!(text.contains("Python Studio worker failed: invalid updater\n"));
        assert!(text.contains('�'));
        assert!(!text.contains('\x1b') && !text.contains('\0'));
    }

    #[test]
    fn human_worker_errors_are_bounded_and_keep_the_latest_stderr() {
        let mut error = ChannelError {
            kind: ChannelFailureKind::Timeout,
            detail: "d".repeat(100_000),
            stderr_tail: vec![b'x'; 100_000],
        };
        error.stderr_tail.extend_from_slice(b"LATEST CAUSE");
        let text = error.to_string();
        assert!(text.len() < 8192);
        assert!(text.ends_with("LATEST CAUSE"));
        assert!(text.contains("[earlier output omitted]"));
        error.stderr_tail = vec![0x1b; 100_000];
        let text = error.to_string();
        assert!(text.len() < 8192);
        assert!(!text.contains('\x1b'));
        let plain = ChannelError {
            kind: ChannelFailureKind::Closed,
            detail: "closed".into(),
            stderr_tail: Vec::new(),
        };
        assert_eq!(plain.to_string(), "worker channel Closed: closed");
    }
}

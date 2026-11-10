//! Process-output publication with deterministic two-stream error handling.
//!
//! A closed downstream pipe is ordinary CLI lifecycle: it must not replace the
//! command's already-decided typed exit code, and it must not prevent the other
//! stream from being attempted. Other I/O failures remain internal failures,
//! but only after both stdout and stderr have had one publication attempt.

use std::io::{self, Write};

/// The process stream whose publication failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One non-broken-pipe output failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PublicationError {
    stream: Stream,
    kind: io::ErrorKind,
}

impl PublicationError {
    /// Stream whose write failed.
    #[cfg(test)]
    pub(crate) const fn stream(self) -> Stream {
        self.stream
    }

    /// Stable operating-system error category.
    #[cfg(test)]
    pub(crate) const fn kind(self) -> io::ErrorKind {
        self.kind
    }
}

fn write_stream(stream: Stream, writer: &mut dyn Write, bytes: &[u8]) -> Option<PublicationError> {
    if bytes.is_empty() {
        return None;
    }
    match writer.write_all(bytes) {
        Ok(()) => None,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => None,
        Err(error) => Some(PublicationError {
            stream,
            kind: error.kind(),
        }),
    }
}

/// Publish both process streams exactly once.
///
/// Stdout is attempted first to preserve normal terminal ordering, but stderr
/// is attempted even when stdout reports a non-broken-pipe failure. When both
/// streams fail, stdout's error wins deterministically. Broken pipes are
/// ignored because the consumer intentionally closed that stream.
pub(crate) fn publish(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    stdout_bytes: &[u8],
    stderr_bytes: &[u8],
) -> Result<(), PublicationError> {
    let stdout_error = write_stream(Stream::Stdout, stdout, stdout_bytes);
    let stderr_error = write_stream(Stream::Stderr, stderr, stderr_bytes);
    if let Some(error) = stdout_error.or(stderr_error) {
        Err(error)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct RecordingWriter {
        bytes: Vec<u8>,
        writes: usize,
        failure: Option<io::ErrorKind>,
    }

    impl RecordingWriter {
        fn failing(kind: io::ErrorKind) -> Self {
            Self {
                failure: Some(kind),
                ..Self::default()
            }
        }
    }

    impl Write for RecordingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            if let Some(kind) = self.failure {
                return Err(io::Error::from(kind));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn broken_stdout_does_not_hide_stderr_or_change_success() {
        let mut stdout = RecordingWriter::failing(io::ErrorKind::BrokenPipe);
        let mut stderr = RecordingWriter::default();
        assert_eq!(
            publish(
                &mut stdout,
                &mut stderr,
                b"ordinary output\n",
                b"typed error\n"
            ),
            Ok(())
        );
        assert_eq!(stdout.writes, 1);
        assert_eq!(stderr.bytes, b"typed error\n");
    }

    #[test]
    fn non_broken_stdout_failure_still_attempts_stderr() {
        let mut stdout = RecordingWriter::failing(io::ErrorKind::PermissionDenied);
        let mut stderr = RecordingWriter::default();
        let error = publish(&mut stdout, &mut stderr, b"output", b"diagnostic")
            .expect_err("permission failure is internal");
        assert_eq!(error.stream(), Stream::Stdout);
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(stderr.bytes, b"diagnostic");
    }

    #[test]
    fn non_broken_stderr_failure_is_reported() {
        let mut stdout = RecordingWriter::default();
        let mut stderr = RecordingWriter::failing(io::ErrorKind::WriteZero);
        let error = publish(&mut stdout, &mut stderr, b"output", b"diagnostic")
            .expect_err("stderr write failure is internal");
        assert_eq!(error.stream(), Stream::Stderr);
        assert_eq!(error.kind(), io::ErrorKind::WriteZero);
        assert_eq!(stdout.bytes, b"output");
    }

    #[test]
    fn empty_streams_are_not_written() {
        let mut stdout = RecordingWriter::failing(io::ErrorKind::PermissionDenied);
        let mut stderr = RecordingWriter::failing(io::ErrorKind::PermissionDenied);
        assert_eq!(publish(&mut stdout, &mut stderr, b"", b""), Ok(()));
        assert_eq!(stdout.writes, 0);
        assert_eq!(stderr.writes, 0);
    }

    #[test]
    fn simultaneous_failures_have_deterministic_stdout_precedence() {
        let mut stdout = RecordingWriter::failing(io::ErrorKind::PermissionDenied);
        let mut stderr = RecordingWriter::failing(io::ErrorKind::WriteZero);
        let error = publish(&mut stdout, &mut stderr, b"output", b"diagnostic")
            .expect_err("both writes fail");
        assert_eq!(error.stream(), Stream::Stdout);
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(stdout.writes, 1);
        assert_eq!(stderr.writes, 1);
    }
}

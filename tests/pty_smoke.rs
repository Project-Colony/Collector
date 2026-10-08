//! Runs the real binary in a pseudo-terminal (ConPTY on Windows), waits for
//! the first frame, presses q and checks that Collector exits cleanly and, on
//! Unix, hands the terminal back the way it found it: same escape sequences
//! last, same line discipline (raw mode off).

use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

/// Generous on purpose: this is a smoke test, not a benchmark, and a cold CI
/// runner can take seconds over the first full process scan.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(15);

/// A box title from the first frame. ConPTY renders the screen itself and
/// need not pass escape sequences such as the alternate-screen switch through,
/// so the test waits for text rather than for a sequence. One word: ratatui
/// skips a blank cell with a cursor move, so a space splits text in two.
const FRAME_MARKER: &[u8] = b"Network";
/// Cursor position request. ConPTY sends one when it starts and waits for the
/// answer, as a real terminal would give.
const CURSOR_QUERY: &[u8] = b"\x1b[6n";
/// The last bytes Collector writes: leave the alternate screen, show the cursor.
const RESTORE: &[u8] = b"\x1b[?1049l\x1b[?25h";

struct Output {
    chunks: Receiver<Vec<u8>>,
    bytes: Vec<u8>,
    answered: usize,
    writer: Box<dyn Write + Send>,
}

impl Output {
    /// Collects output for up to `wait`, answering cursor queries on the way.
    /// Returns false once the reader has reached the end of the output.
    fn pump(&mut self, wait: Duration) -> bool {
        let alive = match self.chunks.recv_timeout(wait) {
            Ok(chunk) => {
                self.bytes.extend_from_slice(&chunk);
                true
            }
            Err(RecvTimeoutError::Timeout) => true,
            Err(RecvTimeoutError::Disconnected) => false,
        };
        let queries = count(&self.bytes, CURSOR_QUERY);
        while self.answered < queries {
            let _ = self.writer.write_all(b"\x1b[1;1R");
            let _ = self.writer.flush();
            self.answered += 1;
        }
        alive
    }

    fn tail(&self) -> String {
        let start = self.bytes.len().saturating_sub(1500);
        String::from_utf8_lossy(&self.bytes[start..])
            .escape_debug()
            .to_string()
    }
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

#[test]
fn quits_on_q_and_restores_the_terminal() {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open a pseudo-terminal");
    // Both sides of a pseudo-terminal share one set of terminal settings, so
    // the master side sees what the child leaves behind.
    #[cfg(unix)]
    let modes_before = pair
        .master
        .get_termios()
        .expect("read the terminal settings")
        .local_flags;
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_collector"));
    cmd.env("TERM", "xterm-256color");
    let mut child = pair.slave.spawn_command(cmd).expect("start collector");
    // Only the child may hold the terminal side open, or on Unix the reader
    // would never see the end of the output.
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().expect("pty reader");
    let (tx, chunks) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while let Ok(n @ 1..) = reader.read(&mut buf) {
            if tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut out = Output {
        chunks,
        bytes: Vec::new(),
        answered: 0,
        writer: pair.master.take_writer().expect("pty writer"),
    };

    let deadline = Instant::now() + FIRST_FRAME_TIMEOUT;
    while count(&out.bytes, FRAME_MARKER) == 0 {
        if Instant::now() > deadline || !out.pump(Duration::from_millis(100)) {
            let _ = child.kill();
            panic!(
                "no frame within {FIRST_FRAME_TIMEOUT:?}; output ends with:\n{}",
                out.tail()
            );
        }
    }

    out.writer.write_all(b"q").expect("press q");
    out.writer.flush().expect("press q");

    let deadline = Instant::now() + EXIT_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll collector") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!(
                "still running {EXIT_TIMEOUT:?} after q; output ends with:\n{}",
                out.tail()
            );
        }
        out.pump(Duration::from_millis(50));
    };

    #[cfg(unix)]
    let modes_after = pair.master.get_termios().map(|t| t.local_flags);

    // ConPTY keeps its output pipe open until the pseudo-console is closed,
    // which dropping the master side does. On Unix the reader stops by itself
    // once the child has exited.
    drop(pair.master);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && out.pump(Duration::from_millis(100)) {}

    assert!(
        status.success(),
        "exited with {status}; output ends with:\n{}",
        out.tail()
    );
    if cfg!(unix) {
        assert!(
            out.bytes.ends_with(RESTORE),
            "the terminal was not restored last; output ends with:\n{}",
            out.tail()
        );
    }
    #[cfg(unix)]
    assert_eq!(modes_after, Some(modes_before), "raw mode was left on");
}

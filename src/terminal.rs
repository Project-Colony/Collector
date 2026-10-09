//! Taking over the terminal and giving it back, whatever ends the program: a
//! quit key, a quit signal from outside, an error or a panic.

use std::io::{self, IsTerminal, Stdout};
use std::panic;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use crossterm::cursor::Show;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

/// Redirected output, a pipe or `ssh host collector` without -t: drawing there
/// would only fill a file or a pipe with escape sequences.
pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// A panic would otherwise print into the alternate screen, which the terminal
/// then throws away, and leave the shell in raw mode with no cursor. Restore
/// first, then let the default hook print the message.
pub fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore();
        default_hook(info);
    }));
}

/// A quit signal from outside stores its number in the returned flag instead
/// of killing Collector on the spot; the event loop sees it within a tick and
/// returns, so the terminal is restored. In raw mode Ctrl+C is a key, not a
/// signal, so SIGINT only comes from another process.
pub fn quit_signal() -> io::Result<Arc<AtomicUsize>> {
    let quit_signal = Arc::new(AtomicUsize::new(0));
    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
        for signal in [SIGTERM, SIGHUP, SIGINT, SIGQUIT] {
            let flag = Arc::clone(&quit_signal);
            signal_hook::flag::register_usize(signal, flag, signal as usize).map_err(|err| {
                io::Error::new(err.kind(), format!("cannot handle signal {signal}: {err}"))
            })?;
        }
    }
    Ok(quit_signal)
}

/// Once the terminal is back: dies of the quit signal, if one came, as its
/// sender expects (a shell reports 128 + its number). The exit code returned
/// is only a fallback, for when the default handler does not end the process.
pub fn die_of_signal(quit_signal: &AtomicUsize) -> Option<ExitCode> {
    #[cfg(unix)]
    if let signal @ 1.. = quit_signal.load(std::sync::atomic::Ordering::SeqCst) {
        let signal = signal as i32;
        let _ = signal_hook::low_level::emulate_default_handler(signal);
        return Some(ExitCode::from(128 + signal as u8));
    }
    #[cfg(not(unix))]
    let _ = quit_signal;
    None
}

/// Raw mode on, alternate screen entered. [`restore`] undoes both.
pub fn enable() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

/// Leaves the alternate screen, shows the cursor and turns raw mode off. Safe
/// to run more than once, and from the panic hook.
pub fn restore() -> io::Result<()> {
    let raw = disable_raw_mode();
    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    raw
}

/// Only presses count: Windows also reports key releases, so without this
/// filter every key arrived twice. In raw mode Ctrl+C is a key, not a signal.
pub fn is_quit(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && match key.code {
            KeyCode::Char('q') => true,
            KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
            _ => false,
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_key_presses_quit() {
        let press = |code, modifiers| KeyEvent::new(code, modifiers);
        assert!(is_quit(&press(KeyCode::Char('q'), KeyModifiers::NONE)));
        assert!(is_quit(&press(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!is_quit(&press(KeyCode::Char('c'), KeyModifiers::NONE)));

        let release = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert!(!is_quit(&release));
    }
}

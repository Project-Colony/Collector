use std::error::Error;
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::{env, panic};

use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Row, Table};
use ratatui::{Frame, Terminal};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const TICK_RATE: Duration = Duration::from_millis(1000);
const PROCESS_LIMIT: usize = 10;
const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = "\
Usage: collector [OPTION]

A terminal system monitor: CPU, memory, disks, network and processes.

Options:
  -h, --help     Print this help and exit
  -V, --version  Print the version and exit

Keys: q or Ctrl+C quits.
";

/// `bytes` moved over `elapsed`, per second. sysinfo's `received()` and
/// `transmitted()` already count from the previous refresh, and drawing takes
/// time too, so the interval is measured rather than assumed.
fn per_second(bytes: u64, elapsed: Duration) -> f64 {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 { bytes as f64 / secs } else { 0.0 }
}

/// A reading as a gauge percentage. `Gauge::percent` panics above 100, and a
/// reading can go over: used memory can exceed the total under cgroup
/// accounting, and a cast alone would turn a huge value into 65535.
fn gauge_percent(value: f64) -> u16 {
    if value.is_finite() {
        value.round().clamp(0.0, 100.0) as u16
    } else {
        0
    }
}

struct App {
    system: System,
    disks: Disks,
    networks: Networks,
    /// Bytes received and transmitted per second, all interfaces together.
    network_rate: (f64, f64),
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh(true);

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        Self {
            system,
            disks,
            networks,
            network_rate: (0.0, 0.0),
            last_tick: Instant::now(),
        }
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh(true);
        self.networks.refresh(true);
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        let (rx, tx) = self.networks.values().fold((0, 0), |(rx, tx), data| {
            (rx + data.received(), tx + data.transmitted())
        });
        self.network_rate = (per_second(rx, elapsed), per_second(tx, elapsed));
        self.last_tick = now;
    }
}

enum Mode {
    Run,
    Help,
    Version,
}

/// Parsed by hand, before the terminal is touched, so `--version` and `--help`
/// work with no TTY at all (the release smoke test runs `--version` headless).
/// Returns the offending argument on error.
fn parse_args(args: &[OsString]) -> Result<Mode, String> {
    let mode = match args.first().map(|arg| arg.to_str()) {
        None => return Ok(Mode::Run),
        Some(Some("-h" | "--help")) => Mode::Help,
        Some(Some("-V" | "--version")) => Mode::Version,
        Some(_) => return Err(args[0].to_string_lossy().into_owned()),
    };
    match args.get(1) {
        None => Ok(mode),
        Some(extra) => Err(extra.to_string_lossy().into_owned()),
    }
}

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    match parse_args(&args) {
        Ok(Mode::Run) => {}
        Ok(Mode::Version) => {
            println!("collector {VERSION}");
            return ExitCode::SUCCESS;
        }
        Ok(Mode::Help) => {
            print!("collector {VERSION}\n\n{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(arg) => {
            eprint!("collector: unexpected argument '{arg}'\n\n{USAGE}");
            return ExitCode::from(2);
        }
    }

    // Redirected output, a pipe or `ssh host collector` without -t: drawing
    // there would only fill a file or a pipe with escape sequences.
    if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
        eprintln!("Collector needs a terminal (with ssh, use ssh -t)");
        return ExitCode::from(1);
    }

    // A panic would otherwise print into the alternate screen, which the
    // terminal then throws away, and leave the shell in raw mode with no
    // cursor. Restore first, then let the default hook print the message.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        default_hook(info);
    }));

    // A quit signal from outside stores its number here instead of killing
    // Collector on the spot; the event loop sees it within a tick and returns,
    // so the terminal is restored below. In raw mode Ctrl+C is a key, not a
    // signal, so SIGINT only comes from another process.
    let quit_signal = Arc::new(AtomicUsize::new(0));
    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
        for signal in [SIGTERM, SIGHUP, SIGINT, SIGQUIT] {
            let flag = Arc::clone(&quit_signal);
            if let Err(err) = signal_hook::flag::register_usize(signal, flag, signal as usize) {
                eprintln!("collector: cannot handle signal {signal}: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let result = run(&quit_signal);
    let restored = restore_terminal();
    let result = result.and(restored.map_err(Into::into));
    if let Err(err) = &result {
        eprintln!("collector: {err}");
    }

    // The terminal is back: now die of the signal, as its sender expects (a
    // shell reports 128 + its number). The exit code is only a fallback.
    #[cfg(unix)]
    if let signal @ 1.. = quit_signal.load(Ordering::SeqCst) {
        let signal = signal as i32;
        let _ = signal_hook::low_level::emulate_default_handler(signal);
        return ExitCode::from(128 + signal as u8);
    }
    if result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Leaves the alternate screen, shows the cursor and turns raw mode off. Safe
/// to run more than once, and from the panic hook.
fn restore_terminal() -> io::Result<()> {
    let raw = disable_raw_mode();
    execute!(io::stdout(), LeaveAlternateScreen, Show)?;
    raw
}

fn run(quit_signal: &AtomicUsize) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    run_app(&mut terminal, quit_signal)
}

/// Only presses count: Windows also reports key releases, so without this
/// filter every key arrived twice. In raw mode Ctrl+C is a key, not a signal.
fn is_quit(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && match key.code {
            KeyCode::Char('q') => true,
            KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
            _ => false,
        }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    quit_signal: &AtomicUsize,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    // Checked after every poll, which waits one tick at most.
    while quit_signal.load(Ordering::SeqCst) == 0 {
        // Only on the tick: any other event (a held key, a key release on
        // Windows, a resize) just redraws. Refreshing on each one rescanned
        // every process many times a second and sampled CPU usage over
        // intervals too short to mean anything.
        if app.last_tick.elapsed() >= TICK_RATE {
            app.refresh();
        }
        terminal.draw(|frame| render_ui(frame, &app))?;

        let timeout = TICK_RATE.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && is_quit(&key)
        {
            break;
        }
    }

    Ok(())
}

fn render_ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(10),
        ])
        .split(frame.area());

    render_cpu_memory(frame, app, main_layout[0]);
    render_disk_network(frame, app, main_layout[1]);
    render_processes(frame, app, main_layout[2]);
}

fn render_cpu_memory(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let cpu_usage = app.system.global_cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(gauge_percent(f64::from(cpu_usage)));
    frame.render_widget(cpu_gauge, layout[0]);

    let total_memory = app.system.total_memory() as f64;
    let used_memory = app.system.used_memory() as f64;
    let memory_percent = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    let memory_gauge = Gauge::default()
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(gauge_percent(memory_percent))
        .label(format!(
            "{:.1} / {:.1} GB",
            used_memory / 1_073_741_824.0,
            total_memory / 1_073_741_824.0
        ));
    frame.render_widget(memory_gauge, layout[1]);
}

fn render_disk_network(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let disk_lines: Vec<Line> = app
        .disks
        .iter()
        .map(|disk| {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;
            Line::from(vec![
                Span::styled(
                    format!("{} ", disk.name().to_string_lossy()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!(
                    "{:.1}/{:.1} GB",
                    used / 1_073_741_824.0,
                    total / 1_073_741_824.0
                )),
            ])
        })
        .collect();

    let disk_block =
        Paragraph::new(disk_lines).block(Block::default().title("Disks").borders(Borders::ALL));
    frame.render_widget(disk_block, layout[0]);

    let (rx, tx) = app.network_rate;
    let net_lines = vec![
        Line::from(format!("RX: {:.1} KB/s", rx / 1024.0)),
        Line::from(format!("TX: {:.1} KB/s", tx / 1024.0)),
        Line::from("q or Ctrl+C to quit"),
    ];
    let net_block =
        Paragraph::new(net_lines).block(Block::default().title("Network").borders(Borders::ALL));
    frame.render_widget(net_block, layout[1]);
}

fn render_processes(frame: &mut Frame, app: &App, area: Rect) {
    let mut processes: Vec<_> = app.system.processes().values().collect();
    let cpu_count = app.system.cpus().len().max(1) as f32;
    processes.sort_by(|a, b| {
        let a_usage = a.cpu_usage() / cpu_count;
        let b_usage = b.cpu_usage() / cpu_count;
        b_usage.total_cmp(&a_usage)
    });

    let rows = processes.into_iter().take(PROCESS_LIMIT).map(|process| {
        let cpu_usage = process.cpu_usage() / cpu_count;
        Row::new(vec![
            process.name().to_string_lossy().into_owned(),
            format!("{:.1}%", cpu_usage),
            format!("{:.1} MB", process.memory() as f64 / 1_048_576.0),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(Row::new(vec!["Process", "CPU", "Memory"]).style(Style::default().fg(Color::Green)))
    .block(
        Block::default()
            .title("Top Processes")
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
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

    #[test]
    fn gauge_percent_stays_in_range() {
        assert_eq!(gauge_percent(f64::NAN), 0);
        assert_eq!(gauge_percent(f64::INFINITY), 0);
        assert_eq!(gauge_percent(-3.0), 0);
        assert_eq!(gauge_percent(150.0), 100);
        assert_eq!(gauge_percent(42.4), 42);
    }

    #[test]
    fn network_rate_is_per_second() {
        assert_eq!(per_second(2048, Duration::from_millis(500)), 4096.0);
        assert_eq!(per_second(2048, Duration::ZERO), 0.0);
    }
}

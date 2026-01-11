use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table};
use ratatui::{Frame, Terminal};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const TICK_RATE: Duration = Duration::from_millis(1000);
const PROCESS_LIMIT: usize = 10;
const COLOR_BACKGROUND: Color = Color::Rgb(12, 12, 12);
const COLOR_BAR_BLUE: Color = Color::Rgb(80, 160, 240);
const COLOR_BAR_GREEN: Color = Color::Rgb(90, 200, 140);
const COLOR_BAR_PINK: Color = Color::Rgb(240, 130, 180);
const COLOR_BAR_YELLOW: Color = Color::Rgb(230, 200, 110);
const COLOR_BORDER_GREEN: Color = Color::Rgb(80, 200, 120);
const COLOR_BORDER_DIM: Color = Color::Rgb(70, 70, 70);
const COLOR_TEXT_DIM: Color = Color::Gray;

struct NetworkTotals {
    received: u64,
    transmitted: u64,
}

impl NetworkTotals {
    fn from_networks(networks: &Networks) -> Self {
        let mut received = 0;
        let mut transmitted = 0;
        for (_name, data) in networks.iter() {
            received += data.received();
            transmitted += data.transmitted();
        }
        Self {
            received,
            transmitted,
        }
    }

    fn delta(&self, newer: &NetworkTotals) -> (u64, u64) {
        (
            newer.received.saturating_sub(self.received),
            newer.transmitted.saturating_sub(self.transmitted),
        )
    }
}

struct App {
    system: System,
    disks: Disks,
    networks: Networks,
    last_network: NetworkTotals,
    last_tick: Instant,
    cpu_history: Vec<u64>,
}

impl App {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh(true);

        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let last_network = NetworkTotals::from_networks(&networks);

        let cpu_history = vec![system.global_cpu_usage().round() as u64];

        Self {
            system,
            disks,
            networks,
            last_network,
            last_tick: Instant::now(),
            cpu_history,
        }
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.last_tick = Instant::now();
        self.push_cpu_history();
    }

    fn push_cpu_history(&mut self) {
        const HISTORY_LIMIT: usize = 60;
        let usage = self.system.global_cpu_usage().round() as u64;
        self.cpu_history.push(usage);
        if self.cpu_history.len() > HISTORY_LIMIT {
            let overflow = self.cpu_history.len() - HISTORY_LIMIT;
            self.cpu_history.drain(0..overflow);
        }
    }

    fn network_delta(&mut self) -> (u64, u64) {
        let current = NetworkTotals::from_networks(&self.networks);
        let delta = self.last_network.delta(&current);
        self.last_network = current;
        delta
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    loop {
        app.refresh();
        terminal.draw(|frame| render_ui(frame, &mut app))?;

        let timeout = TICK_RATE.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn render_ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(8),
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
    let cpu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(layout[0]);

    let cpu_history_block = Block::default()
        .title("CPU")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BORDER_GREEN))
        .style(Style::default().bg(COLOR_BACKGROUND));
    let cpu_history = Sparkline::default()
        .block(cpu_history_block)
        .data(&app.cpu_history)
        .style(Style::default().fg(COLOR_BAR_BLUE));
    frame.render_widget(cpu_history, cpu_layout[0]);

    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER_GREEN))
                .style(Style::default().bg(COLOR_BACKGROUND)),
        )
        .gauge_style(Style::default().fg(COLOR_BAR_BLUE))
        .percent(cpu_usage.round() as u16);
    frame.render_widget(cpu_gauge, cpu_layout[1]);

    let total_memory = app.system.total_memory() as f64;
    let used_memory = app.system.used_memory() as f64;
    let memory_percent = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(layout[1]);

    let memory_gauge = Gauge::default()
        .block(
            Block::default()
                .title("Memory")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER_GREEN))
                .style(Style::default().bg(COLOR_BACKGROUND)),
        )
        .gauge_style(Style::default().fg(COLOR_BAR_GREEN))
        .percent(memory_percent.round() as u16);
    frame.render_widget(memory_gauge, memory_layout[0]);

    let free_memory = app.system.free_memory() as f64;
    let available_memory = app.system.available_memory() as f64;
    let memory_lines = vec![
        Line::from(vec![
            Span::styled("Used ", Style::default().fg(COLOR_BAR_GREEN)),
            Span::raw(format!("{:.1} GB", used_memory / 1_073_741_824.0)),
        ]),
        Line::from(vec![
            Span::styled("Free ", Style::default().fg(COLOR_BAR_BLUE)),
            Span::raw(format!("{:.1} GB", free_memory / 1_073_741_824.0)),
        ]),
        Line::from(vec![
            Span::styled("Avail ", Style::default().fg(COLOR_BAR_YELLOW)),
            Span::raw(format!("{:.1} GB", available_memory / 1_073_741_824.0)),
        ]),
    ];
    let memory_details = Paragraph::new(memory_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER_GREEN))
            .style(Style::default().bg(COLOR_BACKGROUND)),
    );
    frame.render_widget(memory_details, memory_layout[1]);
}

fn render_disk_network(frame: &mut Frame, app: &mut App, area: Rect) {
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
                    Style::default().fg(COLOR_BAR_YELLOW),
                ),
                Span::raw(format!(
                    "{:.1}/{:.1} GB",
                    used / 1_073_741_824.0,
                    total / 1_073_741_824.0
                )),
            ])
        })
        .collect();

    let disk_block = Paragraph::new(disk_lines).block(
        Block::default()
            .title("Disks")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER_DIM))
            .style(Style::default().bg(COLOR_BACKGROUND)),
    );
    frame.render_widget(disk_block, layout[0]);

    let (rx, tx) = app.network_delta();
    let net_lines = vec![
        Line::from(format!("RX: {:.1} KB/s", rx as f64 / 1024.0)),
        Line::from(format!("TX: {:.1} KB/s", tx as f64 / 1024.0)),
        Line::from("Press q to quit"),
    ];
    let net_block = Paragraph::new(net_lines).block(
        Block::default()
            .title("Network")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER_DIM))
            .style(Style::default().bg(COLOR_BACKGROUND)),
    );
    frame.render_widget(net_block, layout[1]);
}

fn render_processes(frame: &mut Frame, app: &App, area: Rect) {
    let mut processes: Vec<_> = app.system.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().total_cmp(&a.cpu_usage()));

    let rows = processes.into_iter().take(PROCESS_LIMIT).map(|process| {
        Row::new(vec![
            Cell::from(Span::styled(
                process.name().to_string_lossy().into_owned(),
                Style::default().fg(COLOR_TEXT_DIM),
            )),
            Cell::from(Span::styled(
                format!("{:.1}%", process.cpu_usage()),
                Style::default().fg(COLOR_BAR_GREEN),
            ))
            .alignment(Alignment::Right),
            Cell::from(Span::styled(
                format!("{:.1} MB", process.memory() as f64 / 1_048_576.0),
                Style::default().fg(COLOR_BAR_BLUE),
            ))
            .alignment(Alignment::Right),
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
    .header(Row::new(vec![
        Cell::from(Span::styled("Process", Style::default().fg(COLOR_TEXT_DIM))),
        Cell::from(Span::styled("CPU", Style::default().fg(COLOR_BAR_YELLOW)))
            .alignment(Alignment::Right),
        Cell::from(Span::styled("Memory", Style::default().fg(COLOR_BAR_PINK)))
            .alignment(Alignment::Right),
    ]))
    .block(
        Block::default()
            .title("Top Processes")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER_GREEN))
            .style(Style::default().bg(COLOR_BACKGROUND)),
    );

    frame.render_widget(table, area);
}

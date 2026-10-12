//! The readings, the Preferences page's state, and the loop that refreshes,
//! draws and reacts to keys and clicks.

use std::collections::HashSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Position;
use sysinfo::{Disk, Disks, Networks, ProcessesToUpdate, System};

use crate::i18n::{self, tf};
use crate::preferences::{self, Preferences, Setting};
use crate::terminal;
use crate::ui::{self, Focus, Hit, Page, preferences::Row};

/// What a key and a click can both do. Each control sends the message its
/// key sends, so the two can never disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    /// The identity button, the page's Close button, `p`, and Esc on the page.
    TogglePreferences,
    /// A category of the open page.
    Category(usize),
    /// A row of the open category that takes focus: focus it, then open or
    /// close it (a section) or move it to its next value (a setting).
    Activate(usize),
}

pub struct App {
    pub system: System,
    disks: Disks,
    networks: Networks,
    /// Bytes received and transmitted per second, all interfaces except
    /// loopback together.
    pub network_rate: (f64, f64),
    last_tick: Instant,
    pub prefs: Preferences,
    /// Where changes are saved; `None` when they cannot be.
    prefs_path: Option<PathBuf>,
    /// A message about the preferences file, shown until the next key or
    /// click.
    pub status: Option<String>,
    /// The Preferences page, while it is open.
    pub page: Option<Page>,
    /// The mouse's cell, for hover.
    pub mouse: Option<Position>,
    /// What the last frame drew that a click can reach.
    hits: Vec<Hit>,
}

impl App {
    fn new(prefs: Preferences, prefs_path: Option<PathBuf>, status: Option<String>) -> Self {
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
            prefs,
            prefs_path,
            status,
            page: None,
            mouse: None,
            hits: Vec::new(),
        }
    }

    /// No readings at all, for drawing tests: no system scan, nothing that
    /// depends on the machine the tests run on.
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            system: System::new(),
            disks: Disks::new(),
            networks: Networks::new(),
            network_rate: (0.0, 0.0),
            last_tick: Instant::now(),
            prefs: Preferences::default(),
            prefs_path: None,
            status: None,
            page: None,
            mouse: None,
            hits: Vec::new(),
        }
    }

    fn refresh_interval(&self) -> Duration {
        Duration::from_secs_f64(self.prefs.refresh_interval)
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh(true);
        self.networks.refresh(true);
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        let (rx, tx) = self
            .networks
            .iter()
            .filter(|(interface, _)| !is_loopback(interface))
            .fold((0, 0), |(rx, tx), (_, data)| {
                (rx + data.received(), tx + data.transmitted())
            });
        self.network_rate = (per_second(rx, elapsed), per_second(tx, elapsed));
        self.last_tick = now;
    }

    /// The disks with their mount points, each mount point once.
    pub fn mounted_disks(&self) -> Vec<(&Path, &Disk)> {
        unique_mount_points(self.disks.iter().map(|disk| (disk.mount_point(), disk)))
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::TogglePreferences => {
                self.page = match self.page {
                    Some(_) => None,
                    None => Some(Page::default()),
                };
            }
            Message::Category(category) => {
                if let Some(page) = &mut self.page {
                    page.category = category;
                    page.focus = Focus::Categories;
                    page.row = 0;
                }
            }
            Message::Activate(index) => {
                let Some(page) = &mut self.page else { return };
                page.focus = Focus::Content;
                page.row = index;
                match focusable_rows(&self.prefs, page).into_iter().nth(index) {
                    Some(Row::Section { key, .. }) => {
                        if !page.expanded.remove(key) {
                            page.expanded.insert(key);
                        }
                    }
                    Some(Row::Setting { setting, .. }) => self.change(setting, 1),
                    _ => {}
                }
            }
        }
    }

    /// Applies a change at once and saves it: there is no Save button.
    fn change(&mut self, setting: Setting, by: isize) {
        self.prefs.step(setting, by);
        if setting == Setting::Language {
            i18n::set_locale(self.prefs.language.locale());
        }
        let Some(path) = &self.prefs_path else { return };
        if let Err(err) = preferences::save(path, &self.prefs) {
            let error = err.to_string();
            self.status = Some(tf("preferences_save_failed", &[("error", &error)]));
        }
    }

    /// Returns true when the key quits.
    fn on_key(&mut self, key: KeyEvent) -> bool {
        // Windows also reports key releases: without this, every key would
        // act twice.
        if key.kind != KeyEventKind::Press {
            return false;
        }
        if terminal::is_quit(&key) {
            return true;
        }
        self.status = None;
        let Some(page) = &mut self.page else {
            if key.code == KeyCode::Char('p') {
                self.update(Message::TogglePreferences);
            }
            return false;
        };
        let rows = focusable_rows(&self.prefs, page).len();
        match (key.code, page.focus) {
            (KeyCode::Char('p') | KeyCode::Esc, _) => self.update(Message::TogglePreferences),
            (KeyCode::Tab | KeyCode::BackTab, Focus::Content) => page.focus = Focus::Categories,
            (
                KeyCode::Tab | KeyCode::BackTab | KeyCode::Right | KeyCode::Enter,
                Focus::Categories,
            ) if rows > 0 => {
                page.focus = Focus::Content;
                page.row = page.row.min(rows - 1);
            }
            (KeyCode::Up, Focus::Categories) => {
                let category = page.category.saturating_sub(1);
                self.update(Message::Category(category));
            }
            (KeyCode::Down, Focus::Categories) => {
                let category = (page.category + 1).min(ui::preferences::CATEGORIES.len() - 1);
                self.update(Message::Category(category));
            }
            (KeyCode::Up, Focus::Content) => page.row = page.row.saturating_sub(1),
            (KeyCode::Down, Focus::Content) => {
                page.row = (page.row + 1).min(rows.saturating_sub(1))
            }
            (KeyCode::Enter | KeyCode::Char(' '), Focus::Content) => {
                let row = page.row;
                self.update(Message::Activate(row));
            }
            (KeyCode::Left | KeyCode::Right, Focus::Content) => {
                let by = if key.code == KeyCode::Left { -1 } else { 1 };
                match focusable_rows(&self.prefs, page).into_iter().nth(page.row) {
                    Some(Row::Setting { setting, .. }) => self.change(setting, by),
                    // Right opens a section, Left closes it.
                    Some(Row::Section { key, .. }) if by > 0 => {
                        page.expanded.insert(key);
                    }
                    Some(Row::Section { key, .. }) => {
                        page.expanded.remove(key);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        let position = Position::new(mouse.column, mouse.row);
        self.mouse = Some(position);
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        self.status = None;
        let clicked = self.hits.iter().find(|hit| hit.area.contains(position));
        if let Some(&Hit { message, .. }) = clicked {
            self.update(message);
        }
    }
}

/// The rows of the open category that take focus, in order: what
/// [`Message::Activate`] and the focused row count.
fn focusable_rows(prefs: &Preferences, page: &Page) -> Vec<Row> {
    ui::preferences::rows(prefs, page)
        .into_iter()
        .filter(Row::takes_focus)
        .collect()
}

/// Takes over the terminal and runs until a quit key or a quit signal. The
/// caller restores the terminal, whatever this returns. `prefs_path` is where
/// changes are saved, and `status` what to tell the user about the file.
pub fn run_app(
    quit_signal: &AtomicUsize,
    prefs: Preferences,
    prefs_path: Option<PathBuf>,
    status: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut terminal = terminal::enable()?;
    let mut app = App::new(prefs, prefs_path, status);

    // Checked after every poll, which waits one refresh interval at most.
    while quit_signal.load(Ordering::SeqCst) == 0 {
        // Only on the tick: any other event (a held key, a key release on
        // Windows, a resize, the mouse moving) just redraws. Refreshing on
        // each one rescanned every process many times a second and sampled
        // CPU usage over intervals too short to mean anything.
        if app.last_tick.elapsed() >= app.refresh_interval() {
            app.refresh();
        }
        let mut hits = Vec::new();
        terminal.draw(|frame| hits = ui::render_ui(frame, &app))?;
        app.hits = hits;

        let timeout = app
            .refresh_interval()
            .saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if app.on_key(key) => break,
                Event::Mouse(mouse) => app.on_mouse(mouse),
                _ => {}
            }
        }
    }

    Ok(())
}

/// `bytes` moved over `elapsed`, per second. sysinfo's `received()` and
/// `transmitted()` already count from the previous refresh, and drawing takes
/// time too, so the interval is measured rather than assumed.
fn per_second(bytes: u64, elapsed: Duration) -> f64 {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 { bytes as f64 / secs } else { 0.0 }
}

/// Loopback traffic never leaves the machine, so it stays out of the rates.
/// sysinfo 0.39 has no loopback flag, so this goes by name: `lo` on Linux,
/// `lo0` on macOS, `Loopback Pseudo-Interface 1` on Windows (where sysinfo
/// already leaves it out, as it has no hardware address).
fn is_loopback(interface: &str) -> bool {
    interface == "lo" || interface == "lo0" || interface.starts_with("Loopback")
}

/// Each mount point once, in the order the system lists them. A mount stacked
/// on another is listed twice, and both rows would show the same space, the
/// top mount's.
fn unique_mount_points<'a, T>(
    items: impl IntoIterator<Item = (&'a Path, T)>,
) -> Vec<(&'a Path, T)> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|&(mount_point, _)| seen.insert(mount_point))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::Language;

    #[test]
    fn loopback_is_recognised_on_every_system() {
        assert!(is_loopback("lo"));
        assert!(is_loopback("lo0"));
        assert!(is_loopback("Loopback Pseudo-Interface 1"));
        assert!(!is_loopback("eth0"));
        assert!(!is_loopback("en0"));
        assert!(!is_loopback("wlo1"));
        assert!(!is_loopback("Ethernet"));
    }

    #[test]
    fn mount_points_are_listed_once_in_order() {
        let mounts = [("/", 1), ("/home", 2), ("/", 3), ("/boot", 4)];
        let unique = unique_mount_points(mounts.map(|(path, n)| (Path::new(path), n)));
        assert_eq!(
            unique,
            [
                (Path::new("/"), 1),
                (Path::new("/home"), 2),
                (Path::new("/boot"), 4)
            ]
        );
    }

    fn press(app: &mut App, code: KeyCode) -> bool {
        app.on_key(KeyEvent::new(code, crossterm::event::KeyModifiers::NONE))
    }

    #[test]
    fn the_page_is_driven_by_the_keyboard() {
        let mut app = App::empty();
        assert!(!press(&mut app, KeyCode::Char('p')));
        let page = app.page.as_ref().expect("p opens Preferences");
        assert_eq!((page.category, page.focus), (0, Focus::Categories));

        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Down);
        assert_eq!(app.page.as_ref().unwrap().category, 2);
        press(&mut app, KeyCode::Up);
        press(&mut app, KeyCode::Up);
        press(&mut app, KeyCode::Up);
        assert_eq!(app.page.as_ref().unwrap().category, 0, "stops at General");

        // Into General's sections: open Language, move to its setting and
        // change it, in both directions.
        i18n::with_locale(i18n::Locale::En, || {
            press(&mut app, KeyCode::Tab);
            press(&mut app, KeyCode::Enter);
            assert!(app.page.as_ref().unwrap().expanded.contains("language"));
            press(&mut app, KeyCode::Down);
            press(&mut app, KeyCode::Right);
            assert_eq!(app.prefs.language, Language::En);
            press(&mut app, KeyCode::Right);
            assert_eq!(app.prefs.language, Language::Fr);
            assert_eq!(i18n::locale(), i18n::Locale::Fr, "applies at once");
            press(&mut app, KeyCode::Left);
            press(&mut app, KeyCode::Left);
            assert_eq!(app.prefs.language, Language::System);
        });

        assert!(!press(&mut app, KeyCode::Esc));
        assert!(app.page.is_none(), "Esc closes");
        press(&mut app, KeyCode::Char('p'));
        let page = app.page.as_ref().unwrap();
        assert!(page.expanded.is_empty(), "sections arrive closed again");
        assert!(press(&mut app, KeyCode::Char('q')), "q still quits");
    }

    #[test]
    fn a_click_does_what_its_hit_says() {
        let mut app = App::empty();
        app.hits = vec![Hit {
            area: ratatui::layout::Rect::new(1, 0, 11, 1),
            message: Message::TogglePreferences,
        }];
        let click = |column, row| MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        app.on_mouse(click(30, 0));
        assert!(app.page.is_none(), "outside the button");
        app.on_mouse(click(5, 0));
        assert!(app.page.is_some());
        app.on_mouse(click(5, 0));
        assert!(app.page.is_none());
    }

    #[test]
    fn network_rate_is_per_second() {
        assert_eq!(per_second(2048, Duration::from_millis(500)), 4096.0);
        assert_eq!(per_second(2048, Duration::ZERO), 0.0);
    }
}

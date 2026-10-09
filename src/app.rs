//! The readings and the loop that refreshes and draws them.

use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use sysinfo::{Disk, Disks, Networks, ProcessesToUpdate, System};

use crate::{terminal, ui};

const TICK_RATE: Duration = Duration::from_millis(1000);

pub struct App {
    pub system: System,
    disks: Disks,
    networks: Networks,
    /// Bytes received and transmitted per second, all interfaces except
    /// loopback together.
    pub network_rate: (f64, f64),
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
}

/// Takes over the terminal and runs until a quit key or a quit signal. The
/// caller restores the terminal, whatever this returns.
pub fn run_app(quit_signal: &AtomicUsize) -> Result<(), Box<dyn Error>> {
    let mut terminal = terminal::enable()?;
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
        terminal.draw(|frame| ui::render_ui(frame, &app))?;

        let timeout = TICK_RATE.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && terminal::is_quit(&key)
        {
            break;
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

    #[test]
    fn network_rate_is_per_second() {
        assert_eq!(per_second(2048, Duration::from_millis(500)), 4096.0);
        assert_eq!(per_second(2048, Duration::ZERO), 0.0);
    }
}

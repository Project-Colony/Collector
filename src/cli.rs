//! The command line: parsed by hand, before the terminal is touched, so
//! `--version` and `--help` work with no TTY at all (the release smoke test
//! runs `--version` headless).

use std::ffi::OsString;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = "\
Usage: collector [OPTION]

A terminal system monitor: CPU, memory, disks, network and processes.

Options:
  -h, --help     Print this help and exit
  -V, --version  Print the version and exit

Keys: q or Ctrl+C quits.
";

pub enum Mode {
    Run,
    Help,
    Version,
}

/// Returns the offending argument on error.
pub fn parse_args(args: &[OsString]) -> Result<Mode, String> {
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

pub fn print_version() -> ExitCode {
    println!("collector {VERSION}");
    ExitCode::SUCCESS
}

pub fn print_help() -> ExitCode {
    print!("collector {VERSION}\n\n{USAGE}");
    ExitCode::SUCCESS
}

pub fn print_unexpected(arg: &str) -> ExitCode {
    eprint!("collector: unexpected argument '{arg}'\n\n{USAGE}");
    ExitCode::from(2)
}

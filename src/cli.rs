//! The command line: parsed by hand, before the terminal is touched, so
//! `--version` and `--help` work with no TTY at all (the release smoke test
//! runs `--version` headless).

use std::ffi::OsString;
use std::process::ExitCode;

use crate::i18n::{t, tf};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Exactly `collector X.Y.Z`, in every locale: the release workflow's smoke
/// test and the signed `.meta` of each asset are checked against it.
pub fn print_version() -> ExitCode {
    println!("collector {VERSION}");
    ExitCode::SUCCESS
}

pub fn print_help() -> ExitCode {
    print!("collector {VERSION}\n\n{}", t("usage"));
    ExitCode::SUCCESS
}

pub fn print_unexpected(arg: &str) -> ExitCode {
    let message = tf("unexpected_argument", &[("arg", arg)]);
    eprint!("{message}\n\n{}", t("usage"));
    ExitCode::from(2)
}

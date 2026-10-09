mod app;
mod cli;
mod terminal;
mod ui;

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use cli::Mode;

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    match cli::parse_args(&args) {
        Ok(Mode::Run) => {}
        Ok(Mode::Version) => return cli::print_version(),
        Ok(Mode::Help) => return cli::print_help(),
        Err(arg) => return cli::print_unexpected(&arg),
    }

    if !terminal::is_interactive() {
        eprintln!("Collector needs a terminal (with ssh, use ssh -t)");
        return ExitCode::from(1);
    }
    terminal::install_panic_hook();
    let quit_signal = match terminal::quit_signal() {
        Ok(flag) => flag,
        Err(err) => {
            eprintln!("collector: {err}");
            return ExitCode::FAILURE;
        }
    };

    let result = app::run_app(&quit_signal);
    let restored = terminal::restore();
    let result = result.and(restored.map_err(Into::into));
    if let Err(err) = &result {
        eprintln!("collector: {err}");
    }

    if let Some(code) = terminal::die_of_signal(&quit_signal) {
        return code;
    }
    if result.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

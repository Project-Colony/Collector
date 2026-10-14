mod app;
mod cli;
mod i18n;
mod paths;
mod preferences;
mod terminal;
mod theme;
mod ui;

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use cli::Mode;
use i18n::{t, tf};

fn main() -> ExitCode {
    // Read first, so --help speaks the chosen language too, but only read:
    // a file that fails to load is set aside further down, by the monitor,
    // which can tell the user.
    let prefs_path = preferences::path();
    let loaded = prefs_path.as_deref().map_or_else(
        |_| Ok(preferences::Preferences::default()),
        preferences::load,
    );
    let prefs = loaded.as_ref().cloned().unwrap_or_default();
    i18n::set_locale(prefs.language.locale());

    let args: Vec<OsString> = env::args_os().skip(1).collect();
    match cli::parse_args(&args) {
        Ok(Mode::Run) => {}
        Ok(Mode::Version) => return cli::print_version(),
        Ok(Mode::Help) => return cli::print_help(),
        Err(arg) => return cli::print_unexpected(&arg),
    }

    if !terminal::is_interactive() {
        eprintln!("{}", t("needs_terminal"));
        return ExitCode::from(1);
    }
    terminal::install_panic_hook();
    let quit_signal = match terminal::quit_signal() {
        Ok(flag) => flag,
        Err(err) => {
            eprintln!("{}", tf("error", &[("error", &err.to_string())]));
            return ExitCode::FAILURE;
        }
    };

    let (save_to, status) = preferences::save_target(prefs_path, &loaded);
    let result = app::run_app(&quit_signal, prefs, save_to, status);
    let restored = terminal::restore();
    let result = result.and(restored.map_err(Into::into));
    if let Err(err) = &result {
        eprintln!("{}", tf("error", &[("error", &err.to_string())]));
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

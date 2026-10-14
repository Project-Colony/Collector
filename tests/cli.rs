//! The command line works with no terminal at all: the release workflow's smoke
//! test runs `--version` headless, and anything else without a terminal must
//! say so instead of drawing into a pipe.
//!
//! The messages follow the system language, so these checks look for what
//! reads the same in English and French: option names, the argument, `ssh -t`.

#[cfg(target_os = "linux")]
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn collector(args: &[&str]) -> Output {
    collector_with_env(args, &[])
}

fn collector_with_env(args: &[&str], env: &[(&str, &str)]) -> Output {
    collector_in(&empty_config_home(), args, env)
}

/// Runs Collector with `config_home` as its config root on Linux and its home
/// on macOS, so the preferences of whoever runs the tests stay out of them.
/// Windows reads its folders from the system, not the environment; its CI
/// runners have no preferences file.
fn collector_in(config_home: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_collector"))
        .args(args)
        .env("XDG_CONFIG_HOME", config_home)
        .env("HOME", config_home)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .expect("run collector")
}

/// A config root with nothing in it. It need not exist: Collector only reads.
fn empty_config_home() -> PathBuf {
    std::env::temp_dir().join(format!("collector-cli-empty-{}", std::process::id()))
}

#[test]
fn version_and_help_need_no_terminal() {
    let version = format!("collector {}", env!("CARGO_PKG_VERSION"));
    for flag in ["-V", "--version", "-h", "--help"] {
        let out = collector(&[flag]);
        assert!(out.status.success(), "{flag}: {:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.starts_with(&version), "{flag}: {stdout}");
    }
    assert!(String::from_utf8_lossy(&collector(&["--help"]).stdout).contains("--version"));
}

#[test]
fn unknown_argument_prints_usage_and_exits_2() {
    let out = collector(&["--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--nope") && stderr.contains("--version"),
        "{stderr}"
    );
}

#[test]
fn without_a_terminal_it_says_so_and_exits_1() {
    let out = collector(&[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ssh -t"), "{stderr}");
}

/// `--version` is the one string that never changes with the language.
#[test]
fn version_is_the_same_in_french() {
    let out = collector_with_env(
        &["--version"],
        &[("LANGUAGE", "fr"), ("LC_ALL", "fr_FR.UTF-8")],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, format!("collector {}\n", env!("CARGO_PKG_VERSION")));
}

/// Linux takes the language from the environment (Windows and macOS from the
/// system settings), so this is where both locales can be checked end to end.
#[cfg(target_os = "linux")]
#[test]
fn messages_follow_the_language_on_linux() {
    let english = [("LANGUAGE", "en"), ("LC_ALL", "C")];
    let french = [("LANGUAGE", "fr"), ("LC_ALL", "fr_FR.UTF-8")];
    let stderr = |env| String::from_utf8_lossy(&collector_with_env(&[], env).stderr).into_owned();
    assert!(
        stderr(&english).contains("Collector needs a terminal (with ssh, use ssh -t)"),
        "{}",
        stderr(&english)
    );
    assert!(
        stderr(&french).contains("Collector a besoin d'un terminal (avec ssh, utilisez ssh -t)"),
        "{}",
        stderr(&french)
    );
    let help =
        String::from_utf8_lossy(&collector_with_env(&["--help"], &french).stdout).into_owned();
    assert!(help.contains("Utilisation : collector"), "{help}");
}

/// A config root holding `preferences` as Collector's preferences file.
#[cfg(target_os = "linux")]
fn config_home_with(name: &str, preferences: &str) -> (PathBuf, PathBuf) {
    let home = std::env::temp_dir().join(format!("collector-cli-{name}-{}", std::process::id()));
    let file = home.join("Colony/Collector/preferences/preferences.json");
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, preferences).unwrap();
    (home, file)
}

/// The language chosen in Preferences wins over the system's, from the very
/// first message.
#[cfg(target_os = "linux")]
#[test]
fn the_language_preference_overrides_the_system() {
    let (home, _) = config_home_with("language", r#"{ "language": "fr" }"#);
    let out = collector_in(&home, &["--help"], &[("LANGUAGE", "en"), ("LC_ALL", "C")]);
    let help = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(help.contains("Utilisation : collector"), "{help}");
    let _ = fs::remove_dir_all(&home);
}

/// Only the monitor sets a broken file aside, where it can say so: the command
/// line reads it, falls back to the defaults and leaves it as it is.
#[cfg(target_os = "linux")]
#[test]
fn the_command_line_never_touches_the_preferences_file() {
    let (home, file) = config_home_with("untouched", "{ not json");
    let out = collector_in(&home, &["--help"], &[("LANGUAGE", "en"), ("LC_ALL", "C")]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Usage: collector"));
    assert_eq!(fs::read_to_string(&file).unwrap(), "{ not json");
    assert!(!file.with_extension("json.invalid").exists());
    let _ = fs::remove_dir_all(&home);
}

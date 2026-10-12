//! The command line works with no terminal at all: the release workflow's smoke
//! test runs `--version` headless, and anything else without a terminal must
//! say so instead of drawing into a pipe.
//!
//! The messages follow the system language, so these checks look for what
//! reads the same in English and French: option names, the argument, `ssh -t`.

use std::process::{Command, Output, Stdio};

fn collector(args: &[&str]) -> Output {
    collector_with_env(args, &[])
}

fn collector_with_env(args: &[&str], env: &[(&str, &str)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_collector"))
        .args(args)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .expect("run collector")
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

//! The command line works with no terminal at all: the release workflow's smoke
//! test runs `--version` headless, and anything else without a terminal must
//! say so instead of drawing into a pipe.

use std::process::{Command, Output, Stdio};

fn collector(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_collector"))
        .args(args)
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
    assert!(String::from_utf8_lossy(&collector(&["--help"]).stdout).contains("Usage:"));
}

#[test]
fn unknown_argument_prints_usage_and_exits_2() {
    let out = collector(&["--nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'--nope'") && stderr.contains("Usage:"),
        "{stderr}"
    );
}

#[test]
fn without_a_terminal_it_says_so_and_exits_1() {
    let out = collector(&[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Collector needs a terminal (with ssh, use ssh -t)"),
        "{stderr}"
    );
}

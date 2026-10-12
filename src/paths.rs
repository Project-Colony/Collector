//! Where Collector keeps its files: the config row of the Colony table
//! (design/filesystem.md in Project-Colony-Resources).
//!
//! | Linux | Windows | macOS |
//! |---|---|---|
//! | `~/.config/Colony/Collector/` | `%LOCALAPPDATA%\Colony\Collector\` | `~/Library/Application Support/Colony/Collector/` |
//!
//! A port of `colony_ui::paths`, not a call to it: colony-ui is built on iced,
//! and depending on it would pull a GUI toolkit into a terminal program, far
//! past Collector's crate and size budgets. The rules are the same: the
//! platform's local config directory (never the roaming one on Windows), the
//! `Colony` vendor directory, then the program, and a program name that is a
//! single safe path component other than the reserved `apps`.
//!
//! Collector keeps preferences and nothing else, so only the config row is
//! ported. It has no data to keep and nothing to cache, and creates neither
//! directory.

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

/// The organisation directory every Colony program nests under.
const VENDOR: &str = "Colony";

/// `<data>/Colony/apps/` is the launcher's shared install root, so no program
/// directory may take that name.
const APPS: &str = "apps";

/// The program's directory name, capitalised as the program spells it.
pub const PROGRAM: &str = "Collector";

/// Collector's config directory, `<config>/Colony/Collector/`, without
/// creating it: saving the preferences creates what it needs.
pub fn config_dir() -> io::Result<PathBuf> {
    config_dir_of(PROGRAM)
}

fn config_dir_of(program: &str) -> io::Result<PathBuf> {
    let root = dirs::config_local_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cannot determine the platform config directory",
        )
    })?;
    Ok(root.join(VENDOR).join(program_component(program)?))
}

/// Rejects anything that is not a single, safe path component, as
/// `colony_ui::paths` does: `..` or a separator would leave the Colony tree,
/// and Windows strips a trailing dot or space, so `Grape.` would be `Grape`.
fn component(name: &str) -> io::Result<&str> {
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name.contains(['/', '\\', '\0'])
        || Path::new(name).components().count() != 1
        || Path::new(name).file_name() != Some(OsStr::new(name));
    if invalid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name:?} is not a safe single path component"),
        ));
    }
    Ok(name)
}

/// [`component`], plus `apps` in any case: Windows and default macOS
/// filesystems ignore case, and upper-casing first also catches a letter that
/// folds to an ASCII one, such as the long s in `appſ`.
fn program_component(name: &str) -> io::Result<&str> {
    let name = component(name)?;
    if name.to_uppercase().eq_ignore_ascii_case(APPS) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name:?} is reserved for the shared install root and cannot name a program"),
        ));
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_dir_nests_under_the_vendor_then_the_program() {
        let path = config_dir().unwrap();
        let tail: Vec<_> = path
            .components()
            .rev()
            .take(2)
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        assert_eq!(tail, ["Collector", "Colony"], "{path:?}");
    }

    #[test]
    fn a_program_name_cannot_escape_the_colony_tree() {
        for bad in [
            "..",
            ".",
            "",
            "../../etc",
            "a/b",
            "a\\b",
            "with\0nul",
            "Grape.",
            "Grape ",
        ] {
            let err = config_dir_of(bad).expect_err(bad);
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{bad:?}");
        }
    }

    #[test]
    fn the_install_root_name_is_reserved_in_any_case() {
        for reserved in ["apps", "Apps", "APPS", "apps.", "apps ", "app\u{17F}"] {
            let err = config_dir_of(reserved).expect_err(reserved);
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{reserved:?}");
        }
        for good in ["Colony", "Collector", "SAM - Colony Edition"] {
            assert!(config_dir_of(good).is_ok(), "{good:?}");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_uses_the_xdg_config_directory() {
        // XDG_CONFIG_HOME may be set; only assert the default shape when not.
        if std::env::var_os("XDG_CONFIG_HOME").is_none() {
            let home = std::env::var("HOME").expect("HOME");
            assert_eq!(
                config_dir().unwrap(),
                PathBuf::from(home).join(".config/Colony/Collector")
            );
        }
    }

    /// Never AppData\Roaming: `dirs::config_dir()` would be.
    #[cfg(windows)]
    #[test]
    fn windows_uses_local_app_data() {
        let local = std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA");
        assert_eq!(
            config_dir().unwrap(),
            PathBuf::from(local).join("Colony").join("Collector")
        );
    }
}

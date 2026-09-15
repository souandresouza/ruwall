pub const __VERSION__: &str = "3.3.1";
pub const __CACHE_VERSION__: &str = "1.1.0";

use std::path::PathBuf;

use include_dir::{include_dir, Dir};

pub const ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

/// "Darwin", "Windows" or "Linux", mirroring python's platform.uname()[0].
pub fn os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "Darwin",
        "windows" => "Windows",
        _ => "Linux",
    }
}

fn env_or(name: &str, default: String) -> String {
    std::env::var(name).unwrap_or(default)
}

pub fn home_dir() -> PathBuf {
    if std::env::consts::OS == "windows" {
        env_or("USERPROFILE", String::new())
    } else {
        env_or("HOME", String::new())
    }
    .into()
}

pub fn cache_dir() -> PathBuf {
    if std::env::consts::OS == "windows" {
        if let Ok(c) = std::env::var("APPDATA") {
            return PathBuf::from(c).join("wal");
        }
        return home_dir().join(".cache").join("wal");
    }

    let xdg_cache = env_or("XDG_CACHE_HOME", home_dir().join(".cache").to_string_lossy().into_owned());
    PathBuf::from(env_or("PYWAL_CACHE_DIR", PathBuf::from(xdg_cache).join("wal").to_string_lossy().into_owned()))
}

pub fn conf_dir() -> PathBuf {
    if std::env::consts::OS == "windows" {
        if let Ok(c) = std::env::var("APPDATA") {
            return PathBuf::from(c).join("wal");
        }
        return home_dir().join(".config").join("wal");
    }

    let xdg_conf = env_or("XDG_CONFIG_HOME", home_dir().join(".config").to_string_lossy().into_owned());
    PathBuf::from(xdg_conf).join("wal")
}
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::sync::OnceLock;

/// Serialises in-process tests that mutate process-global env / statics.
pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

/// Route cache+config into a shared temp dir for this test binary and return
/// its path, isolating in-process tests from ~/.cache and ~/.config.
///
/// The dir is fixed per-process so concurrently running tests keep agreeing on
/// the process-global PYWAL_CACHE_DIR/XDG_CONFIG_HOME env vars.
pub fn isolated_env() -> std::path::PathBuf {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let _guard = env_lock();
        let base = std::env::temp_dir().join(format!("wal-test-{}", std::process::id()));
        let cache = base.join("cache");
        let conf = base.join("conf");
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::create_dir_all(&cache);
        let _ = std::fs::create_dir_all(&conf);
        std::env::set_var("PYWAL_CACHE_DIR", &cache);
        std::env::set_var("XDG_CONFIG_HOME", &conf);
        base
    })
    .join("cache")
}

/// Path to the compiled `ruwall` binary.
pub fn wal_bin() -> &'static str {
    &env!("CARGO_BIN_EXE_ruwall")[..]
}

/// Deterministic identifier for a set of CLI args (used to build dirs).
fn cli_id(args: &[&str]) -> String {
    args.join(" ").replace(['/', ' ', '"'], "_")
}

fn cli_dirs(args: &[&str]) -> (std::path::PathBuf, std::path::PathBuf) {
    let id = cli_id(args);
    let cache = std::env::temp_dir().join(format!("wal-cli-{}-{}", std::process::id(), id));
    let conf = std::env::temp_dir().join(format!("wal-conf-{}-{}", std::process::id(), id));
    let _ = std::fs::remove_dir_all(&cache);
    let _ = std::fs::remove_dir_all(&conf);
    let _ = std::fs::create_dir_all(&cache);
    let _ = std::fs::create_dir_all(&conf);
    (cache, conf)
}

/// Cache dir used by a CLI invocation with these args.
pub fn cli_cache_dir(args: &[&str]) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "wal-cli-{}-{}",
        std::process::id(),
        cli_id(args)
    ))
}

/// Config dir used by a CLI invocation with these args.
pub fn cli_conf_dir(args: &[&str]) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "wal-conf-{}-{}",
        std::process::id(),
        cli_id(args)
    ))
}

/// Run the wal binary with isolated cache/config dirs.
pub fn run_wal(args: &[&str]) -> std::process::Output {
    let (cache, conf) = cli_dirs(args);
    run_wal_in(args, &cache, &conf)
}

/// Run the wal binary against explicit cache/config dirs.
pub fn run_wal_in(
    args: &[&str],
    cache: &std::path::Path,
    conf: &std::path::Path,
) -> std::process::Output {
    let _ = std::fs::create_dir_all(cache);
    let _ = std::fs::create_dir_all(conf);

    std::process::Command::new(wal_bin())
        .args(args)
        .env("PYWAL_CACHE_DIR", cache)
        .env("XDG_CONFIG_HOME", conf)
        .env("HOME", std::env::temp_dir())
        .output()
        .expect("failed to spawn wal binary")
}

pub fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
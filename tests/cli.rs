use std::path::Path;

use serde_json::Value;

fn run_wal(args: &[&str]) -> std::process::Output {
    common::run_wal(args)
}

fn colors_json(output_dir: &Path) -> Value {
    let text = std::fs::read_to_string(output_dir.join("colors.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn test_version() {
    let out = run_wal(&["-v"]);
    assert!(out.status.success());
    assert!(common::stdout(&out).contains("ruwall 3.3.1"));
}

#[test]
fn test_list_backends() {
    let out = run_wal(&["--backend"]);
    assert!(out.status.success());
    assert!(common::stdout(&out).contains("wal"));
}

#[test]
fn test_list_themes() {
    let out = run_wal(&["--theme"]);
    assert!(out.status.success());
    let text = common::stdout(&out);
    assert!(text.contains("Dark Themes"));
    assert!(text.contains("base16-monokai"));
}

#[test]
fn test_theme_file_output() {
    let out = run_wal(&["-f", "base16-monokai", "-n", "-e", "-q"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let cache = common::cli_cache_dir(&["-f", "base16-monokai", "-n", "-e", "-q"]);
    let json = colors_json(&cache);
    assert_eq!(json["special"]["background"], "#272822");
    assert_eq!(json["colors"]["color0"], "#272822");
    assert_eq!(json["colors"]["color15"], "#f9f8f5");
    assert_eq!(json["alpha"], "100");
    assert!(cache.join("sequences").is_file());
    assert!(cache.join("colors.sh").is_file());
}

#[test]
fn test_alpha() {
    let out = run_wal(&["-f", "base16-monokai", "-a", "85", "-n", "-e", "-q"]);
    assert!(out.status.success());

    let cache = common::cli_cache_dir(&["-f", "base16-monokai", "-a", "85", "-n", "-e", "-q"]);
    let json = colors_json(&cache);
    assert_eq!(json["alpha"], "85");
}

#[test]
fn test_background() {
    let out = run_wal(&["-f", "base16-monokai", "-b", "000000", "-n", "-e", "-q"]);
    assert!(out.status.success());

    let cache = common::cli_cache_dir(&["-f", "base16-monokai", "-b", "000000", "-n", "-e", "-q"]);
    let json = colors_json(&cache);
    assert_eq!(json["special"]["background"], "#000000");
    assert_eq!(json["colors"]["color0"], "#000000");
}

#[test]
fn test_image_generation() {
    let out = run_wal(&["-i", "tests/test_files/test.png", "-n", "-e", "-q"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let cache = common::cli_cache_dir(&["-i", "tests/test_files/test.png", "-n", "-e", "-q"]);
    let json = colors_json(&cache);
    for i in 0..16 {
        let color = json["colors"][format!("color{}", i)].as_str().unwrap();
        assert_eq!(color.len(), 7, "color{} = {}", i, color);
        assert!(color.starts_with('#'));
    }
    assert_eq!(
        json["wallpaper"],
        format!("{}/tests/test_files/test.png", std::env::current_dir().unwrap().display())
    );
}

#[test]
fn test_light_theme() {
    let out = run_wal(&["-i", "tests/test_files/test.png", "-l", "-n", "-e", "-q"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let cache = common::cli_cache_dir(&["-i", "tests/test_files/test.png", "-l", "-n", "-e", "-q"]);
    let json = colors_json(&cache);
    assert_eq!(json["colors"]["color0"].as_str().unwrap().len(), 7);
}

#[test]
fn test_save_theme() {
    let out = run_wal(&["-f", "base16-monokai", "-p", "clitest", "-n", "-e", "-q"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let conf = common::cli_conf_dir(&["-f", "base16-monokai", "-p", "clitest", "-n", "-e", "-q"]);
    let path = conf.join("wal/colorschemes/dark/clitest.json");
    assert!(path.is_file(), "missing {}", path.display());
}

#[test]
fn test_clear_cache() {
    // Generate from an image first so schemes/ exists in the cache.
    let gen = run_wal(&["-i", "tests/test_files/test.png", "-n", "-e", "-q"]);
    assert!(gen.status.success());
    let cache = common::cli_cache_dir(&["-i", "tests/test_files/test.png", "-n", "-e", "-q"]);
    assert!(cache.join("schemes").exists());

    let conf = common::cli_conf_dir(&["-i", "tests/test_files/test.png", "-n", "-e", "-q"]);
    let out = common::run_wal_in(&["-c"], &cache, &conf);
    assert!(out.status.success());
    assert!(!cache.join("schemes").exists());
}

#[test]
fn test_cached_wallpaper() {
    let gen_args = &["-i", "tests/test_files/test.jpg", "-n", "-e", "-q"];
    let gen = run_wal(gen_args);
    assert!(gen.status.success());
    let cache = common::cli_cache_dir(gen_args);
    assert!(cache.join("wal").is_file());

    let conf = common::cli_conf_dir(gen_args);
    let out = common::run_wal_in(&["-w", "-n", "-e", "-q"], &cache, &conf);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

    let json = colors_json(&cache);
    for i in 0..16 {
        assert_eq!(json["colors"][format!("color{}", i)].as_str().unwrap().len(), 7);
    }
}

#[test]
fn test_invalid_image_fails() {
    let out = run_wal(&["-i", "nope.jpg", "-n", "-e", "-q"]);
    assert!(!out.status.success());
}

#[test]
fn test_dir_without_images_fails() {
    let out = run_wal(&["-i", "tests", "-n", "-e", "-q"]);
    assert!(!out.status.success());
}

#[test]
fn test_conflicting_args() {
    let out = run_wal(&["-i", "tests/test_files/test.jpg", "-f", "base16-monokai"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(common::stdout(&out).contains("Conflicting") || String::from_utf8_lossy(&out.stderr).contains("Conflicting"));
}

#[test]
fn test_no_input() {
    let out = run_wal(&[]);
    assert!(!out.status.success());
}

mod common;
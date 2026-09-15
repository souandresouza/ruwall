use std::path::Path;

use ruwall::theme::{Theme, ThemeColors, Special};
use ruwall::util;

#[test]
fn test_color_import() {
    common::isolated_env();
    let theme = Theme::from_value(
        util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap(),
    );
    assert_eq!(theme.colors.color0, "#1F211E");
    assert_eq!(theme.colors.color15, "#F5F1F4");
    assert_eq!(theme.special.background, "#1F211E");
    assert_eq!(theme.special.foreground, "#F5F1F4");
    assert_eq!(theme.wallpaper, "5.png");
    assert_eq!(theme.alpha, "100");
}

#[test]
fn test_color_import_no_wallpaper_and_alpha() {
    common::isolated_env();
    let theme = Theme::from_value(
        util::read_file_json(Path::new("tests/test_files/test_file2.json")).unwrap(),
    );
    assert_eq!(theme.wallpaper, "None");
    assert_eq!(theme.alpha, "100");
}

#[test]
fn test_theme_file_fallback() {
    common::isolated_env();
    // Missing wallpaper/alpha default to "None"/"100", missing colors to "".
    let value = serde_json::json!({
        "colors": { "color3": "#00aabb" },
    });
    let mut theme = Theme::from_value(value);
    assert_eq!(theme.wallpaper, "None");
    assert_eq!(theme.alpha, "100");
    assert_eq!(theme.colors.get(3), "#00aabb");
    assert_eq!(theme.colors.get(0), "");
    theme.wallpaper = String::from("None");
    assert_eq!(theme.wallpaper, "None");
}

#[test]
fn test_theme_colors_index() {
    let mut colors = ThemeColors::default();
    colors.set(5, "#123456");
    assert_eq!(colors.get(5), "#123456");
    assert_eq!(colors.get(15), "");
    colors.set(15, "#654321");
    assert_eq!(colors.get(15), "#654321");
    assert_eq!(colors.as_vec().len(), 16);
}

#[test]
fn test_special_defaults() {
    let special = Special { ..Default::default() };
    assert_eq!(special.background, "");
}

mod common;
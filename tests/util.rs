use std::path::Path;

use ruwall::util;

#[test]
fn test_read_file() {
    let lines = util::read_file(Path::new("tests/test_files/test_file")).unwrap();
    assert_eq!(lines[0], "/home/dylan/Pictures/Wallpapers/1.jpg");
}

#[test]
fn test_read_file_json_start() {
    let value = util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap();
    assert_eq!(value["colors"]["color0"], "#1F211E");
}

#[test]
fn test_read_file_json_end() {
    let value = util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap();
    assert_eq!(value["colors"]["color15"], "#F5F1F4");
}

#[test]
fn test_read_wallpaper() {
    let value = util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap();
    assert_eq!(value["wallpaper"], "5.png");
}

#[test]
fn test_save_file() {
    let tmp = common::isolated_env().join("test_file");
    util::save_file("Hello, world", &tmp);
    assert!(tmp.is_file());
}

#[test]
fn test_save_file_json() {
    let cache = common::isolated_env();
    let value = util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap();
    util::save_file_json(&value, &cache.join("test_out.json")).unwrap();
    assert!(cache.join("test_out.json").is_file());
}

#[test]
fn test_create_dir() {
    let tmp = common::isolated_env().join("nested/dir");
    util::create_dir(&tmp).unwrap();
    assert!(tmp.is_dir());
}

#[test]
fn test_hex_to_rgb_black() {
    assert_eq!(util::hex_to_rgb("#000000"), [0, 0, 0]);
}

#[test]
fn test_hex_to_rgb_white() {
    assert_eq!(util::hex_to_rgb("#ffffff"), [255, 255, 255]);
}

#[test]
fn test_hex_to_rgb_rand() {
    assert_eq!(util::hex_to_rgb("#98aec2"), [152, 174, 194]);
}

#[test]
fn test_hex_to_xrgba() {
    assert_eq!(util::hex_to_xrgba("#98aec2"), "98/ae/c2/ff");
}

#[test]
fn test_rgb_to_hex() {
    assert_eq!(util::rgb_to_hex([152, 174, 194]), "#98aec2");
}

#[test]
fn test_darken_color() {
    assert_eq!(util::darken_color("#ffffff", 0.25), "#bfbfbf");
}

#[test]
fn test_lighten_color() {
    assert_eq!(util::lighten_color("#000000", 0.25), "#3f3f3f");
}

mod common;
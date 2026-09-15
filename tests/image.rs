use std::path::Path;

use ruwall::image;

#[test]
fn test_get_img() {
    common::isolated_env();
    let result = image::get("tests/test_files/test.jpg", false, false);
    assert!(result.contains("tests/test_files/test.jpg"));
}

#[test]
fn test_get_img_dir() {
    common::isolated_env();
    let result = image::get("tests/test_files", false, false);
    assert!(result.ends_with(".jpg") || result.ends_with(".png"));
}

#[test]
fn test_get_img_iterative() {
    let cache = common::isolated_env();
    // Clear the cached wallpaper so "next" starts from the first image.
    let _ = std::fs::remove_file(cache.join("wal"));
    // Sorted natural order: test.jpg < test.png
    let result = image::get("tests/test_files", true, false);
    assert!(result.contains("test.jpg"), "got: {}", result);
}

#[test]
fn test_get_img_recursive() {
    common::isolated_env();
    let root = Path::new("tests/test_files");
    let result = image::get(root.to_str().unwrap(), false, true);
    assert!(result.ends_with(".jpg") || result.ends_with(".png"));
}

#[test]
fn test_abspath() {
    assert_eq!(
        image::abspath("tests/test_files/test.jpg"),
        format!(
            "{}/tests/test_files/test.jpg",
            std::env::current_dir().unwrap().display()
        )
    );
    assert_eq!(image::abspath("/tmp/foo/../bar"), "/tmp/bar");
}

mod common;
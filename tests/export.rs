use std::path::Path;

use ruwall::export;
use ruwall::theme::Theme;
use ruwall::util;

fn test_theme() -> Theme {
    let value = util::read_file_json(Path::new("tests/test_files/test_file.json")).unwrap();
    Theme::from_value(value)
}

#[test]
fn test_all_templates() {
    let cache = common::isolated_env();
    let theme = test_theme();
    export::every(&theme, &cache);

    let sh = cache.join("colors.sh");
    assert!(sh.is_file());
    let lines = util::read_file(&sh).unwrap();
    assert_eq!(lines[6], "foreground='#F5F1F4'");
}

#[test]
fn test_css_template() {
    let cache = common::isolated_env();
    let theme = test_theme();
    let tmp = cache.join("test.css");
    export::color(&theme, "css", Some(&tmp));

    assert!(tmp.is_file());
    let lines = util::read_file(&tmp).unwrap();
    assert_eq!(lines[6], "    --background: #1F211E;");
}

#[test]
fn test_color_alpha_template() {
    let cache = common::isolated_env();
    util::set_alpha_num("85");
    let theme = test_theme();
    let tmp = cache.join("colors.Xresources");
    export::color(&theme, "xresources", Some(&tmp));

    assert!(tmp.is_file());
    let content = util::read_file(&tmp).unwrap().join("\n");
    assert!(content.contains("[85]#1F211E"), "got: {}", content);
}

#[test]
fn test_get_export_type() {
    assert_eq!(export::get_export_type("css"), "colors.css");
    assert_eq!(export::get_export_type("shell"), "colors.sh");
    assert_eq!(export::get_export_type("xresources"), "colors.Xresources");
}

mod common;
use ruwall::sequences;
use ruwall::theme::Theme;
use ruwall::util;

fn bg() -> &'static str {
    "#1F211E"
}

#[test]
fn test_set_special() {
    assert_eq!(
        sequences::set_special(11, bg(), "h", "100"),
        "\x1b]11;#1F211E\x1b\\"
    );
}

#[test]
fn test_set_special_alpha() {
    assert_eq!(
        sequences::set_special(11, bg(), "h", "99"),
        "\x1b]11;[99]#1F211E\x1b\\"
    );
}

#[test]
fn test_set_special_bg_alpha() {
    assert_eq!(
        sequences::set_special(708, bg(), "", "99"),
        "\x1b]708;[99]#1F211E\x1b\\"
    );
}

#[test]
fn test_set_color() {
    assert_eq!(sequences::set_color(11, bg()), "\x1b]4;11;#1F211E\x1b\\");
}

#[test]
fn test_set_iterm_tab_color() {
    assert_eq!(sequences::set_iterm_tab_color(bg()).len(), 84);
}

#[test]
fn test_create_sequences_std() {
    let value = util::read_file_json(std::path::Path::new("tests/test_files/test_file.json")).unwrap();
    let theme = Theme::from_value(value);
    let sequences = sequences::create_sequences(&theme, false);

    assert!(sequences.contains("\x1b]4;0;#1F211E\x1b\\"));
    assert!(sequences.contains("\x1b]11;#1F211E\x1b\\"));
    assert!(sequences.contains("\x1b]10;#F5F1F4\x1b\\"));
    assert!(sequences.contains("\x1b]708;#1F211E\x1b\\"));
}

#[test]
fn test_create_sequences_vte() {
    let value = util::read_file_json(std::path::Path::new("tests/test_files/test_file.json")).unwrap();
    let theme = Theme::from_value(value);
    let sequences = sequences::create_sequences(&theme, true);

    assert!(!sequences.contains("708"));
}
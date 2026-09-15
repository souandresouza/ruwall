use std::path::Path;

use crate::settings::os_name;
use crate::theme::Theme;
use crate::util;

const __MODULE__: &str = "sequences";

fn is_darwin() -> bool {
    os_name() == "Darwin"
}

pub fn set_special(index: u32, color: &str, iterm_name: &str, alpha: &str) -> String {
    if is_darwin() && !iterm_name.is_empty() {
        return format!(
            "\x1b]P{}{}\x1b\\",
            iterm_name,
            color.trim_start_matches('#')
        );
    }

    if matches!(index, 11 | 708) && alpha != "100" {
        return format!("\x1b]{};[{}]{}\x1b\\", index, alpha, color);
    }

    format!("\x1b]{};{}\x1b\\", index, color)
}

pub fn set_color(index: u32, color: &str) -> String {
    if is_darwin() && index < 20 {
        return format!("\x1b]P{:x}{}\x1b\\", index, color.trim_start_matches('#'));
    }

    format!("\x1b]4;{};{}\x1b\\", index, color)
}

pub fn set_iterm_tab_color(color: &str) -> String {
    let rgb = util::hex_to_rgb(color);
    format!(
        "\x1b]6;1;bg;red;brightness;{}\x07\x1b]6;1;bg;green;brightness;{}\x07\x1b]6;1;bg;blue;brightness;{}\x07",
        rgb[0], rgb[1], rgb[2]
    )
}

pub fn create_sequences(colors: &Theme, vte_fix: bool) -> String {
    let alpha = &colors.alpha;

    let mut sequences: Vec<String> = Vec::new();

    for index in 0..16 {
        sequences.push(set_color(index as u32, colors.colors.get(index)));
    }

    sequences.extend([
        set_special(10, &colors.special.foreground, "g", "100"),
        set_special(11, &colors.special.background, "h", alpha),
        set_special(12, &colors.special.cursor, "l", "100"),
        set_special(13, &colors.special.foreground, "j", "100"),
        set_special(17, &colors.special.foreground, "k", "100"),
        set_special(19, &colors.special.background, "m", "100"),
        set_color(232, &colors.special.background),
        set_color(256, &colors.special.foreground),
        set_color(257, &colors.special.background),
    ]);

    if !vte_fix {
        sequences.push(set_special(708, &colors.special.background, "", alpha));
    }

    if is_darwin() {
        sequences.push(set_iterm_tab_color(&colors.special.background));
    }

    sequences.join("")
}

pub fn send(colors: &Theme, cache_dir: &Path, to_send: bool, vte_fix: bool) {
    let tty_pattern = if is_darwin() {
        "/dev/ttys00[0-9]*"
    } else {
        "/dev/pts/[0-9]*"
    };

    let sequences = create_sequences(colors, vte_fix);

    if to_send {
        for term in util::glob(tty_pattern) {
            util::save_file(&sequences, &term);
        }
    }

    util::save_file(&sequences, &cache_dir.join("sequences"));
    util::log_info(__MODULE__, "Set terminal colors.");
}
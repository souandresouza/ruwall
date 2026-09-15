use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::settings::{ASSETS, cache_dir, conf_dir};
use crate::theme::Theme;
use crate::util;

const __MODULE__: &str = "export";

/// Parse a percent value the way pywal's Color methods do:
/// keep digits and dots, then divide by 100.
fn parse_percent(percent: &str) -> f64 {
    let cleaned: String = percent
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let num: f64 = cleaned.parse().unwrap_or(0.0);
    num / 100.0
}

/// Evaluate a color function/attribute. Returns None for unknown names.
fn color_value(name: &str, color: &str, args: Option<&str>) -> Option<String> {
    match (name, args) {
        ("lighten", Some(args)) => Some(util::lighten_color(color, parse_percent(args))),
        ("darken", Some(args)) => Some(util::darken_color(color, parse_percent(args))),
        ("saturate", Some(args)) => Some(util::saturate_color(color, parse_percent(args))),
        ("strip", None) => Some(color.trim_start_matches('#').to_string()),
        ("rgb", None) => {
            let [r, g, b] = util::hex_to_rgb(color);
            Some(format!("{},{},{}", r, g, b))
        }
        ("xrgba", None) => Some(util::hex_to_xrgba(color)),
        ("alpha", None) => Some(format!("[{}]{}", util::alpha_num(), color)),
        ("alpha_dec", None) => {
            let d = util::alpha_dec();
            if d.fract() == 0.0 {
                Some(format!("{:.1}", d))
            } else {
                Some(format!("{}", d))
            }
        }
        ("decimal", None) => {
            let value = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
            Some(format!("#{}", value))
        }
        ("decimal_strip", None) => {
            let value = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
            Some(format!("{}", value))
        }
        ("octal", None) => {
            let value = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
            Some(format!("#{:o}", value))
        }
        ("octal_strip", None) => {
            let value = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
            Some(format!("{:o}", value))
        }
        ("red", None) => {
            let [r, _, _] = util::hex_to_rgb(color);
            Some(format!("{:.3}", r as f64 / 255.0))
        }
        ("green", None) => {
            let [_, g, _] = util::hex_to_rgb(color);
            Some(format!("{:.3}", g as f64 / 255.0))
        }
        ("blue", None) => {
            let [_, _, b] = util::hex_to_rgb(color);
            Some(format!("{:.3}", b as f64 / 255.0))
        }
        ("hex_color", None) => Some(color.to_string()),
        // Methods without a call, or attributes with arguments, are invalid.
        ("lighten" | "darken" | "saturate", None) => None,
        _ => None,
    }
}

/// Scan a line for a `{token}` (brace adjacent, not doubled) and return its
/// byte range plus the token content.
fn first_token(line: &str, from: usize) -> Option<(usize, usize, String)> {
    let bytes = line.as_bytes();
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'{' && (i == 0 || bytes[i - 1] != b'{') {
            let mut j = i + 1;
            let mut content = String::new();
            let mut found = false;
            while j < bytes.len() {
                match bytes[j] {
                    b'{' => break,
                    b'}' => {
                        if j + 1 < bytes.len() && bytes[j + 1] == b'}' {
                            break;
                        }
                        found = true;
                        break;
                    }
                    c => content.push(c as char),
                }
                j += 1;
            }
            if found {
                return Some((i, j, content));
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    None
}

/// Process one `{token}` and update the colors map with any generated color.
/// Returns Some((replace_str, replacement)) when the token is a color
/// function/attribute, None when it isn't (left for the format pass).
fn process_token(content: &str, colors: &mut HashMap<String, String>) -> Option<(String, String)> {
    let (cname, funcs) = match content.find('.') {
        Some(i) => (&content[..i], &content[i + 1..]),
        None => (content, ""),
    };
    if cname.is_empty() {
        return None;
    }
    if funcs.is_empty() {
        return None;
    }

    let base = match colors.get(cname) {
        Some(value) => value.clone(),
        None => {
            util::log_error(__MODULE__, &format!("Key '{}' not found in template.", cname));
            return None;
        }
    };

    let mut new_value: Option<String> = Some(base);
    let mut replace_str = format!("{}", cname);

    for piece in funcs.split(|c| c == ')' || c == '.') {
        if piece.is_empty() {
            continue;
        }

        let (name, args) = match piece.split_once('(') {
            Some((name, args)) => (name, Some(args)),
            None => (piece, None),
        };
        let name = name.strip_prefix('.').unwrap_or(name);

        let current = new_value.as_ref()?;

        if matches!(name, "lighten" | "darken" | "saturate") {
            let result = color_value(name, current, args);
            if result.is_none() {
                util::log_error(
                    __MODULE__,
                    &format!("Syntax error in template: '{}'.", content),
                );
                return None;
            }
            replace_str.push('.');
            replace_str.push_str(name);
            if args.is_some() {
                replace_str.push('(');
                replace_str.push_str(args.unwrap_or(""));
                replace_str.push(')');
            }
            new_value = result;
        } else {
            let result = color_value(name, current, args);
            if result.is_none() {
                util::log_error(
                    __MODULE__,
                    &format!("Syntax error in template: '{}'.", content),
                );
                return None;
            }
            replace_str.push('.');
            replace_str.push_str(name);
            new_value = result;
        }
    }

    let final_value = new_value?;
    let new_color_clean = final_value.replace(['[', ']', '.'], "_");
    let replacement = format!("color{}", new_color_clean);

    colors.insert(replacement.clone(), final_value);
    Some((replace_str, replacement))
}

/// Render template content: apply color functions then substitute `{key}`.
fn render_template(input_content: &str, colors: &mut HashMap<String, String>) -> Option<String> {
    let lines = input_content.split_inclusive('\n');
    let mut processed: Vec<String> = Vec::new();

    for line in lines {
        let mut line = line.to_string();
        let mut pos = 0;
        loop {
            match first_token(&line, pos) {
                Some((_start, end, content)) => {
                    if !content.contains('.') {
                        pos = end + 1;
                        continue;
                    }
                    match process_token(&content, colors) {
                        Some((replace_str, repl)) => {
                            line = line.replace(&replace_str, &repl);
                            pos = 0;
                        }
                        None => {
                            pos = end + 1;
                        }
                    }
                }
                None => break,
            }
        }
        processed.push(line);
    }

    let joined = processed.concat();
    match format_template(&joined, colors) {
        Ok(result) => Some(result),
        Err(err) => {
            util::log_error(
                __MODULE__,
                &format!("Syntax error in template file: {}.", err),
            );
            None
        }
    }
}

/// Python str.format()-ish substitution: `{{`/`}}` escapes and plain `{key}`.
fn format_template(line: &str, colors: &HashMap<String, String>) -> Result<String, String> {
    let bytes = line.as_bytes();
    let mut out = String::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    out.push('{');
                    i += 2;
                    continue;
                }
                let mut name = String::new();
                i += 1;
                loop {
                    if i >= bytes.len() {
                        return Err("unmatched '{'".to_string());
                    }
                    match bytes[i] {
                        b'}' => {
                            i += 1;
                            break;
                        }
                        b'{' => return Err("nested format field".to_string()),
                        b':' | b'!' => return Err("format spec unsupported".to_string()),
                        c => name.push(c as char),
                    }
                    i += 1;
                }
                match colors.get(&name) {
                    Some(value) => out.push_str(value),
                    None => return Err(format!("Key '{}' not found", name)),
                }
            }
            b'}' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                    out.push('}');
                    i += 2;
                    continue;
                }
                return Err("unmatched '}'".to_string());
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }

    Ok(out)
}

fn flatten_colors(colors: &Theme) -> HashMap<String, String> {
    let mut all = HashMap::new();
    all.insert("wallpaper".to_string(), colors.wallpaper.clone());
    all.insert("alpha".to_string(), colors.alpha.clone());
    all.insert("background".to_string(), colors.special.background.clone());
    all.insert("foreground".to_string(), colors.special.foreground.clone());
    all.insert("cursor".to_string(), colors.special.cursor.clone());
    for i in 0..16 {
        all.insert(format!("color{}", i), colors.colors.get(i).to_string());
    }
    all
}

/// Process a template read from disk and save the result.
pub fn template(colors: &Theme, input_file: &Path, output_file: Option<&Path>) {
    let input_content = match std::fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(_) => return,
    };
    let mut colors_map = flatten_colors(colors);
    if let Some(result) = render_template(&input_content, &mut colors_map) {
        if let Some(output) = output_file {
            util::save_file(&result, output);
        }
    }
}

pub fn get_export_type(export_type: &str) -> String {
    let map = [
        ("css", "colors.css"),
        ("dmenu", "colors-wal-dmenu.h"),
        ("dwm", "colors-wal-dwm.h"),
        ("st", "colors-wal-st.h"),
        ("tabbed", "colors-wal-tabbed.h"),
        ("gtk2", "colors-gtk2.rc"),
        ("json", "colors.json"),
        ("konsole", "colors-konsole.colorscheme"),
        ("kitty", "colors-kitty.conf"),
        ("nqq", "colors-nqq.css"),
        ("plain", "colors"),
        ("putty", "colors-putty.reg"),
        ("rofi", "colors-rofi.Xresources"),
        ("scss", "colors.scss"),
        ("shell", "colors.sh"),
        ("speedcrunch", "colors-speedcrunch.json"),
        ("sway", "colors-sway"),
        ("tty", "colors-tty.sh"),
        ("vscode", "colors-vscode.json"),
        ("waybar", "colors-waybar.css"),
        ("xresources", "colors.Xresources"),
        ("xmonad", "colors.hs"),
        ("yaml", "colors.yml"),
    ];
    match map.iter().find(|(key, _)| *key == export_type) {
        Some((_, name)) => name.to_string(),
        None => export_type.to_string(),
    }
}

fn template_files() -> Vec<(String, &'static [u8])> {
    let mut files = Vec::new();
    if let Some(templates) = ASSETS.get_dir("templates") {
        for file in templates.files() {
            let name = file
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            files.push((name, file.contents()));
        }
    }
    files
}

fn user_template_files() -> Vec<PathBuf> {
    let dir = conf_dir().join("templates");
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == ".DS_Store" || name.ends_with(".swp") {
                continue;
            }
            paths.push(entry.path());
        }
    }
    paths
}

pub fn every(colors: &Theme, output_dir: &Path) {
    util::create_dir(&conf_dir().join("templates")).ok();

    for (name, contents) in template_files() {
        if name == ".DS_Store" || name.ends_with(".swp") {
            continue;
        }
        let input_content = String::from_utf8_lossy(contents).to_string();
        let output = output_dir.join(name);
        let mut colors_map = flatten_colors(colors);
        if let Some(result) = render_template(&input_content, &mut colors_map) {
            util::save_file(&result, &output);
        }
    }

    for path in user_template_files() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let input_content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let output = output_dir.join(name);
        let mut colors_map = flatten_colors(colors);
        if let Some(result) = render_template(&input_content, &mut colors_map) {
            util::save_file(&result, &output);
        }
    }

    util::log_info(__MODULE__, "Exported all files.");
    util::log_info(__MODULE__, "Exported all user files.");
}

pub fn color(colors: &Theme, export_type: &str, output_file: Option<&Path>) {
    let template_name = get_export_type(export_type);
    let output_file = match output_file {
        Some(path) => path.to_path_buf(),
        None => cache_dir().join(&template_name),
    };

    let mut contents: Option<&'static [u8]> = None;
    for (name, data) in template_files() {
        if name == template_name {
            contents = Some(data);
            break;
        }
    }

    match contents {
        Some(contents) => {
            let input_content = String::from_utf8_lossy(contents).to_string();
            let mut colors_map = flatten_colors(colors);
            if let Some(result) = render_template(&input_content, &mut colors_map) {
                util::save_file(&result, &output_file);
            }
            util::log_info(__MODULE__, &format!("Exported {}.", export_type));
        }
        None => {
            util::log_warn(__MODULE__, &format!("Template '{}' doesn't exist.", export_type));
        }
    }
}
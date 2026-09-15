use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

/// pywal's logging module name -> color tag shown in brackets.
const LEVEL_INFO: &str = "\x1b[1;32mI";
const LEVEL_WARN: &str = "\x1b[1;33mW";
const LEVEL_ERROR: &str = "\x1b[1;31mE";

static QUIET: AtomicBool = AtomicBool::new(false);
static ALPHA_NUM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::from("100")));

pub fn set_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::Relaxed);
}

pub fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

fn log(level: &str, module: &str, msg: &str) {
    if is_quiet() {
        return;
    }
    let out = format!(
        "[{level}\x1b[0m] \x1b[1;31m{module}\x1b[0m: {msg}",
        level = level,
        module = module,
        msg = msg
    );
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{}", out);
}

pub fn log_info(module: &str, msg: &str) {
    log(LEVEL_INFO, module, msg);
}

pub fn log_warn(module: &str, msg: &str) {
    log(LEVEL_WARN, module, msg);
}

pub fn log_error(module: &str, msg: &str) {
    log(LEVEL_ERROR, module, msg);
}

pub fn set_alpha_num(alpha: &str) {
    *ALPHA_NUM.lock().unwrap() = alpha.to_string();
}

pub fn alpha_num() -> String {
    ALPHA_NUM.lock().unwrap().clone()
}

pub fn alpha_dec() -> f64 {
    let num: f64 = alpha_num().trim().parse().unwrap_or(100.0);
    num / 100.0
}

pub fn create_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

pub fn read_file(input_file: &Path) -> std::io::Result<Vec<String>> {
    let content = std::fs::read_to_string(input_file)?;
    Ok(content.lines().map(String::from).collect())
}

pub fn read_file_raw(input_file: &Path) -> std::io::Result<Vec<String>> {
    let content = std::fs::read_to_string(input_file)?;
    Ok(content
        .split_inclusive('\n')
        .map(String::from)
        .collect())
}

pub fn read_file_json(input_file: &Path) -> std::io::Result<serde_json::Value> {
    let content = std::fs::read_to_string(input_file)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save_file(data: &str, export_file: &Path) {
    if let Some(parent) = export_file.parent() {
        let _ = create_dir(parent);
    }

    if let Err(err) = std::fs::write(export_file, data) {
        if err.kind() == std::io::ErrorKind::PermissionDenied {
            log_warn("util", &format!("Couldn't write to {}.", export_file.display()));
        }
    }
}

pub fn save_file_json(data: &serde_json::Value, export_file: &Path) -> std::io::Result<()> {
    if let Some(parent) = export_file.parent() {
        let _ = create_dir(parent);
    }

    std::fs::write(export_file, serde_json::to_string_pretty(data)?)
}

pub fn hex_to_rgb(color: &str) -> [u8; 3] {
    let value = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
    [
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
    ]
}

pub fn hex_to_xrgba(color: &str) -> String {
    let col = color.to_lowercase();
    let col = col.trim_start_matches('#');
    let col = col.chars().take(6).collect::<String>();
    let mut out = String::with_capacity(11);
    for (i, c) in col.chars().enumerate() {
        out.push(c);
        if i % 2 == 1 {
            out.push('/');
        }
    }
    out.push_str("ff");
    out
}

pub fn rgb_to_hex(color: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2])
}

pub fn darken_color(color: &str, amount: f64) -> String {
    let rgb = hex_to_rgb(color);
    rgb_to_hex([
        (rgb[0] as f64 * (1.0 - amount)) as u8,
        (rgb[1] as f64 * (1.0 - amount)) as u8,
        (rgb[2] as f64 * (1.0 - amount)) as u8,
    ])
}

pub fn lighten_color(color: &str, amount: f64) -> String {
    let rgb = hex_to_rgb(color);
    rgb_to_hex([
        (rgb[0] as f64 + (255.0 - rgb[0] as f64) * amount) as u8,
        (rgb[1] as f64 + (255.0 - rgb[1] as f64) * amount) as u8,
        (rgb[2] as f64 + (255.0 - rgb[2] as f64) * amount) as u8,
    ])
}

pub fn blend_color(color: &str, color2: &str) -> String {
    let a = hex_to_rgb(color);
    let b = hex_to_rgb(color2);
    rgb_to_hex([
        (0.5 * a[0] as f64 + 0.5 * b[0] as f64) as u8,
        (0.5 * a[1] as f64 + 0.5 * b[1] as f64) as u8,
        (0.5 * a[2] as f64 + 0.5 * b[2] as f64) as u8,
    ])
}

/// colorsys.rgb_to_hls
fn rgb_to_hls(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let maxc = r.max(g).max(b);
    let minc = r.min(g).min(b);
    let l = (maxc + minc) / 2.0;
    let (mut h, s);
    if maxc == minc {
        h = 0.0;
        s = 0.0;
    } else {
        if l <= 0.5 {
            s = (maxc - minc) / (maxc + minc);
        } else {
            s = (maxc - minc) / (2.0 - maxc - minc);
        }
        let rc = (maxc - r) / (maxc - minc);
        let gc = (maxc - g) / (maxc - minc);
        let bc = (maxc - b) / (maxc - minc);
        if r == maxc {
            h = bc - gc;
        } else if g == maxc {
            h = 2.0 + rc - bc;
        } else {
            h = 4.0 + gc - rc;
        }
        h = (h / 6.0) % 1.0;
    }
    (h, l, s)
}

/// colorsys.hls_to_rgb
fn hls_to_rgb(h: f64, l: f64, s: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - (l * s)
    };
    let p = 2.0 * l - q;
    let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 0.5 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

pub fn saturate_color(color: &str, amount: f64) -> String {
    let rgb = hex_to_rgb(color);
    let (r, g, b) = (
        rgb[0] as f64 / 255.0,
        rgb[1] as f64 / 255.0,
        rgb[2] as f64 / 255.0,
    );
    let (h, l, _s) = rgb_to_hls(r, g, b);
    let (r, g, b) = hls_to_rgb(h, l, amount);
    rgb_to_hex([
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    ])
}

/// colorsys.rgb_to_yiq -> (Y, I, Q)
pub fn rgb_to_yiq(color: &str) -> (f64, f64, f64) {
    let rgb = hex_to_rgb(color);
    let (r, g, b) = (
        rgb[0] as f64,
        rgb[1] as f64,
        rgb[2] as f64,
    );
    (
        0.299 * r + 0.587 * g + 0.114 * b,
        0.596 * r - 0.274 * g - 0.322 * b,
        0.211 * r - 0.523 * g + 0.312 * b,
    )
}

pub fn sort_by_yiq(colors: &mut [String]) {
    colors.sort_by(|a, b| {
        let (a_y, a_i, a_q) = rgb_to_yiq(a);
        let (b_y, b_i, b_q) = rgb_to_yiq(b);
        a_y.partial_cmp(&b_y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a_i.partial_cmp(&b_i).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a_q.partial_cmp(&b_q).unwrap_or(std::cmp::Ordering::Equal))
    });
}

pub fn disown(cmd: &[&str]) {
    if cmd.is_empty() {
        return;
    }
    let mut c = Command::new(cmd[0]);
    c.args(&cmd[1..]);
    c.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Err(err) = c.spawn() {
        log_warn("util", &format!("Couldn't spawn '{}': {}", cmd[0], err));
    }
}

pub fn run(cmd: &[&str]) -> std::process::ExitStatus {
    if cmd.is_empty() {
        return Default::default();
    }
    let mut c = Command::new(cmd[0]);
    c.args(&cmd[1..]);
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    match c.status() {
        Ok(status) => status,
        Err(err) => {
            log_warn("util", &format!("Couldn't run '{}': {}", cmd[0], err));
            Default::default()
        }
    }
}

pub fn which(cmd: &str) -> bool {
    if Path::new(cmd).exists() && is_executable(Path::new(cmd)) {
        return true;
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let full = dir.join(cmd);
            if full.exists() && is_executable(&full) {
                return true;
            }
        }
    }
    false
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) => meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Check if a process is running by name using `pidof`.
pub fn get_pid(name: &str) -> bool {
    if !which("pidof") {
        return false;
    }
    let mut cmd = Command::new("pidof");
    if std::env::consts::OS != "macos" {
        cmd.arg("-s");
    }
    cmd.arg(name);
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Glob with '*' and '[0-9]' support.
pub fn glob(pattern: &str) -> Vec<PathBuf> {
    let pattern = Path::new(pattern);
    let parent = pattern.parent().unwrap_or(Path::new(""));
    let name = pattern
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut matches = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().into_owned();
            if glob_match(&name, &fname) {
                matches.push(entry.path());
            }
        }
    }
    matches.sort();
    matches
}

fn glob_match(pattern: &str, s: &str) -> bool {
    match_pattern(pattern.as_bytes(), s.as_bytes())
}

fn match_pattern(p: &[u8], s: &[u8]) -> bool {
    if p.is_empty() {
        return s.is_empty();
    }
    match p[0] {
        b'*' => {
            for i in 0..=s.len() {
                if match_pattern(&p[1..], &s[i..]) {
                    return true;
                }
            }
            false
        }
        b'[' => {
            if let Some(close) = p.iter().position(|&c| c == b']') {
                if close > 0 {
                    let class = &p[1..close];
                    let matched = if class[0] == b'^' || class[0] == b'!' {
                        !class_contains(&class[1..], s)
                    } else {
                        class_contains(class, s)
                    };
                    if matched && !s.is_empty() {
                        return match_pattern(&p[close + 1..], &s[1..]);
                    }
                    return false;
                }
            }
            !s.is_empty() && p[0] == s[0] && match_pattern(&p[1..], &s[1..])
        }
        _ => !s.is_empty() && p[0] == s[0] && match_pattern(&p[1..], &s[1..]),
    }
}

fn class_contains(class: &[u8], c: &[u8]) -> bool {
    if c.is_empty() {
        return false;
    }
    let ch = c[0];
    let mut i = 0;
    while i < class.len() {
        if i + 2 < class.len() && class[i + 1] == b'-' {
            if class[i] <= ch && ch <= class[i + 2] {
                return true;
            }
            i += 3;
        } else {
            if class[i] == ch {
                return true;
            }
            i += 1;
        }
    }
    false
}

pub fn normalize_img_path(img: &str) -> String {
    if std::env::consts::OS == "windows" {
        img.replace('\\', "/")
    } else {
        img.to_string()
    }
}
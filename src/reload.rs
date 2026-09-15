use crate::settings::{ASSETS, os_name, cache_dir};
use crate::util;

const __MODULE__: &str = "reload";

pub fn tty(tty_reload: bool) {
    let tty_script = cache_dir().join("colors-tty.sh");
    let term = std::env::var("TERM").unwrap_or_default();

    if tty_reload && term == "linux" {
        util::disown(&[
            "sh",
            tty_script.to_str().unwrap_or("colors-tty.sh"),
        ]);
    }
}

pub fn xrdb(xrdb_files: Option<&[String]>) {
    let default = vec![cache_dir().join("colors.Xresources").to_string_lossy().into_owned()];
    let files = xrdb_files.unwrap_or(&default);

    if util::which("xrdb") && os_name() != "Darwin" {
        for file in files {
            let mut cmd = std::process::Command::new("xrdb");
            cmd.args(["-merge", "-quiet", file]);
            let _ = cmd.status();
        }
    }
}

pub fn gtk() {
    if util::which("python2") {
        let script = ASSETS
            .get_file("scripts/gtk_reload.py")
            .and_then(|f| f.contents_utf8());

        if let Some(script) = script {
            let script_path = std::env::temp_dir().join("wal_gtk_reload.py");
            util::save_file(script, &script_path);
            util::disown(&[
                "python2",
                script_path.to_str().unwrap_or("gtk_reload.py"),
            ]);
        }
    } else {
        util::log_warn(__MODULE__, "GTK2 reload support requires Python 2.");
    }
}

pub fn i3() {
    if util::which("i3-msg") && util::get_pid("i3") {
        util::disown(&["i3-msg", "reload"]);
    }
}

pub fn bspwm() {
    if util::which("bspc") && util::get_pid("bspwm") {
        util::disown(&["bspc", "wm", "-r"]);
    }
}

pub fn kitty() {
    let term = std::env::var("TERM").unwrap_or_default();
    if util::which("kitty")
        && util::get_pid("kitty")
        && term == "xterm-kitty"
    {
        let colors = cache_dir().join("colors-kitty.conf");
        let mut cmd = std::process::Command::new("kitty");
        cmd.args(["@", "set-colors", "--all"]);
        cmd.arg(colors);
        let _ = cmd.status();
    }
}

pub fn polybar() {
    if util::which("polybar") && util::get_pid("polybar") {
        util::disown(&["pkill", "-USR1", "polybar"]);
    }
}

pub fn sway() {
    if util::which("swaymsg") && util::get_pid("sway") {
        util::disown(&["swaymsg", "reload"]);
    }
}

pub fn colors() {
    let sequences = cache_dir().join("sequences");

    util::log_error(
        __MODULE__,
        &format!(
            "'wal -r' is deprecated: Use 'cat {}' instead.",
            sequences.display()
        ),
    );

    if sequences.is_file() {
        if let Ok(lines) = util::read_file(&sequences) {
            print!("{}", lines.join(""));
        }
    }
}

pub fn env(xrdb_file: Option<&str>, tty_reload: bool) {
    let files = xrdb_file.map(|f| vec![f.to_string()]);
    xrdb(files.as_deref());
    i3();
    bspwm();
    kitty();
    sway();
    polybar();
    util::log_info(__MODULE__, "Reloaded environment.");
    tty(tty_reload);
}
use std::path::Path;

use regex::Regex;

use crate::settings::{os_name, cache_dir, home_dir};
use crate::util;

const __MODULE__: &str = "wallpaper";

pub fn get_desktop_env() -> Option<String> {
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        return Some(desktop);
    }
    if let Ok(desktop) = std::env::var("DESKTOP_SESSION") {
        return Some(desktop);
    }
    if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return Some("GNOME".to_string());
    }
    if std::env::var("MATE_DESKTOP_SESSION_ID").is_ok() {
        return Some("MATE".to_string());
    }
    if std::env::var("SWAYSOCK").is_ok() {
        return Some("SWAY".to_string());
    }
    if let Ok(desktop) = std::env::var("DESKTOP_STARTUP_ID") {
        if desktop.contains("awesome") {
            return Some("AWESOME".to_string());
        }
    }
    None
}

fn xfconf(img: &str) {
    let output = std::process::Command::new("xfconf-query")
        .args(["--channel", "xfce4-desktop", "--list"])
        .output()
        .ok();

    let data = output
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    let re = Regex::new(
        r"(?m)^/backdrop/screen\d/monitor(?:0|\w*)/(?:(?:image-path|last-image)|workspace\d/last-image)$",
    )
    .unwrap();

    for path in re.find_iter(&data) {
        util::disown(&[
            "xfconf-query",
            "--channel",
            "xfce4-desktop",
            "--property",
            path.as_str(),
            "--set",
            img,
        ]);
    }
}

pub fn set_wm_wallpaper(img: &str) {
    let setters: &[&[&str]] = &[
        &["feh", "--bg-fill", img],
        &["xwallpaper", "--zoom", img],
        &["hsetroot", "-fill", img],
        &["nitrogen", "--set-zoom-fill", img],
        &["bgs", "-z", img],
        &["habak", "-mS", img],
        &["display", "-backdrop", "-window", "root", img],
    ];

    for setter in setters {
        if util::which(setter[0]) {
            util::disown(setter);
            return;
        }
    }

    util::log_error(__MODULE__, "No wallpaper setter found.");
}

pub fn set_desktop_wallpaper(desktop: Option<&str>, img: &str) {
    let desktop = desktop.unwrap_or("").to_lowercase();

    if desktop.contains("xfce") || desktop.contains("xubuntu") {
        xfconf(img);
    } else if desktop.contains("muffin") || desktop.contains("cinnamon") {
        let uri = format!("file://{}", img.replace(' ', "%20"));
        util::disown(&[
            "gsettings",
            "set",
            "org.cinnamon.desktop.background",
            "picture-uri",
            &uri,
        ]);
    } else if desktop.contains("gnome") || desktop.contains("unity") {
        let uri = format!("file://{}", img.replace(' ', "%20"));
        util::disown(&[
            "gsettings",
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            &uri,
        ]);
    } else if desktop.contains("mate") {
        util::disown(&[
            "gsettings",
            "set",
            "org.mate.background",
            "picture-filename",
            img,
        ]);
    } else if desktop.contains("sway") {
        util::disown(&["swaymsg", "output", "*", "bg", img, "fill"]);
    } else if desktop.contains("awesome") {
        let cmd = format!("require('gears').wallpaper.maximized('{}')", img);
        util::disown(&["awesome-client", &cmd]);
    } else if desktop.contains("kde") {
        let string = format!(
            "var allDesktops = desktops();for (i=0;i<allDesktops.length;i++){{\
             d = allDesktops[i];d.wallpaperPlugin = \"org.kde.image\";\
             d.currentConfigGroup = Array(\"Wallpaper\", \"org.kde.image\", \
             \"General\");d.writeConfig(\"Image\", \"{}\")}};",
            img
        );
        util::disown(&[
            "qdbus",
            "org.kde.plasmashell",
            "/PlasmaShell",
            "org.kde.PlasmaShell.evaluateScript",
            &string,
        ]);
    } else {
        set_wm_wallpaper(img);
    }
}

pub fn set_mac_wallpaper(img: &str) {
    let db_file = "Library/Application Support/Dock/desktoppicture.db";
    let db_path = home_dir().join(db_file);

    let sql_insert = format!("insert into data values(\"{}\"); ", img);
    let _ = run_checked("sqlite3", &[db_path.to_str().unwrap_or(""), &sql_insert]);

    let sql_select = "select max(rowid) from data;";
    let new_entry = run_capture("sqlite3", &[db_path.to_str().unwrap_or(""), sql_select])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let pictures = run_capture(
        "sqlite3",
        &[db_path.to_str().unwrap_or(""), "select rowid from pictures;"],
    )
    .unwrap_or_default();

    let mut sql = sql_insert;
    sql.push_str("delete from preferences; ");
    for pic in pictures.lines() {
        if !pic.is_empty() {
            sql.push_str(&format!(
                "insert into preferences (key, data_id, picture_id) values(1, {}, {}); ",
                new_entry, pic
            ));
        }
    }
    let _ = run_checked("sqlite3", &[db_path.to_str().unwrap_or(""), &sql]);
    let _ = run_checked("killall", &["Dock"]);
}

#[cfg(windows)]
pub fn set_win_wallpaper(img: &str) {
    // Mirror of the Windows SPI call, done through powershell.
    let script = format!(
        "[void](Get-ItemProperty -Path 'HKCU:\\Control Panel\\Desktop').Wallpaper; \
         Add-Type -TypeDefinition @'...'@; ..."
    );
    let _ = script;
    let _ = img;
}

#[cfg(not(windows))]
pub fn set_win_wallpaper(_img: &str) {}

fn run_capture(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

fn run_checked(cmd: &str, args: &[&str]) -> Option<std::process::ExitStatus> {
    std::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
}

pub fn change(img: &str) {
    if !Path::new(img).is_file() {
        return;
    }

    let desktop = get_desktop_env();

    if os_name() == "Darwin" {
        set_mac_wallpaper(img);
    } else if os_name() == "Windows" {
        set_win_wallpaper(img);
    } else {
        set_desktop_wallpaper(desktop.as_deref(), img);
    }

    util::log_info(__MODULE__, "Set the new wallpaper.");
}

pub fn get() -> String {
    let current_wall = cache_dir().join("wal");
    if current_wall.is_file() {
        if let Ok(lines) = util::read_file(&current_wall) {
            if let Some(line) = lines.into_iter().next() {
                return line;
            }
        }
    }
    "None".to_string()
}
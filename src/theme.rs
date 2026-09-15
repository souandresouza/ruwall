use std::fs;
use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::settings::{ASSETS, conf_dir};
use crate::util;

const __MODULE__: &str = "theme";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Special {
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub foreground: String,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ThemeColors {
    #[serde(default)] pub color0: String,
    #[serde(default)] pub color1: String,
    #[serde(default)] pub color2: String,
    #[serde(default)] pub color3: String,
    #[serde(default)] pub color4: String,
    #[serde(default)] pub color5: String,
    #[serde(default)] pub color6: String,
    #[serde(default)] pub color7: String,
    #[serde(default)] pub color8: String,
    #[serde(default)] pub color9: String,
    #[serde(default)] pub color10: String,
    #[serde(default)] pub color11: String,
    #[serde(default)] pub color12: String,
    #[serde(default)] pub color13: String,
    #[serde(default)] pub color14: String,
    #[serde(default)] pub color15: String,
}

impl ThemeColors {
    pub fn get(&self, index: usize) -> &str {
        match index {
            0 => &self.color0,
            1 => &self.color1,
            2 => &self.color2,
            3 => &self.color3,
            4 => &self.color4,
            5 => &self.color5,
            6 => &self.color6,
            7 => &self.color7,
            8 => &self.color8,
            9 => &self.color9,
            10 => &self.color10,
            11 => &self.color11,
            12 => &self.color12,
            13 => &self.color13,
            14 => &self.color14,
            _ => &self.color15,
        }
    }

    pub fn set(&mut self, index: usize, value: impl Into<String>) {
        let value = value.into();
        match index {
            0 => self.color0 = value,
            1 => self.color1 = value,
            2 => self.color2 = value,
            3 => self.color3 = value,
            4 => self.color4 = value,
            5 => self.color5 = value,
            6 => self.color6 = value,
            7 => self.color7 = value,
            8 => self.color8 = value,
            9 => self.color9 = value,
            10 => self.color10 = value,
            11 => self.color11 = value,
            12 => self.color12 = value,
            13 => self.color13 = value,
            14 => self.color14 = value,
            _ => self.color15 = value,
        }
    }

    pub fn as_vec(&self) -> Vec<String> {
        (0..16).map(|i| self.get(i).to_string()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    #[serde(default = "default_wallpaper")]
    pub wallpaper: String,
    #[serde(default = "default_alpha")]
    pub alpha: String,
    #[serde(default)]
    pub special: Special,
    #[serde(default)]
    pub colors: ThemeColors,
}

fn default_wallpaper() -> String {
    "None".to_string()
}

fn default_alpha() -> String {
    crate::util::alpha_num()
}

impl Theme {
    pub fn from_value(mut value: Value) -> Self {
        if value.get("wallpaper").is_none() {
            value["wallpaper"] = Value::String("None".to_string());
        }
        if value.get("alpha").is_none() {
            value["alpha"] = Value::String(crate::util::alpha_num());
        }
        if value.get("color").is_some() {
            terminal_sexy_to_wal(&mut value);
        }
        serde_json::from_value(value).expect("invalid theme file")
    }

    pub fn from_palette(colors: &[String], img: &str) -> Self {
        let mut pairs = ThemeColors::default();
        for i in 0..16 {
            pairs.set(i, colors.get(i).cloned().unwrap_or_default());
        }
        let special = Special {
            background: colors.get(0).cloned().unwrap_or_default(),
            foreground: colors.get(15).cloned().unwrap_or_default(),
            cursor: colors.get(15).cloned().unwrap_or_default(),
        };
        Theme {
            wallpaper: crate::util::normalize_img_path(img),
            alpha: crate::util::alpha_num(),
            special,
            colors: pairs,
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("theme serialization failed")
    }
}

fn terminal_sexy_to_wal(data: &mut Value) {
    let obj = match data.as_object_mut() {
        Some(obj) => obj,
        None => return,
    };

    let foreground = obj.get("foreground").cloned();
    let background = obj.get("background").cloned();
    let color = obj.get("color").cloned();

    let mut colors = Map::new();
    if let Some(color) = color {
        if let Some(arr) = color.as_array() {
            for (i, c) in arr.iter().enumerate() {
                colors.insert(format!("color{}", i), c.clone());
            }
        }
    }

    let cursor = obj
        .get("color")
        .and_then(|c| c.get(9).cloned())
        .unwrap_or(Value::String(String::new()));

    let mut special = Map::new();
    if let Some(fg) = foreground {
        special.insert("foreground".to_string(), fg);
    }
    if let Some(bg) = background {
        special.insert("background".to_string(), bg);
    }
    special.insert("cursor".to_string(), cursor);

    obj.insert("colors".to_string(), Value::Object(colors));
    obj.insert("special".to_string(), Value::Object(special));
}

fn embedded_theme_path(bri: &str, name: &str) -> Option<&'static include_dir::File<'static>> {
    ASSETS.get_file(format!("colorschemes/{}/{}", bri, name))
}

pub fn list_themes(dark: bool) -> Vec<String> {
    let bri = if dark { "dark" } else { "light" };
    let mut names = Vec::new();
    if let Some(dir) = ASSETS.get_dir(format!("colorschemes/{}", bri)) {
        for file in dir.files() {
            if let Some(name) = file.path().file_name() {
                names.push(name.to_string_lossy().into_owned());
            }
        }
    }
    names
}

pub fn list_themes_user() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for bri in ["dark", "light"] {
        let dir = conf_dir().join("colorschemes").join(bri);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    paths.push(entry.path());
                }
            }
        }
    }
    paths
}

pub fn get_random_theme(dark: bool) -> String {
    let mut themes = list_themes(dark);
    themes.shuffle(&mut rand::thread_rng());
    themes.into_iter().next().unwrap_or_default()
}

pub fn get_random_theme_user() -> String {
    let mut themes: Vec<String> = list_themes_user()
        .iter()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .collect();
    themes.shuffle(&mut rand::thread_rng());
    themes.into_iter().next().unwrap_or_default()
}

pub fn file(input_file: &str, light: bool) -> Theme {
    util::create_dir(&conf_dir().join("colorschemes/light")).ok();
    util::create_dir(&conf_dir().join("colorschemes/dark")).ok();

    let bri = if light { "light" } else { "dark" };

    let theme_name = Path::new(input_file)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let theme_name = format!("{}.json", theme_name);
    let user_theme_file = conf_dir().join("colorschemes").join(bri).join(&theme_name);

    enum Source {
        Builtin(String),
        Disk(PathBuf),
    }

    let selection: Source = match input_file {
        "random" | "random_dark" => Source::Builtin(get_random_theme(true)),
        "random_light" => Source::Builtin(get_random_theme(false)),
        "random_user" => Source::Disk(PathBuf::from(get_random_theme_user())),
        _ => {
            if user_theme_file.is_file() {
                Source::Disk(user_theme_file)
            } else if Path::new(input_file).is_file() {
                Source::Disk(PathBuf::from(input_file))
            } else {
                Source::Builtin(theme_name)
            }
        }
    };

    let (exists, content, basename): (bool, Option<String>, String) = match selection {
        Source::Disk(path) => {
            let exists = path.is_file();
            let content = if exists {
                fs::read_to_string(&path).ok()
            } else {
                None
            };
            let basename = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            (exists, content, basename)
        }
        Source::Builtin(name) => {
            let found = embedded_theme_path(bri, &name);
            let basename = name.split('/').last().unwrap_or(&name).to_string();
            match found {
                Some(file) => (true, file.contents_utf8().map(String::from), basename),
                None => (false, None, basename),
            }
        }
    };

    if exists {
        util::log_info(
            __MODULE__,
            &format!("Set theme to \x1b[1;37m{}\x1b[0m.", basename),
        );
        util::save_file(
            &basename,
            &crate::settings::cache_dir().join("last_used_theme"),
        );

        let content = content.unwrap_or_default();
        let value: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
        return Theme::from_value(value);
    }

    util::log_error(__MODULE__, &format!("No {} colorscheme file found.", bri));
    util::log_error(__MODULE__, "Try adding   '-l' to set light themes.");
    util::log_error(__MODULE__, "Try removing '-l' to set dark themes.");
    std::process::exit(1);
}

pub fn save(colors: &Theme, theme_name: &str, light: bool) {
    let bri = if light { "light" } else { "dark" };
    let theme_file = format!("{}.json", theme_name);
    let theme_path = conf_dir().join("colorschemes").join(bri).join(theme_file);
    if let Err(err) = util::save_file_json(&colors.to_value(), &theme_path) {
        util::log_warn(__MODULE__, &format!("Couldn't save theme: {}", err));
    }
}

pub fn list_out() {
    let dark_themes = list_themes(true);
    let light_themes = list_themes(false);
    let user_themes: Vec<String> = list_themes_user()
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().replace(".json", ""))
        .collect();

    let last_used_theme = util::read_file(&crate::settings::cache_dir().join("last_used_theme"))
        .ok()
        .and_then(|l| l.into_iter().next())
        .map(|t| t.replace(".json", ""))
        .unwrap_or_default();

    let dark_themes: Vec<String> = dark_themes
        .into_iter()
        .map(|t| t.replace(".json", ""))
        .collect();
    let light_themes: Vec<String> = light_themes
        .into_iter()
        .map(|t| t.replace(".json", ""))
        .collect();

    let render = |name: &str, list: Vec<String>| {
        let mut list = list;
        list.sort();
        let body: Vec<String> = list
            .into_iter()
            .map(|t| {
                if t == last_used_theme {
                    format!("{} (last used)", t)
                } else {
                    t
                }
            })
            .collect();
        println!("\x1b[1;32m{}\x1b[0m:", name);
        print!(" - ");
        println!("{}", body.join("\n - "));
    };

    if !user_themes.is_empty() {
        render("User Themes", user_themes);
    }
    render("Dark Themes", dark_themes);
    render("Light Themes", light_themes);

    println!("\x1b[1;32mExtra\x1b[0m:");
    println!(" - random (select a random dark theme)");
    println!(" - random_dark (select a random dark theme)");
    println!(" - random_light (select a random light theme)");
    println!(" - random_user (select a random user theme)");
}
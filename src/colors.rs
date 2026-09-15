use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;

use crate::backend;
use crate::settings::{__CACHE_VERSION__, cache_dir};
use crate::theme::Theme;
use crate::util;

const __MODULE__: &str = "colors";

/// The color backends shipped with this port.
pub fn list_backends() -> Vec<String> {
    vec!["wal".to_string()]
}

pub fn get_backend(backend: &str) -> String {
    if backend == "random" {
        let mut backends = list_backends();
        backends.shuffle(&mut rand::thread_rng());
        return backends.into_iter().next().unwrap_or_default();
    }
    backend.to_string()
}

pub fn cache_fname(img: &str, backend: &str, light: bool, sat: Option<&str>) -> PathBuf {
    let color_type = if light { "light" } else { "dark" };
    let mut file_name = String::new();
    for c in img.chars() {
        match c {
            '/' | '\\' | '|' | '.' => file_name.push('_'),
            _ => file_name.push(c),
        }
    }
    let file_size = std::fs::metadata(img).map(|m| m.len()).unwrap_or(0);
    let sat = sat.unwrap_or("");

    cache_dir()
        .join("schemes")
        .join(format!(
            "{}_{}_{}_{}_{}_{}.json",
            file_name, color_type, backend, sat, file_size, __CACHE_VERSION__
        ))
}

fn saturate_colors(mut colors: Vec<String>, sat: Option<&str>) -> Vec<String> {
    if let Some(amount) = sat {
        let amount: f64 = amount.parse().unwrap_or(0.0);
        if amount <= 1.0 {
            for i in 0..colors.len() {
                if !matches!(i, 0 | 7 | 8 | 15) {
                    colors[i] = util::saturate_color(&colors[i], amount);
                }
            }
        }
    }
    colors
}

pub fn get(img: &str, light: bool, backend: &str, sat: Option<&str>) -> Theme {
    let cache_file = cache_fname(img, backend, light, sat);

    if cache_file.is_file() {
        let value = util::read_file_json(&cache_file)
            .map_err(|e| e.to_string())
            .unwrap_or_else(|e| {
                util::log_error(__MODULE__, &e);
                std::process::exit(1)
            });
        let mut theme = Theme::from_value(value);
        theme.alpha = util::alpha_num();
        util::log_info(__MODULE__, "Found cached colorscheme.");
        theme
    } else {
        util::log_info(__MODULE__, "Generating a colorscheme.");
        let backend = get_backend(backend);
        util::log_info(__MODULE__, &format!("Using {} backend.", backend));

        let colors = backend::get(Path::new(img), light, 16).unwrap_or_else(|e| {
            util::log_error(__MODULE__, &e);
            std::process::exit(1)
        });
        let colors = saturate_colors(colors, sat);
        let theme = Theme::from_palette(&colors, img);

        if let Err(e) = util::save_file_json(&theme.to_value(), &cache_file) {
            util::log_warn(__MODULE__, &format!("Couldn't cache colorscheme: {}", e));
        }
        util::log_info(__MODULE__, "Generation complete.");
        theme
    }
}

pub fn palette_print() {
    for i in 0..16 {
        if i % 8 == 0 {
            println!();
        }
        let index = if i > 7 {
            format!("8;5;{}", i)
        } else {
            i.to_string()
        };
        print!("\x1b[4{}m{}\x1b[0m", index, " ".repeat(4));
    }
    println!("\n");
}
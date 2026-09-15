use std::path::{Path, PathBuf};

use rand::seq::SliceRandom;

use crate::settings::cache_dir;
use crate::util;

const __MODULE__: &str = "image";

const FILE_TYPES: [&str; 5] = [".png", ".jpg", ".jpeg", ".jpe", ".gif"];

fn is_image(name: &str) -> bool {
    let lower = name.to_lowercase();
    FILE_TYPES.iter().any(|t| lower.ends_with(t))
}

/// Abspath equivalent (lexical, works on non-existent paths).
pub fn abspath(path: &str) -> String {
    let p = Path::new(path);
    let absolute = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(p)
    };

    let mut out = PathBuf::new();
    for comp in absolute.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out.to_string_lossy().into_owned()
}

pub fn get_image_dir(img_dir: &Path) -> (Vec<String>, String) {
    let current_wall = basename(&crate::wallpaper::get());
    let mut images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(img_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_image(&name) {
                images.push(name);
            }
        }
    }
    (images, current_wall)
}

pub fn get_image_dir_recursive(img_dir: &Path) -> (Vec<String>, String) {
    let mut current_wall = basename(&crate::wallpaper::get());
    let mut images = Vec::new();

    let mut stack: Vec<PathBuf> = vec![img_dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_type = entry.file_type().ok();
                let is_dir = file_type
                    .map(|t| t.is_dir())
                    .unwrap_or(false);
                let is_file = file_type.map(|t| t.is_file()).unwrap_or(false);
                let name = entry.file_name().to_string_lossy().into_owned();
                let full = entry.path();

                if is_dir {
                    stack.push(entry.path());
                } else if is_file && is_image(&name) {
                    if name.ends_with(&current_wall) {
                        current_wall = full.to_string_lossy().into_owned();
                    }
                    images.push(full.to_string_lossy().into_owned());
                }
            }
        }
    }

    (images, current_wall)
}

pub fn get_random_image(img_dir: &Path, recursive: bool) -> String {
    let (mut images, current_wall) = if recursive {
        get_image_dir_recursive(img_dir)
    } else {
        get_image_dir(img_dir)
    };

    if images.len() > 2 && images.contains(&current_wall) {
        images.retain(|img| *img != current_wall);
    } else if images.is_empty() {
        util::log_error(__MODULE__, "No images found in directory.");
        std::process::exit(1);
    }

    images.shuffle(&mut rand::thread_rng());
    let first = images.into_iter().next().unwrap_or_default();
    if recursive {
        first
    } else {
        img_dir.join(&first).to_string_lossy().into_owned()
    }
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts = split_num(a);
    let b_parts = split_num(b);

    for (x, y) in a_parts.iter().zip(b_parts.iter()) {
        let ord = match (x, y) {
            (Part::Num(n), Part::Num(m)) => n.cmp(m),
            (Part::Str(s), Part::Str(t)) => s.cmp(t),
            (Part::Num(n), Part::Str(s)) => format!("{}", n).cmp(s),
            (Part::Str(s), Part::Num(n)) => s.cmp(&format!("{}", n)),
        };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    a_parts.len().cmp(&b_parts.len())
}

enum Part {
    Num(u64),
    Str(String),
}

fn split_num(s: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut num = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            num.push(c);
            if chars.peek().map(|p| !p.is_ascii_digit()).unwrap_or(true) {
                parts.push(Part::Num(num.parse().unwrap_or(0)));
                num.clear();
            }
        } else {
            parts.push(Part::Str(c.to_string()));
        }
    }
    parts
}

pub fn get_next_image(img_dir: &Path, recursive: bool) -> String {
    let (images, current_wall) = if recursive {
        get_image_dir_recursive(img_dir)
    } else {
        get_image_dir(img_dir)
    };

    if images.is_empty() {
        util::log_error(__MODULE__, "No images found in directory.");
        std::process::exit(1);
    }

    let mut sorted = images.clone();
    sorted.sort_by(|a, b| natural_cmp(a, b));

    let next_index = match sorted.iter().position(|img| *img == current_wall) {
        Some(index) => index + 1,
        None => 0,
    };

    let image = if next_index < sorted.len() {
        sorted[next_index].clone()
    } else {
        sorted[0].clone()
    };

    if recursive {
        image
    } else {
        img_dir.join(&image).to_string_lossy().into_owned()
    }
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

pub fn get(img: &str, iterative: bool, recursive: bool) -> String {
    let wal_img = if std::path::Path::new(img).is_file() {
        img.to_string()
    } else if std::path::Path::new(img).is_dir() {
        let dir = std::path::Path::new(img);
        if iterative {
            get_next_image(dir, recursive)
        } else {
            get_random_image(dir, recursive)
        }
    } else {
        util::log_error(__MODULE__, "No valid image file found.");
        std::process::exit(1);
    };

    let wal_img = abspath(&wal_img);

    // Cache the image file path.
    util::save_file(&wal_img, &cache_dir().join("wal"));

    util::log_info(
        __MODULE__,
        &format!("Using image \x1b[1;37m{}\x1b[0m.", basename(&wal_img)),
    );
    wal_img
}
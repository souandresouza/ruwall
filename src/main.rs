use std::io::IsTerminal;

use ruwall::{
    colors, export, image, reload, sequences, settings, theme, util, wallpaper,
};
use ruwall::settings::{__VERSION__, cache_dir};
use ruwall::theme::Theme;

struct Cli {
    alpha: Option<String>,
    background: Option<String>,
    backend: Option<String>,
    backend_given: bool,
    theme: Option<String>,
    theme_given: bool,
    iterative: bool,
    recursive: bool,
    saturate: Option<String>,
    preview: bool,
    vte: bool,
    clear_cache: bool,
    image: Option<String>,
    light: bool,
    no_wallpaper: bool,
    scripts: Vec<String>,
    save_theme: Option<String>,
    quiet: bool,
    restore: bool,
    skip_sequences: bool,
    skip_tty: bool,
    reload_old: bool,
    version: bool,
    cached_wallpaper: bool,
    no_env_reload: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            alpha: None,
            background: None,
            backend: None,
            backend_given: false,
            theme: None,
            theme_given: false,
            iterative: false,
            recursive: false,
            saturate: None,
            preview: false,
            vte: false,
            clear_cache: false,
            image: None,
            light: false,
            no_wallpaper: false,
            scripts: Vec::new(),
            save_theme: None,
            quiet: false,
            restore: false,
            skip_sequences: false,
            skip_tty: false,
            reload_old: false,
            version: false,
            cached_wallpaper: false,
            no_env_reload: false,
        }
    }
}

fn print_help() {
    println!("ruwall - Generate colorschemes on the fly");
    println!();
    println!("usage: ruwall [-h] [-a ALPHA] [-b BACKGROUND] [--backend [BACKEND]]");
    println!("            [-f FILE] [--iterative] [--recursive]");
    println!("            [--saturate SATURATE] [--preview] [--vte] [-c] [-i IMAGE]");
    println!("            [-l] [-n] [-o SCRIPT_NAME] [-p THEME_NAME] [-q] [-r] [-R]");
    println!("            [-s] [-t] [-v] [-w] [-e]");
    println!();
    println!("options:");
    println!("  -h, --help            show this help message and exit");
    println!("  -a ALPHA              Set terminal background transparency. *Only works in URxvt*");
    println!("  -b BACKGROUND         Custom background color to use.");
    println!("  --backend [BACKEND]   Which color backend to use.");
    println!("                        Use 'ruwall --backend' to list backends.");
    println!("  -f FILE, --theme [FILE]");
    println!("                        Which colorscheme file to use.");
    println!("                        Use 'ruwall --theme' to list builtin and user themes.");
    println!("  --iterative           When ruwall is given a directory as input and this flag is used:");
    println!("                        Go through the images in order instead of shuffled.");
    println!("  --recursive           When ruwall is given a directory as input and this flag is used:");
    println!("                        Search for images recursively in subdirectories instead of the root only.");
    println!("  --saturate SATURATE   Set the color saturation.");
    println!("  --preview             Print the current color palette.");
    println!("  --vte                 Fix text-artifacts printed in VTE terminals.");
    println!("  -c                    Delete all cached colorschemes.");
    println!("  -i IMAGE              Which image or directory to use.");
    println!("  -l                    Generate a light colorscheme.");
    println!("  -n                    Skip setting the wallpaper.");
    println!("  -o SCRIPT_NAME        External script to run after \"ruwall\".");
    println!("  -p THEME_NAME         permanently save theme to $XDG_CONFIG_HOME/wal/colorschemes with");
    println!("                        the specified name");
    println!("  -q                    Quiet mode, don't print anything.");
    println!("  -r                    'ruwall -r' is deprecated: Use (cat ~/.cache/wal/sequences &) instead.");
    println!("  -R                    Restore previous colorscheme.");
    println!("  -s                    Skip changing colors in terminals.");
    println!("  -t                    Skip changing colors in tty.");
    println!("  -v                    Print \"ruwall\" version.");
    println!("  -w                    Use last used wallpaper for color generation.");
    println!("  -e                    Skip reloading gtk/xrdb/i3/sway/polybar");
}

fn cli_error(message: &str) -> ! {
    eprintln!("usage: ruwall [-h] [-a ALPHA] [-b BACKGROUND] [--backend [BACKEND]]");
    eprintln!("            [-f FILE] [--iterative] [--recursive]");
    eprintln!("            [--saturate SATURATE] [--preview] [--vte] [-c] [-i IMAGE]");
    eprintln!("            [-l] [-n] [-o SCRIPT_NAME] [-p THEME_NAME] [-q] [-r] [-R]");
    eprintln!("            [-s] [-t] [-v] [-w] [-e]");
    eprintln!("wal: error: {}", message);
    std::process::exit(2);
}

fn parse_args(raw: &[String]) -> Cli {
    let mut cli = Cli::default();
    let mut i = 0;

    fn value_from(input: &[String], i: &mut usize, opt: &str) -> String {
        if *i + 1 < input.len() {
            *i += 1;
            input[*i].clone()
        } else {
            cli_error(&format!("argument {}: expected one argument", opt));
        }
    }

    while i < raw.len() {
        let arg = &raw[i];

        if arg == "--" {
            i += 1;
            continue;
        }

        if let Some(rest) = arg.strip_prefix("--") {
            let (name, inline) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (rest, None),
            };

            match name {
                "backend" => {
                    cli.backend_given = true;
                    cli.backend = match inline {
                        Some(v) => {
                            i += 1;
                            Some(v)
                        }
                        None => {
                            if i + 1 < raw.len() && !raw[i + 1].starts_with('-') {
                                i += 1;
                                Some(raw[i].clone())
                            } else {
                                Some("list_backends".to_string())
                            }
                        }
                    };
                }
                "theme" => {
                    cli.theme_given = true;
                    cli.theme = match inline {
                        Some(v) => {
                            i += 1;
                            Some(v)
                        }
                        None => {
                            if i + 1 < raw.len() && !raw[i + 1].starts_with('-') {
                                i += 1;
                                Some(raw[i].clone())
                            } else {
                                Some("list_themes".to_string())
                            }
                        }
                    };
                }
                "saturate" => {
                    cli.saturate = Some(value_from(raw, &mut i, "--saturate"));
                }
                "iterative" => cli.iterative = true,
                "recursive" => cli.recursive = true,
                "preview" => cli.preview = true,
                "vte" => cli.vte = true,
                "help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => cli_error(&format!("unrecognized arguments: {}", arg)),
            }
            i += 1;
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            let chars: Vec<char> = arg[1..].chars().collect();
            let mut j = 0;
            while j < chars.len() {
                let c = chars[j];
                match c {
                    'a' | 'b' | 'i' | 'o' | 'p' => {
                        let name = c.to_string();
                        let value = if j + 1 < chars.len() {
                            chars[j + 1..].iter().collect::<String>()
                        } else {
                            value_from(raw, &mut i, &format!("-{}", name))
                        };
                        match c {
                            'a' => cli.alpha = Some(value),
                            'b' => cli.background = Some(value),
                            'i' => cli.image = Some(value),
                            'o' => cli.scripts.push(value),
                            _ => cli.save_theme = Some(value),
                        }
                        break;
                    }
                    'f' => {
                        cli.theme_given = true;
                        if j + 1 < chars.len() {
                            cli.theme = Some(chars[j + 1..].iter().collect());
                        } else if i + 1 < raw.len() && !raw[i + 1].starts_with('-') {
                            i += 1;
                            cli.theme = Some(raw[i].clone());
                        } else {
                            cli.theme = Some("list_themes".to_string());
                        }
                        break;
                    }
                    'l' => cli.light = true,
                    'c' => cli.clear_cache = true,
                    'e' => cli.no_env_reload = true,
                    'n' => cli.no_wallpaper = true,
                    'q' => cli.quiet = true,
                    'r' => cli.reload_old = true,
                    'R' => cli.restore = true,
                    's' => cli.skip_sequences = true,
                    't' => cli.skip_tty = true,
                    'v' => cli.version = true,
                    'w' => cli.cached_wallpaper = true,
                    'h' => {
                        print_help();
                        std::process::exit(0);
                    }
                    _ => cli_error(&format!("unrecognized arguments: -{}", c)),
                }
                j += 1;
            }
            i += 1;
            continue;
        }

        cli_error(&format!("unrecognized arguments: {}", arg));
    }

    cli
}

fn process_exiting(cli: &Cli, raw_len: usize) {
    if raw_len <= 1 {
        print_help();
        std::process::exit(1);
    }

    if cli.version {
        println!("ruwall {}\n", __VERSION__);
        std::process::exit(0);
    }

    if cli.preview {
        println!("Current colorscheme:");
        colors::palette_print();
        std::process::exit(0);
    }

    if cli.image.is_some() && cli.theme_given {
        cli_error("Conflicting arguments -i and -f.");
    }

    if cli.reload_old {
        reload::colors();
        std::process::exit(0);
    }

    if cli.clear_cache {
        let scheme_dir = cache_dir().join("schemes");
        let _ = std::fs::remove_dir_all(&scheme_dir);
        std::process::exit(0);
    }

    if cli.image.is_none()
        && !cli.theme_given
        && !cli.restore
        && !cli.cached_wallpaper
        && !cli.backend_given
    {
        cli_error("No input specified.\n--backend, --theme, -i or -R are required.");
    }

    if cli.theme.as_deref() == Some("list_themes") {
        theme::list_out();
        std::process::exit(0);
    }

    if cli.backend.as_deref() == Some("list_backends") {
        println!(
            "{}{}",
            "\x1b[1;32mBackends\x1b[0m:",
            colors::list_backends()
                .iter()
                .map(|b| format!("\n - {}", b))
                .collect::<Vec<_>>()
                .join("")
        );
        std::process::exit(0);
    }
}

fn process(cli: &Cli) {
    if cli.quiet {
        util::set_quiet(true);
    }

    if let Some(alpha) = &cli.alpha {
        util::set_alpha_num(alpha);
    }

    let mut colors_plain: Option<Theme> = None;

    if let Some(img) = &cli.image {
        let image_file = image::get(img, cli.iterative, cli.recursive);
        colors_plain = Some(colors::get(
            &image_file,
            cli.light,
            cli.backend.as_deref().unwrap_or("wal"),
            cli.saturate.as_deref(),
        ));
    } else if cli.theme_given {
        let theme_name = cli.theme.as_deref().unwrap_or("list_themes");
        colors_plain = Some(theme::file(theme_name, cli.light));
    } else if cli.restore {
        colors_plain = Some(theme::file(
            &cache_dir().join("colors.json").to_string_lossy(),
            cli.light,
        ));
    } else if cli.cached_wallpaper {
        let cached = util::read_file(&cache_dir().join("wal")).unwrap_or_default();
        if let Some(wall) = cached.into_iter().next() {
            colors_plain = Some(colors::get(
                &wall,
                cli.light,
                cli.backend.as_deref().unwrap_or("wal"),
                cli.saturate.as_deref(),
            ));
        } else {
            cli_error("No cached wallpaper found.");
        }
    }

    let colors_plain = match colors_plain {
        Some(theme) => theme,
        None => {
            cli_error("No input specified.\n--backend, --theme, -i or -R are required.");
        }
    };

    let mut colors_plain = colors_plain;
    if let Some(bg) = &cli.background {
        let bg = format!("#{}", bg.trim_start_matches('#'));
        colors_plain.special.background = bg.clone();
        colors_plain.colors.set(0, bg);
    }

    if !cli.no_wallpaper {
        wallpaper::change(&colors_plain.wallpaper);
    }

    if let Some(name) = &cli.save_theme {
        theme::save(&colors_plain, name, cli.light);
    }

    sequences::send(
        &colors_plain,
        &cache_dir(),
        !cli.skip_sequences,
        cli.vte,
    );

    if !cli.quiet && std::io::stdout().is_terminal() {
        colors::palette_print();
    }

    export::every(&colors_plain, &cache_dir());

    if !cli.no_env_reload {
        let xrdb_file = None;
        reload::env(xrdb_file, !cli.skip_tty);
    }

    for cmd in &cli.scripts {
        util::disown(&[cmd]);
    }

    if !cli.no_env_reload {
        reload::gtk();
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().collect();
    let raw_len = raw.len();

    util::create_dir(&settings::conf_dir().join("templates")).ok();
    util::create_dir(&settings::conf_dir().join("colorschemes/light")).ok();
    util::create_dir(&settings::conf_dir().join("colorschemes/dark")).ok();

    let cli = parse_args(&raw[1..]);

    process_exiting(&cli, raw_len);
    process(&cli);
}
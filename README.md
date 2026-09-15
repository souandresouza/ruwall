<h3 align="center"><img src="https://i.imgur.com/5WgMACe.gif" width="200px"></h3>
<p align="center">Generate and change color-schemes on the fly.</p>

<p align="center">
<a href="./LICENSE.md"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

Ruwall is a tool that generates a color palette from the dominant colors in an image. It then applies the colors system-wide and on-the-fly in all of your favourite programs.

This is a native [Rust](https://www.rust-lang.org/) port of the classic
[pywal](https://github.com/dylanaraps/pywal) tool. It keeps the same command
line interface and behavior as the original, with everything (including the
image color extraction backend) bundled into a single binary. No Python
interpreter is required.

Predefined themes are built into the binary (over 250 themes ships inside
`assets/colorschemes`), together with all output templates (`assets/templates`)
and the GTK reload helper (`assets/scripts`). The templates are embedded at
compile time, so the binary is self-contained.

Terminal emulators and TTYs have their color-schemes updated in real-time with
no delay. With minimal configuration this functionality can be extended to
almost anything running on your system.

## Installation

Requirements: a Rust toolchain (2021 edition, rustc 1.80+ recommended).

```sh
cargo build --release
# The binary lands in target/release/ruwall
install -m755 target/release/ruwall ~/.local/bin/ruwall
```

## Usage

```
ruwall [-h] [-a ALPHA] [-b BACKGROUND] [--backend [BACKEND]]
    [-f FILE] [--iterative] [--recursive]
    [--saturate SATURATE] [--preview] [--vte] [-c] [-i IMAGE]
    [-l] [-n] [-o SCRIPT_NAME] [-p THEME_NAME] [-q] [-r] [-R]
    [-s] [-t] [-v] [-w] [-e]
```

Generate a colorscheme from an image:

```sh
ruwall -i path/to/image.jpg
```

Use a predefined theme (see `ruwall --theme` for the full list):

```sh
ruwall -f base16-monokai
```

List the available color backends:

```sh
ruwall --backend
```

The cached colorscheme is written to the cache directory (`$XDG_CACHE_HOME/wal`
or `~/.cache/wal`) as `colors.json` plus one output file per template
(`colors.sh`, `colors.css`, `colors.Xresources`, …), exactly like the
original `wal`. This keeps existing scripts and templates working unchanged.

### Options

| Option | Description |
| --- | --- |
| `-a ALPHA` | Set terminal background transparency. Only works in URxvt. |
| `-b BACKGROUND` | Custom background color to use. |
| `--backend [BACKEND]` | Which color backend to use. There is one native backend, `wal` (kmeans). |
| `-f FILE, --theme [FILE]` | Which colorscheme file to use. Lists all themes without a value. |
| `--iterative` | Pick images from a directory in order instead of shuffled. |
| `--recursive` | Search a directory recursively for images. |
| `--saturate SATURATE` | Set the color saturation (0.0 to 1.0). |
| `--preview` | Print the current color palette. |
| `--vte` | Fix text-artifacts printed in VTE terminals. |
| `-c` | Delete all cached colorschemes. |
| `-i IMAGE` | Which image or directory to use. |
| `-l` | Generate a light colorscheme. |
| `-n` | Skip setting the wallpaper. |
| `-o SCRIPT_NAME` | External script to run after ruwall. |
| `-p THEME_NAME` | Save the generated theme to `$XDG_CONFIG_HOME/wal/colorschemes`. |
| `-q` | Quiet mode, don't print anything. |
| `-r` | Deprecated, use `cat ~/.cache/wal/sequences` instead. |
| `-R` | Restore the previous colorscheme. |
| `-s` | Skip changing colors in terminals. |
| `-t` | Skip changing colors in the Linux TTY. |
| `-v` | Print the ruwall version. |
| `-w` | Use the cached wallpaper for color generation. |
| `-e` | Skip reloading gtk/xrdb/i3/sway/polybar. |

## Custom themes and templates

User themes go in `$XDG_CONFIG_HOME/wal/colorschemes/{dark,light}/`. User
templates go in `$XDG_CONFIG_HOME/wal/templates/`; any template there is
rendered on every run alongside the built-in ones. See
`assets/templates/colors.css` for the available `{color}`, `{background}`,
`{background.alpha}` … substitutions.

## Development

```sh
cargo build
cargo test
```

The source is split into modules mirroring the original pywal package:
`settings`, `util`, `theme`, `image`, `backend` (kmeans), `colors`,
`sequences`, `export`, `wallpaper`, `reload` and the CLI entry point `main`.

## License

MIT. See [LICENSE](./LICENSE).

Portions of this project (the theme files, output templates and the general
behavior) are derived from [pywal](https://github.com/dylanaraps/pywal),
MIT licensed by Dylan Araps; that upstream license is kept in
[LICENSE.md](./LICENSE.md).
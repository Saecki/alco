use anyhow::bail;
use serde::{Deserialize, Serialize};
use yaml_rust::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust::scanner::Marker;
use yaml_rust::{Yaml, YamlLoader};

use std::path::Path;
use std::time::{Duration, SystemTime};
use std::{fs, io};

pub use cmus::reload_cmus;
pub use delta::reload_delta;
pub use nvim::reload_neovim;
pub use tmux::reload_tmux;

pub const DEFAULT_CONFIG_FILE: &str = "~/.config/alacritty/alacritty.yml";
pub const DEFAULT_COLORSCHEME_DIR: &str = "~/.config/alacritty/colors/";
pub const DEFAULT_TMUX_FILE: &str = "~/.config/tmux/colors/current.conf";
pub const DEFAULT_TMUX_SELECTOR: &str = "~/.config/alco/tmux-selector.yml";
pub const DEFAULT_NEOVIM_FILE: &str = "~/.config/nvim/colors.vim";
pub const DEFAULT_DELTA_FILE: &str = "~/.config/delta/colors/current.gitconfig";
pub const DEFAULT_DELTA_SELECTOR: &str = "~/.config/alco/delta-selector.yml";
pub const DEFAULT_CMUS_SELECTOR: &str = "~/.config/alco/cmus-selector.yml";

#[cfg(feature = "tmux")]
mod tmux;
#[cfg(not(feature = "tmux"))]
mod tmux {
    use anyhow::bail;
    use std::path::Path;

    pub fn reload_tmux(
        _: impl AsRef<Path>,
        _: impl AsRef<Path>,
        _: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        bail!("alco was compiled without the tmux feature flag")
    }
}

#[cfg(feature = "neovim")]
mod nvim;
#[cfg(not(feature = "neovim"))]
mod nvim {
    use anyhow::bail;
    use std::path::Path;

    pub async fn reload_neovim() -> anyhow::Result<()> {
        bail!("alco was compiled without the neovim feature flag")
    }
}

#[cfg(feature = "delta")]
mod delta;
#[cfg(not(feature = "delta"))]
mod delta {
    use anyhow::bail;
    use std::path::Path;

    pub fn reload_delta(
        _: impl AsRef<Path>,
        _: impl AsRef<Path>,
        _: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        bail!("alco was compiled without the delta feature flag")
    }
}

#[cfg(feature = "cmus")]
mod cmus;
#[cfg(not(feature = "cmus"))]
mod cmus {
    use anyhow::bail;
    use std::path::Path;

    pub fn reload_cmus(_: impl AsRef<Path>, _: impl AsRef<str>) -> anyhow::Result<()> {
        bail!("alco was compiled without the tmux feature flag")
    }
}

struct ColorEventReceiver<T> {
    listener: T,
}

impl<T: FnMut(Event, Marker)> ColorEventReceiver<T> {
    fn new(listener: T) -> Self {
        Self { listener }
    }
}

impl<T: FnMut(Event, Marker)> MarkedEventReceiver for ColorEventReceiver<T> {
    fn on_event(&mut self, event: Event, marker: Marker) {
        (self.listener)(event, marker);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Current {
    changed: String,
}

impl Current {
    fn new(changed: String) -> Self {
        Self { changed }
    }

    fn now() -> Self {
        let changed = humantime::format_rfc3339(SystemTime::now()).to_string();
        Self::new(changed)
    }
}

pub struct Status {
    pub file_name: String,
    pub duration: Duration,
}

impl Status {
    pub fn new(file_name: String, duration: Duration) -> Self {
        Status { file_name, duration }
    }
}

pub fn apply(
    config_file: impl AsRef<Path>,
    scheme_dir: impl AsRef<Path>,
    scheme_file: &str,
) -> anyhow::Result<()> {
    let new_colors = parse_colors(scheme_dir.as_ref().join(scheme_file))?;

    let config_str = fs::read_to_string(config_file.as_ref())?;
    let config_lines = config_str.lines().collect::<Vec<_>>();
    let mut new_config_str = String::new();
    let mut line_index = 0;

    let mut current_path: Vec<String> = Vec::new();
    let mut last_line = 0;
    let mut last_col = 0;

    let mut parser = Parser::new(config_str.chars());
    let mut receiver = ColorEventReceiver::new(|event, mark| {
        if let Event::Scalar(name, _, _, _) = event {
            if mark.line() != last_line {
                if mark.col() == last_col {
                    current_path.pop();
                    current_path.push(name);
                    last_line = mark.line();
                    last_col = mark.col();
                } else if mark.col() == last_col + 2 {
                    current_path.push(name);
                    last_line = mark.line();
                    last_col = mark.col();
                } else if mark.col() < last_col {
                    let indent = mark.col() / 2;
                    for _ in indent..current_path.len() {
                        current_path.pop();
                    }
                    current_path.push(name);
                    last_line = mark.line();
                    last_col = mark.col();
                }
            } else if let Some(v) = value(&new_colors, &current_path) {
                if let Some(stringified) = stringify(v) {
                    for i in line_index..mark.line() - 1 {
                        new_config_str.push_str(config_lines[i]);
                        new_config_str.push('\n');
                    }
                    new_config_str.push_str(&config_lines[mark.line() - 1][0..mark.col()]);
                    new_config_str.push_str(&stringified);
                    new_config_str.push('\n');
                    line_index = mark.line();
                }
            }
        }
    });
    parser.load(&mut receiver, true)?;

    for i in line_index..config_lines.len() {
        new_config_str.push_str(config_lines[i]);
        new_config_str.push('\n');
    }

    fs::write(config_file, new_config_str)?;

    let current_dir = scheme_dir.as_ref().join("current");
    let current_file = current_dir.join(scheme_file);
    let _ = fs::remove_dir_all(&current_dir);
    fs::create_dir_all(&current_dir)?;
    let current_str = serde_yaml::to_string(&Current::now())?;
    fs::write(current_file, current_str)?;

    Ok(())
}

pub fn toggle(
    config_file: impl AsRef<Path>,
    scheme_dir: impl AsRef<Path>,
    reverse: bool,
) -> anyhow::Result<String> {
    let mut available_schemes: Vec<_> = list(scheme_dir.as_ref())?;
    if available_schemes.is_empty() {
        bail!("No colorschemes available");
    }
    available_schemes.sort();

    let mut index = 0;
    if let Ok(c) = status(scheme_dir.as_ref()) {
        if let Some(i) = available_schemes.iter().position(|f| f == &c.file_name) {
            index = if reverse {
                (available_schemes.len() + i - 1) % available_schemes.len()
            } else {
                (i + 1) % available_schemes.len()
            };
        }
    }

    let new_scheme = available_schemes.remove(index);

    apply(config_file, scheme_dir, &new_scheme)?;

    Ok(new_scheme)
}

pub fn list(dir: impl AsRef<Path>) -> Result<Vec<String>, io::Error> {
    fs::read_dir(dir.as_ref()).map(|read_dir| {
        read_dir
            .filter_map(Result::ok)
            .filter(|d| d.metadata().map(|m| m.is_file()).unwrap_or(false))
            .filter_map(|d| d.file_name().to_str().map(str::to_owned))
            .collect()
    })
}

pub fn status(scheme_dir: impl AsRef<Path>) -> anyhow::Result<Status> {
    let mut current_file = scheme_dir.as_ref().join("current");

    match fs::read_dir(&current_file)?.into_iter().next() {
        Some(Ok(d)) => match d.file_name().to_str().map(str::to_owned) {
            Some(c) => {
                current_file.push(&c);
                match parse_current(current_file) {
                    Ok(d) => Ok(Status::new(c, d)),
                    Err(_) => Ok(Status::new(c, Duration::new(0, 0))),
                }
            }
            None => bail!("Error reading current colorscheme file"),
        },
        _ => bail!("No current colorscheme file found"),
    }
}

fn parse_current(file: impl AsRef<Path>) -> anyhow::Result<Duration> {
    let status_str = fs::read_to_string(file)?;
    let current: Current = serde_yaml::from_str(&status_str)?;
    let time = humantime::parse_rfc3339(&current.changed)?;
    let now = SystemTime::now();
    let duration = now.duration_since(time)?;

    Ok(duration)
}

fn parse_colors(file: impl AsRef<Path>) -> anyhow::Result<Yaml> {
    let config_str = fs::read_to_string(file)?;
    let config = YamlLoader::load_from_str(&config_str)?;

    if let Some(c) = config.into_iter().next() {
        return Ok(c);
    }

    bail!("Error parsing colors")
}

fn value<'a>(yaml: &'a Yaml, path: &[String]) -> Option<&'a Yaml> {
    let mut current = yaml;

    for key in path {
        if let Yaml::Hash(h) = current {
            let value = h.iter().find(|(k, _)| match k {
                Yaml::String(s) => s == key,
                _ => false,
            });

            current = value?.1;
        }
    }

    Some(current)
}

fn stringify(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(s) => Some(format!("'{}'", s)),
        Yaml::Integer(i) => Some(i.to_string()),
        Yaml::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}

fn selector<'a>(selector: &'a Yaml, key: &'_ str) -> Option<&'a str> {
    let map = selector.as_hash()?;
    let mut default = None;

    for (k, v) in map.iter() {
        if k.as_str()? == key {
            return v.as_str();
        } else if k.as_str()? == "else" {
            default = v.as_str();
        }
    }

    default
}

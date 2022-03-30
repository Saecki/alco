use async_std::task::{block_on, spawn};
use clap::{crate_authors, crate_version, Arg, Command, ValueHint};
use clap_complete::generate;
use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
use shellexpand::tilde;

use std::path::Path;
use std::process::exit;
use std::time::Duration;

const BIN_NAME: &str = "alco";

const BASH: &str = "bash";
const ELVISH: &str = "elvish";
const FISH: &str = "fish";
const PWRSH: &str = "powershell";
const ZSH: &str = "zsh";

struct Options {
    alacritty: AlacrittyOptions,
    kitty: KittyOptions,
    tmux: TmuxOptions,
    neovim: NeovimOptions,
    starship: StarshipOptions,
    delta: DeltaOptions,
    cmus: CmusOptions,
}

struct AlacrittyOptions {
    reload: bool,
    file: String,
    in_file: String,
    selector: String,
}

struct KittyOptions {
    reload: bool,
    file: String,
    socket: String,
    selector: String,
}

struct TmuxOptions {
    reload: bool,
    file: String,
    selector: String,
}

struct NeovimOptions {
    reload: bool,
    command: String,
}

struct StarshipOptions {
    reload: bool,
    file: String,
    in_file: String,
    selector: String,
}

struct DeltaOptions {
    reload: bool,
    file: String,
    selector: String,
}

struct CmusOptions {
    reload: bool,
    selector: String,
}

fn main() {
    let mut app = Command::new("alco")
        .bin_name(BIN_NAME)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Update terminal colorschemes on the fly")
        .arg(
            Arg::new("configuration file")
                .long("config-file")
                .short('c')
                .default_value(alco::DEFAULT_CONFIG_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("Alco's configuration file"),
        )
        .arg(
            Arg::new("colorscheme file")
                .long("colorscheme-file")
                .short('C')
                .default_value(alco::DEFAULT_COLORSCHEME_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The file that contains a list of colorschemes"),
        )
        .arg(
            Arg::new("reload all")
                .long("reload-all")
                .short('a')
                .takes_value(false)
                .help("Reload all additional colorschemes"),
        )
        .arg(
            Arg::new("reload alacritty")
                .long("reload-alacritty")
                .short('A')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload alacritty updating the configuration file"),
        )
        .arg(
            Arg::new("alacritty file")
                .long("alacritty-file")
                .default_value(alco::DEFAULT_ALACRITTY_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The alacritty configuration file which will updated"),
        )
        .arg(
            Arg::new("alacritty in file")
                .long("alacritty-in-file")
                .default_value(alco::DEFAULT_ALACRITTY_IN_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The alacritty configuration file which will updated"),
        )
        .arg(
            Arg::new("alacritty selector")
                .long("alacritty-selector")
                .default_value(alco::DEFAULT_ALACRITTY_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The alacritty selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("reload kitty")
                .long("reload-kitty")
                .short('k')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload kitty by sourcing a configuration file"),
        )
        .arg(
            Arg::new("kitty file")
                .long("kitty-file")
                .default_value(alco::DEFAULT_KITTY_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The kitty configuration file which will be overwritten and sourced"),
        )
        .arg(
            Arg::new("kitty selector")
                .long("kitty-selector")
                .default_value(alco::DEFAULT_KITTY_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The kitty selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("kitty socket")
                .long("kitty-socket")
                .default_value(alco::DEFAULT_KITTY_SOCKET)
                .value_name("socket")
                .value_hint(ValueHint::FilePath)
                .help("The unix socket on which kitty is listening for remote control"),
        )
        .arg(
            Arg::new("reload tmux")
                .long("reload-tmux")
                .short('t')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload tmux by sourcing a configuration file"),
        )
        .arg(
            Arg::new("tmux file")
                .long("tmux-file")
                .default_value(alco::DEFAULT_TMUX_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The tmux configuration file which will be overwritten and sourced"),
        )
        .arg(
            Arg::new("tmux selector")
                .long("tmux-selector")
                .default_value(alco::DEFAULT_TMUX_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The tmux selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("reload neovim")
                .long("reload-neovim")
                .short('n')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload neovim by sourcing a configuration file"),
        )
        .arg(
            Arg::new("neovim command")
                .long("neovim-command")
                .default_value(alco::DEFAULT_NEOVIM_COMMAND)
                .value_name("command")
                .value_hint(ValueHint::FilePath)
                .help("The neovim command that will be executed to update the colorscheme"),
        )
        .arg(
            Arg::new("reload starship")
                .long("reload-starship")
                .short('d')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload starship by updating the configuration file"),
        )
        .arg(
            Arg::new("starship file")
                .long("starship-file")
                .default_value(alco::DEFAULT_STARSHIP_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The starship configuration file which will be overwritten"),
        )
        .arg(
            Arg::new("starship in file")
                .long("starship-in-file")
                .default_value(alco::DEFAULT_STARSHIP_IN_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The starship in file which will be read"),
        )
        .arg(
            Arg::new("starship selector")
                .long("starship-selector")
                .default_value(alco::DEFAULT_STARSHIP_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The starship selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("reload delta")
                .long("reload-delta")
                .short('d')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload delta by updating the configuration file"),
        )
        .arg(
            Arg::new("delta file")
                .long("delta-file")
                .default_value(alco::DEFAULT_DELTA_FILE)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The delta configuration file which will be overwritten"),
        )
        .arg(
            Arg::new("delta selector")
                .long("delta-selector")
                .default_value(alco::DEFAULT_DELTA_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The delta selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("reload cmus")
                .long("reload-cmus")
                .short('m')
                .takes_value(false)
                .conflicts_with("reload all")
                .help("Also reload cmus by sourcing a configuration file"),
        )
        .arg(
            Arg::new("cmus selector")
                .long("cmus-selector")
                .default_value(alco::DEFAULT_CMUS_SELECTOR)
                .value_name("file")
                .value_hint(ValueHint::FilePath)
                .help("The cmus selector file which contains a colorscheme mapping"),
        )
        .arg(
            Arg::new("generate completion")
                .long("generate-completion")
                .short('g')
                .value_name("shell")
                .possible_values(&[BASH, ZSH, FISH, ELVISH, PWRSH])
                .help("Generates a completion script for the specified shell"),
        )
        .subcommands(vec![
            Command::new("apply")
                .bin_name("alco-apply")
                .about("Apply a colorscheme")
                .arg(Arg::new("colorscheme").index(1).value_name("colorscheme").required(true)),
            Command::new("toggle")
                .bin_name("alco-toggle")
                .about("Toggle the colorscheme between available options")
                .arg(
                    Arg::new("reverse")
                        .long("reverse")
                        .short('r')
                        .takes_value(false)
                        .help("Toggle in reverse order between available colorschemes"),
                ),
            Command::new("list").bin_name("alco-list").about("List available colorschemes"),
            Command::new("status").bin_name("alco-status").about("Print the current status").arg(
                Arg::new("time")
                    .long("time")
                    .short('t')
                    .takes_value(false)
                    .help("Print the duration since the last change"),
            ),
        ]);

    let app_m = app.clone().get_matches();

    let generate_completion = app_m.value_of("generate completion");
    if let Some(shell) = generate_completion {
        let mut stdout = std::io::stdout();
        match shell {
            BASH => generate(Bash, &mut app, BIN_NAME, &mut stdout),
            ELVISH => generate(Elvish, &mut app, BIN_NAME, &mut stdout),
            FISH => generate(Fish, &mut app, BIN_NAME, &mut stdout),
            ZSH => generate(Zsh, &mut app, BIN_NAME, &mut stdout),
            PWRSH => generate(PowerShell, &mut app, BIN_NAME, &mut stdout),
            _ => unreachable!(),
        }

        exit(0);
    }

    let colors_file = tilde(app_m.value_of("colorscheme file").unwrap()).into_owned();
    let config_file = tilde(app_m.value_of("configuration file").unwrap()).into_owned();
    let reload_all = app_m.is_present("reload all");

    let alacritty = AlacrittyOptions {
        reload: app_m.is_present("reload alacritty") | reload_all,
        file: tilde(app_m.value_of("alacritty file").unwrap()).into_owned(),
        in_file: tilde(app_m.value_of("alacritty in file").unwrap()).into_owned(),
        selector: tilde(app_m.value_of("alacritty selector").unwrap()).into_owned(),
    };
    let kitty = KittyOptions {
        reload: app_m.is_present("reload kitty") | reload_all,
        file: tilde(app_m.value_of("kitty file").unwrap()).into_owned(),
        socket: tilde(app_m.value_of("kitty socket").unwrap()).into_owned(),
        selector: tilde(app_m.value_of("kitty selector").unwrap()).into_owned(),
    };
    let tmux = TmuxOptions {
        reload: app_m.is_present("reload tmux") | reload_all,
        file: tilde(app_m.value_of("tmux file").unwrap()).into_owned(),
        selector: tilde(app_m.value_of("tmux selector").unwrap()).into_owned(),
    };
    let neovim = NeovimOptions {
        reload: app_m.is_present("reload neovim") | reload_all,
        command: app_m.value_of("neovim command").unwrap().to_owned(),
    };
    let starship = StarshipOptions {
        reload: app_m.is_present("reload starship") | reload_all,
        file: tilde(app_m.value_of("starship file").unwrap()).into_owned(),
        in_file: tilde(app_m.value_of("starship in file").unwrap()).into_owned(),
        selector: tilde(app_m.value_of("starship selector").unwrap()).into_owned(),
    };
    let delta = DeltaOptions {
        reload: app_m.is_present("reload delta") | reload_all,
        file: tilde(app_m.value_of("delta file").unwrap()).into_owned(),
        selector: tilde(app_m.value_of("delta selector").unwrap()).into_owned(),
    };
    let cmus = CmusOptions {
        reload: app_m.is_present("reload cmus") | reload_all,
        selector: tilde(app_m.value_of("cmus selector").unwrap()).into_owned(),
    };

    let opts = Options {
        alacritty,
        kitty,
        tmux,
        neovim,
        starship,
        delta,
        cmus,
    };

    match app_m.subcommand() {
        Some(("apply", sub_m)) => {
            let colorscheme = sub_m.value_of("colorscheme").unwrap().to_owned();
            apply(colors_file, config_file, &colorscheme, opts);
        }
        Some(("toggle", sub_m)) => {
            let reverse = sub_m.is_present("reverse");
            toggle(colors_file, config_file, reverse, opts);
        }
        Some(("list", _)) => list(colors_file),
        Some(("status", sub_m)) => {
            let time = sub_m.is_present("time");
            status(config_file, time);
        }
        _ => {
            app.print_help().ok();
        }
    }
}

fn apply(
    colors_file: impl AsRef<Path>,
    config_file: impl AsRef<Path>,
    colorscheme: &str,
    opts: Options,
) {
    match alco::apply(colors_file, config_file, colorscheme.to_owned()) {
        Ok(_) => apply_colorscheme(colorscheme, opts),
        Err(e) => {
            println!("Error applying colorscheme {}:\n{:?}", colorscheme, e);
        }
    }
}

fn toggle(
    colors_file: impl AsRef<Path>,
    config_file: impl AsRef<Path>,
    reverse: bool,
    opts: Options,
) {
    match alco::toggle(&colors_file, &config_file, reverse) {
        Ok(colorscheme) => apply_colorscheme(&colorscheme, opts),
        Err(e) => println!("Error toggling colorscheme:\n{}", e),
    }
}

fn apply_colorscheme(colorscheme: &str, opts: Options) {
    block_on(async move {
        let a = if opts.alacritty.reload {
            Some(spawn(reload_alacritty(opts.alacritty, colorscheme.to_owned())))
        } else {
            None
        };
        let k = if opts.kitty.reload {
            Some(spawn(reload_kitty(opts.kitty, colorscheme.to_owned())))
        } else {
            None
        };
        let t = if opts.tmux.reload {
            Some(spawn(reload_tmux(opts.tmux, colorscheme.to_owned())))
        } else {
            None
        };
        let n =
            if opts.neovim.reload { Some(spawn(reload_neovim(opts.neovim.command))) } else { None };
        let s = if opts.starship.reload {
            Some(spawn(reload_starship(opts.starship, colorscheme.to_owned())))
        } else {
            None
        };
        let d = if opts.delta.reload {
            Some(spawn(reload_delta(opts.delta, colorscheme.to_owned())))
        } else {
            None
        };
        let m = if opts.cmus.reload {
            Some(spawn(reload_cmus(opts.cmus, colorscheme.to_owned())))
        } else {
            None
        };

        if let Some(a) = a {
            a.await;
        }
        if let Some(k) = k {
            k.await;
        }
        if let Some(t) = t {
            t.await;
        }
        if let Some(n) = n {
            n.await;
        }
        if let Some(s) = s {
            s.await;
        }
        if let Some(d) = d {
            d.await;
        }
        if let Some(m) = m {
            m.await;
        }
    });
}

fn list(dir: impl AsRef<Path>) {
    match alco::list(dir.as_ref()) {
        Ok(files) => {
            for f in files {
                println!("{}", f);
            }
        }
        Err(_) => {
            println!("Error listing files in dir: {}", dir.as_ref().display());
            std::process::exit(1);
        }
    }
}

fn status(scheme_dir: impl AsRef<Path>, time: bool) {
    match alco::status(scheme_dir) {
        Ok(s) => {
            if time {
                let seconds = Duration::from_secs(s.duration.as_secs());
                println!("{} changed {} ago", s.current, humantime::format_duration(seconds),);
            } else {
                println!("{}", s.current);
            }
        }
        Err(e) => println!("Error getting current colorscheme:\n{}", e),
    }
}

async fn reload_alacritty(opts: AlacrittyOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_alacritty(opts.file, opts.in_file, opts.selector, colorscheme) {
        println!("Error reloading alacritty colorscheme:\n{}", e);
    }
}

async fn reload_kitty(opts: KittyOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_kitty(opts.file, opts.socket, opts.selector, colorscheme) {
        println!("Error reloading kitty colorscheme:\n{}", e);
    }
}

async fn reload_tmux(opts: TmuxOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_tmux(opts.file, opts.selector, colorscheme) {
        println!("Error reloading tmux colorscheme:\n{}", e);
    }
}

async fn reload_neovim(command: impl AsRef<str>) {
    if let Err(e) = alco::reload_neovim(command).await {
        println!("Error reloading neovim colorscheme:\n{}", e);
    }
}

async fn reload_starship(opts: StarshipOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_starship(opts.file, opts.in_file, opts.selector, colorscheme) {
        println!("Error reloading starship colorscheme:\n{}", e);
    }
}

async fn reload_delta(opts: DeltaOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_delta(opts.file, opts.selector, colorscheme) {
        println!("Error reloading delta colorscheme:\n{}", e);
    }
}

async fn reload_cmus(opts: CmusOptions, colorscheme: impl AsRef<str>) {
    if let Err(e) = alco::reload_cmus(opts.selector, colorscheme) {
        println!("Error reloading cmus colorscheme:\n{}", e);
    }
}

mod game;
mod map;

use anyhow::{anyhow, Result};
use game::GameBuilder;
use itertools::Itertools;
use piston_window::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    rename_all = "kebab-case",
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DeriveDisplayOrder),
)]
struct Opt {
    /// Width of window in pixels
    #[structopt(short, long, default_value = "900")]
    width: u32,

    /// Height of window in pixels
    #[structopt(short, long, default_value = "600")]
    height: u32,

    /// Theme file (GNOME Mahjongg format)
    #[structopt(
        short,
        long,
        default_value = "/usr/share/gnome-mahjongg/themes/postmodern.svg"
    )]
    theme: PathBuf,

    /// Map file (GNOME Mahjongg format)
    #[structopt(
        short,
        long,
        default_value = "/usr/share/gnome-mahjongg/maps/mahjongg.map"
    )]
    map: PathBuf,

    /// Background color
    #[structopt(short, long, default_value = "#34385b")]
    background: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    anyhow::ensure!(opt.theme.exists(), "Theme file not found");

    let map = {
        let mut maps = if opt.map.exists() {
            map::parse_maps(opt.map)?
        } else {
            eprintln!("Map file not found. Will default to built-in layout.");
            vec![map::default::EASY.clone()]
        };

        match maps.len() {
            0 => unreachable!(),
            1 => maps.swap_remove(0),
            _ => {
                use dialoguer::theme::ColorfulTheme;
                use dialoguer::Select;

                if let Some(selected) = Select::with_theme(&ColorfulTheme::default())
                    .items(&maps.iter().map(|map| &map.name).collect::<Vec<_>>())
                    .default(0)
                    .interact_opt()?
                {
                    maps.swap_remove(selected)
                } else {
                    return Ok(());
                }
            }
        }
    };
    let background_color = parse_color(&opt.background)?;

    let mut window: PistonWindow = WindowSettings::new(&map.name, [opt.width, opt.height])
        .build()
        .map_err(|_| anyhow!("Failed to create window"))?;
    window.set_lazy(true);

    let mut game = GameBuilder::new(&mut window)
        .theme_file(opt.theme)
        .map(map)
        .background_color(&background_color)
        .build()?;

    game.run(&mut window);

    Ok(())
}

fn parse_color(string: &str) -> Result<[f32; 3]> {
    let components: Result<Vec<_>, _> = match string {
        hex if hex.starts_with('#') && hex.len() == 4 => hex
            .chars()
            .skip(1)
            .map(|x| u8::from_str_radix(&format!("{}{}", x, x), 16))
            .collect(),
        hex if hex.starts_with('#') && hex.len() == 7 => hex
            .chars()
            .skip(1)
            .tuples()
            .map(|(a, b)| u8::from_str_radix(&format!("{}{}", a, b), 16))
            .collect(),
        rgb => rgb.split(',').map(|c| c.trim().parse::<u8>()).collect(),
    };

    if let Ok(components) = components {
        if let [r, g, b] = *components {
            return Ok([
                r as f32 / u8::MAX as f32,
                g as f32 / u8::MAX as f32,
                b as f32 / u8::MAX as f32,
            ]);
        }
    }

    Err(anyhow!("Failed to parse color"))
}

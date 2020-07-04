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
    #[structopt(short, long)]
    theme: Option<PathBuf>,

    /// Map file (GNOME Mahjongg format)
    #[structopt(short, long)]
    map: Option<PathBuf>,

    /// Layout name
    #[structopt(short, long)]
    layout: Option<String>,

    /// Background color
    #[structopt(short, long)]
    background: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut window: PistonWindow =
        WindowSettings::new(env!("CARGO_PKG_NAME"), [opt.width, opt.height])
            .build()
            .map_err(|_| anyhow!("Failed to create window"))?;
    window.set_lazy(true);

    let mut builder = GameBuilder::new(&mut window);
    if let Some(theme) = opt.theme {
        builder.theme_file(theme);
    }
    if let Some(map) = opt.map {
        builder.map_file(map);
    }
    if let Some(layout) = opt.layout {
        builder.layout_name(&layout);
    }
    if let Some(background_color) = opt.background {
        let color = parse_color(&background_color)?;
        builder.background_color(&color);
    }

    let mut game = builder.build()?;
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

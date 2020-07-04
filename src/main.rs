mod game;
mod map;

use anyhow::{anyhow, Result};
use game::GameBuilder;
use piston_window::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "mahjongg",
    author = env!("CARGO_PKG_AUTHORS"),
    rename_all = "kebab-case",
)]
struct Opt {
    /// Width of window
    #[structopt(short, long, default_value = "800")]
    width: u32,

    /// Height of window
    #[structopt(short, long, default_value = "600")]
    height: u32,

    /// tileset image file (GNOME Mahjongg format)
    #[structopt(
        short,
        long,
        default_value = "/usr/share/gnome-mahjongg/themes/smooth.png"
    )]
    tileset: PathBuf,

    /// map file (GNOME Mahjongg format)
    #[structopt(
        short,
        long,
        default_value = "/usr/share/gnome-mahjongg/maps/mahjongg.map"
    )]
    map_file: PathBuf,

    /// map name
    #[structopt(short = "n", long, default_value = "easy")]
    map_name: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut window: PistonWindow = WindowSettings::new("Mahjongg", [opt.width, opt.height])
        .exit_on_esc(true)
        .build()
        .map_err(|_| anyhow!("Failed to create window"))?;
    window.set_lazy(true);

    let mut game = GameBuilder::new(&mut window)
        .tileset_file(opt.tileset)
        .map_file(opt.map_file)
        .map_name(&opt.map_name)
        .build()?;

    game.run(&mut window);

    Ok(())
}

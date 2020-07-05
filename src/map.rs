pub mod default;
mod gnome_mahjongg;
mod kmahjongg;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Slot {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub name: String,
    pub slots: Vec<Slot>,
    pub width: usize,
    pub height: usize,
}

pub fn load_from_paths<P: AsRef<Path>>(paths: &[P]) -> Vec<Map> {
    let mut maps = vec![default::EASY.clone()];

    for path in paths {
        if path.as_ref().is_file() {
            if let Ok(mut loaded) = load_maps(path) {
                maps.append(&mut loaded);
            }
        } else if path.as_ref().is_dir() {
            if let Ok(rd) = path.as_ref().read_dir() {
                for entry in rd {
                    if let Ok(entry) = entry {
                        if let Ok(mut loaded) = load_maps(entry.path()) {
                            maps.append(&mut loaded);
                        }
                    }
                }
            }
        }
    }

    maps
}

pub fn load_maps<P: AsRef<Path>>(path: P) -> Result<Vec<Map>> {
    let ext = path
        .as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    if let Some(ext) = ext {
        match &ext[..] {
            "map" => return gnome_mahjongg::load(path),
            "desktop" => return kmahjongg::load(path),
            _ => (),
        }
    }

    Err(anyhow::anyhow!("Not a map file"))
}

fn calc_size(slots: &[Slot]) -> (usize, usize) {
    let mut width = 0;
    let mut height = 0;
    for slot in slots {
        width = width.max(slot.x);
        height = height.max(slot.y);
    }

    (width as usize + 2, height as usize + 2)
}

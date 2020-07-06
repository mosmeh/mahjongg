use super::{Map, Slot};
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<Map>> {
    let (name, filename) = parse_desktop_file(path)?;

    let slots = parse_layout_file(filename)?;
    let (width, height) = super::calc_size(&slots);

    let map = Map {
        name,
        slots,
        width,
        height,
    };
    Ok(vec![map])
}

fn parse_desktop_file<P: AsRef<Path>>(path: P) -> Result<(String, PathBuf)> {
    const LAYOUT_VERSION_FORMAT: i32 = 1;

    let desktop = ini::Ini::load_from_file(&path)?;
    let section = desktop
        .section(Some("KMahjonggLayout"))
        .ok_or_else(|| anyhow!("Invalid map format"))?;

    if let Some(version) = section.get("VersionFormat") {
        if version.parse::<i32>().unwrap_or(0) > LAYOUT_VERSION_FORMAT {
            return Err(anyhow!("Unsupported version"));
        }
    }

    let name = section
        .get("Name")
        .ok_or_else(|| anyhow!("Invalid map: no map name"))?
        .to_string();
    let filename = section
        .get("FileName")
        .ok_or_else(|| anyhow!("Invalid map: no layout filename"))?;

    // safe to unwrap, or reading desktop file would have failed
    let filename = path.as_ref().parent().unwrap().join(filename);

    Ok((name, filename))
}

fn parse_layout_file<P: AsRef<Path>>(path: P) -> Result<Vec<Slot>> {
    let mut lines = BufReader::new(File::open(path)?).lines();

    let magic = lines
        .next()
        .ok_or_else(|| anyhow!("Invalid layout: empty file"))??;

    let (data, width, height, depth) = match &magic[..] {
        "kmahjongg-layout-v1.0" => {
            let mut data = String::new();

            for line in lines {
                let line = line?;
                if line.as_bytes()[0] != b'#' {
                    data.push_str(&line);
                }
            }

            (data, 32, 16, 5)
        }
        "kmahjongg-layout-v1.1" => {
            let mut data = String::new();
            let mut width = 0;
            let mut height = 0;
            let mut depth = 0;

            for line in lines {
                let line = line?;
                match line.as_bytes()[0] {
                    b'w' => width = line.trim_start_matches('w').parse()?,
                    b'h' => height = line.trim_start_matches('h').parse()?,
                    b'd' => depth = line.trim_start_matches('d').parse()?,
                    b'#' => (),
                    _ => data.push_str(&line),
                }
            }

            (data, width, height, depth)
        }
        _ => return Err(anyhow!("Unsupported layout format")),
    };

    if width == 0 || height == 0 || depth == 0 || data.len() != width * height * depth {
        return Err(anyhow!("Invalid layout"));
    }

    let mut x = 0;
    let mut y = 0;
    let mut z = 0;
    let mut slots = Vec::new();

    for c in data.as_bytes() {
        if *c == b'1' {
            slots.push(Slot { x, y, z });
        }

        x += 1;
        if x == width as isize {
            x = 0;
            y += 1;
            if y == height as isize {
                y = 0;
                z += 1;
                if z == depth as isize {
                    if slots.len() % 2 != 0 {
                        break;
                    }

                    return Ok(slots);
                }
            }
        }
    }

    Err(anyhow!("Invalid layout"))
}

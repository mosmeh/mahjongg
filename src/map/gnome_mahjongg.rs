use super::{Map, Slot};
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<Map>> {
    let reader = BufReader::new(File::open(path)?);
    let mahjongg: markup::Mahjongg = serde_xml_rs::from_reader(reader)?;

    let mut maps = Vec::with_capacity(mahjongg.maps.len());

    for map in mahjongg.maps {
        let mut slots = Vec::new();
        for item in map.items {
            parse_items(&mut slots, item, 0);
        }

        if slots.len() % 2 != 0 {
            return Err(anyhow::anyhow!("Invalid map"));
        }

        let (width, height) = super::calc_size(&slots);
        maps.push(Map {
            name: map.name,
            slots,
            width,
            height,
        });
    }

    Ok(maps)
}

fn parse_items(slots: &mut Vec<Slot>, item: markup::Item, layer_z: isize) {
    use markup::{Item, Layer};

    match item {
        Item::Layer(Layer { z, items }) => {
            for child in items {
                parse_items(slots, child, z);
            }
        }
        Item::Row { left, right, y, z } => {
            let z = z.unwrap_or(layer_z);
            for x in (left..=right).step_by(2) {
                slots.push(Slot { x, y, z });
            }
        }
        Item::Column { x, top, bottom, z } => {
            let z = z.unwrap_or(layer_z);
            for y in (top..=bottom).step_by(2) {
                slots.push(Slot { x, y, z });
            }
        }
        Item::Block {
            left,
            right,
            top,
            bottom,
            z,
        } => {
            let z = z.unwrap_or(layer_z);
            for x in (left..=right).step_by(2) {
                for y in (top..=bottom).step_by(2) {
                    slots.push(Slot { x, y, z })
                }
            }
        }
        Item::Tile { x, y, z } => {
            let z = z.unwrap_or(layer_z);
            slots.push(Slot { x, y, z });
        }
    }
}

mod markup {
    use serde::{Deserialize, Deserializer};

    #[derive(Debug, Deserialize)]
    pub struct Mahjongg {
        #[serde(rename = "map")]
        pub maps: Vec<Map>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Map {
        pub name: String,
        #[serde(rename = "$value")]
        pub items: Vec<Item>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Layer {
        #[serde(default)]
        pub z: isize,
        #[serde(rename = "$value")]
        pub items: Vec<Item>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Item {
        Layer(Layer),
        Row {
            #[serde(default, deserialize_with = "deserialize_pos")]
            left: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            right: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            y: isize,
            z: Option<isize>,
        },
        Column {
            #[serde(default, deserialize_with = "deserialize_pos")]
            x: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            top: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            bottom: isize,
            z: Option<isize>,
        },
        Block {
            #[serde(default, deserialize_with = "deserialize_pos")]
            left: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            right: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            top: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            bottom: isize,
            z: Option<isize>,
        },
        Tile {
            #[serde(default, deserialize_with = "deserialize_pos")]
            x: isize,
            #[serde(default, deserialize_with = "deserialize_pos")]
            y: isize,
            z: Option<isize>,
        },
    }

    fn deserialize_pos<'de, D>(deserializer: D) -> Result<isize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        if string.ends_with(".5") {
            string
                .trim_end_matches(".5")
                .parse()
                .map(|x: isize| x * 2 + 1)
                .map_err(serde::de::Error::custom)
        } else {
            string
                .parse()
                .map(|x: isize| x * 2)
                .map_err(serde::de::Error::custom)
        }
    }
}

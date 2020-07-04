mod easy;

use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod xml {
    use serde::{Deserialize, Deserializer};

    #[derive(Deserialize)]
    pub struct MapDef {
        #[serde(rename = "map")]
        pub maps: Vec<Map>,
    }

    #[derive(Deserialize)]
    pub struct Map {
        pub name: String,
        pub scorename: String,
        #[serde(rename = "layer")]
        pub layers: Vec<Layer>,
    }

    #[derive(Deserialize)]
    pub struct Layer {
        pub z: usize,
        #[serde(rename = "$value")]
        pub items: Vec<Item>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Item {
        Row {
            #[serde(deserialize_with = "deserialize_pos")]
            left: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            right: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            y: usize,
        },
        Column {
            #[serde(deserialize_with = "deserialize_pos")]
            x: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            top: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            bottom: usize,
        },
        Block {
            #[serde(deserialize_with = "deserialize_pos")]
            left: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            right: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            top: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            bottom: usize,
        },
        Tile {
            #[serde(deserialize_with = "deserialize_pos")]
            x: usize,
            #[serde(deserialize_with = "deserialize_pos")]
            y: usize,
        },
    }

    fn deserialize_pos<'de, D>(deserializer: D) -> Result<usize, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        if string.ends_with(".5") {
            string
                .trim_end_matches(".5")
                .parse()
                .map(|x: usize| x * 2 + 1)
                .map_err(serde::de::Error::custom)
        } else {
            string
                .parse()
                .map(|x: usize| x * 2)
                .map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub slots: Vec<Slot>,
    pub width: usize,
    pub height: usize,
}

fn calc_size(slots: &[Slot]) -> (usize, usize) {
    let mut width = 0;
    let mut height = 0;
    for slot in slots {
        width = width.max(slot.x);
        height = height.max(slot.y);
    }

    (width + 2, height + 2)
}

pub fn parse_maps<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Map>> {
    use xml::Item::*;

    let reader = BufReader::new(File::open(path)?);
    let def: xml::MapDef = serde_xml_rs::from_reader(reader)?;

    let mut hashmap: HashMap<String, Map> = HashMap::new();
    hashmap.insert(easy::NAME.to_string(), easy::MAP.clone());

    for map in def.maps {
        let mut slots = Vec::new();

        for layer in map.layers {
            for item in layer.items {
                match item {
                    Row { left, right, y } => {
                        for x in (left..=right).step_by(2) {
                            slots.push(Slot { x, y, z: layer.z });
                        }
                    }
                    Column { x, top, bottom } => {
                        for y in (top..=bottom).step_by(2) {
                            slots.push(Slot { x, y, z: layer.z });
                        }
                    }
                    Block {
                        left,
                        right,
                        top,
                        bottom,
                    } => {
                        for x in (left..=right).step_by(2) {
                            for y in (top..=bottom).step_by(2) {
                                slots.push(Slot { x, y, z: layer.z })
                            }
                        }
                    }
                    Tile { x, y } => slots.push(Slot { x, y, z: layer.z }),
                }
            }
        }

        let name = map.scorename;
        let (width, height) = calc_size(&slots);
        let map = Map {
            slots,
            width,
            height,
        };
        hashmap.insert(name, map);
    }

    Ok(hashmap)
}

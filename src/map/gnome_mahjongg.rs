use super::{Map, Slot};
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn load<P: AsRef<Path>>(path: P) -> Result<Vec<Map>> {
    use xml::Item::*;

    let reader = BufReader::new(File::open(path)?);
    let def: xml::MapDef = serde_xml_rs::from_reader(reader)?;

    let mut maps = Vec::with_capacity(def.maps.len());

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
        pub z: isize,
        #[serde(rename = "$value")]
        pub items: Vec<Item>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Item {
        Row {
            #[serde(deserialize_with = "deserialize_pos")]
            left: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            right: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            y: isize,
        },
        Column {
            #[serde(deserialize_with = "deserialize_pos")]
            x: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            top: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            bottom: isize,
        },
        Block {
            #[serde(deserialize_with = "deserialize_pos")]
            left: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            right: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            top: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            bottom: isize,
        },
        Tile {
            #[serde(deserialize_with = "deserialize_pos")]
            x: isize,
            #[serde(deserialize_with = "deserialize_pos")]
            y: isize,
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

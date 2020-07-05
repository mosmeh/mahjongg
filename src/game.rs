use super::map::{self, Map, Slot};
use anyhow::{anyhow, Result};
use piston_window::*;
use rand::prelude::*;
use std::path::{Path, PathBuf};

pub struct Game {
    map: Map,
    background_color: [f32; 4],
    tiles: Vec<Tile>,
    theme_texture: G2dTexture,
    selected: Option<usize>,
    cursor_pos: Option<[f64; 2]>,
    history: Vec<(usize, usize)>,
}

impl Game {
    pub fn run(&mut self, mut window: &mut PistonWindow) {
        while let Some(event) = window.next() {
            if event.render_args().is_some() {
                self.draw(&mut window, &event);
            }

            if let Some(pos) = event.mouse_cursor_args() {
                self.on_mouse_cursor(pos);
            }

            if let Some(button) = event.press_args() {
                match button {
                    Button::Mouse(MouseButton::Left) => self.on_left_click(&mut window),
                    Button::Mouse(MouseButton::Right) => self.on_right_click(),
                    _ => (),
                }
            }
        }
    }

    fn draw(&self, window: &mut PistonWindow, event: &Event) {
        let geometry = self.calc_geometry(window.draw_size());
        let width = geometry.image_size.width;
        let height = geometry.image_size.height;

        window.draw_2d(event, |c, g, _| {
            clear(self.background_color, g);

            for (i, tile) in self.tiles.iter().enumerate() {
                if !tile.visible {
                    continue;
                }

                let pos = geometry.calc_tile_pos(&tile.slot);

                let draw_state =
                    c.draw_state
                        .scissor([pos.x as u32, pos.y as u32, width as u32, height as u32]);

                let texture_x = get_image_offset(tile.id) as f64 * width;
                let texture_y = self
                    .selected
                    .map(|s| if s == i { height } else { 0.0 })
                    .unwrap_or(0.0);
                let transform = c
                    .transform
                    .trans(pos.x as f64 - texture_x, pos.y as f64 - texture_y)
                    .scale(geometry.image_scale.width, geometry.image_scale.height);

                Image::new().draw(&self.theme_texture, &draw_state, transform, g);
            }
        });
    }

    fn on_mouse_cursor(&mut self, pos: [f64; 2]) {
        self.cursor_pos = Some(pos);
    }

    fn on_left_click(&mut self, window: &mut PistonWindow) {
        let pointed = if let Some(pointed) = self.get_pointed_tile_index(window.draw_size()) {
            pointed
        } else {
            // points to nothing
            return;
        };

        if !tile_is_exposed(pointed, &self.tiles) {
            // pointed tile is not removable
            return;
        }

        let prev = if let Some(prev) = self.selected {
            prev
        } else {
            // select first of pair
            self.selected = Some(pointed);
            return;
        };

        if pointed == prev {
            // cancel selection
            self.selected = None;
            return;
        }

        if self.tiles[pointed].matches(&self.tiles[prev]) {
            // remove tile

            self.tiles[prev].visible = false;
            self.tiles[pointed].visible = false;
            self.history.push((prev, pointed));

            self.selected = None;
            return;
        }

        // select another tile
        self.selected = Some(pointed);
    }

    fn on_right_click(&mut self) {
        // undo
        if let Some(last) = self.history.pop() {
            self.tiles[last.0].visible = true;
            self.tiles[last.1].visible = true;

            self.selected = None;
        }
    }

    fn get_pointed_tile_index(&self, draw_size: Size) -> Option<usize> {
        let cursor_pos = self.cursor_pos?;

        let geometry = self.calc_geometry(draw_size);

        let width = geometry.image_size.width;
        let height = geometry.image_size.height;

        let mut pointed: Option<usize> = None;
        for (i, tile) in self.tiles.iter().enumerate() {
            if !tile.visible {
                continue;
            }

            let pos = geometry.calc_tile_pos(&tile.slot);

            if pos.x as f64 <= cursor_pos[0]
                && cursor_pos[0] < pos.x as f64 + width
                && pos.y as f64 <= cursor_pos[1]
                && cursor_pos[1] < pos.y as f64 + height
            {
                pointed = Some(i);
            }
        }

        pointed
    }

    fn calc_geometry(&self, draw_size: Size) -> Geometry {
        let theme_size = self.theme_texture.get_size();
        let theme_aspect = (theme_size.1 as f64 / 2.0) / (theme_size.0 as f64 / 43.0);
        let map_size = Size::from([
            (self.map.width + 2) as f64,
            (self.map.height + 2) as f64 * theme_aspect,
        ]);

        let unit_width = (draw_size.width as f64 / map_size.width)
            .min(draw_size.height as f64 / map_size.height);
        let unit_height = unit_width * theme_aspect;

        let tile_size = Size::from([unit_width * 2.0, unit_height * 2.0]);
        let tile_layer_offset = Position::from([
            (tile_size.width / 7.0) as i32,
            (tile_size.height / 10.0) as i32,
        ]);

        let offset = Position::from([
            ((draw_size.width as f64 - self.map.width as f64 * unit_width) / 2.0) as i32,
            ((draw_size.height as f64 - self.map.height as f64 * unit_height) / 2.0) as i32,
        ]);

        let image_size = Size::from([
            tile_size.width + tile_layer_offset.x as f64,
            tile_size.height + tile_layer_offset.y as f64,
        ]);

        let image_scale = Size::from([
            (image_size.width * 43.0) / theme_size.0 as f64,
            (image_size.height * 2.0) / theme_size.1 as f64,
        ]);

        Geometry {
            offset,
            tile_size,
            tile_layer_offset,
            image_size,
            image_scale,
        }
    }
}

#[derive(Debug)]
struct Tile {
    id: usize,
    slot: Slot,
    visible: bool,
}

impl Tile {
    fn matches(&self, other: &Tile) -> bool {
        self.id / 4 == other.id / 4
    }
}

fn tile_is_exposed(index: usize, tiles: &[Tile]) -> bool {
    if !tiles[index].visible {
        return false;
    }

    let slot = &tiles[index].slot;

    let mut blocked_left = false;
    let mut blocked_right = false;
    for (i, tile) in tiles.iter().enumerate() {
        if i == index || !tile.visible {
            continue;
        }

        if tile.slot.z == slot.z + 1
            && tile.slot.x >= slot.x - 1
            && tile.slot.x <= slot.x + 1
            && tile.slot.y >= slot.y - 1
            && tile.slot.y <= slot.y + 1
        {
            return false;
        }

        if tile.slot.z == slot.z && tile.slot.y >= slot.y - 1 && tile.slot.y <= slot.y + 1 {
            if tile.slot.x == slot.x - 2 {
                blocked_left = true;
            }
            if tile.slot.x == slot.x + 2 {
                blocked_right = true;
            }
            if blocked_left && blocked_right {
                return false;
            }
        }
    }

    true
}

struct Geometry {
    offset: Position,
    tile_size: Size,
    tile_layer_offset: Position,
    image_size: Size,
    image_scale: Size,
}

impl Geometry {
    fn calc_tile_pos(&self, slot: &Slot) -> Position {
        Position::from([
            self.offset.x
                + (slot.x as f64 * self.tile_size.width / 2.0) as i32
                + slot.z as i32 * self.tile_layer_offset.x,
            self.offset.y + (slot.y as f64 * self.tile_size.height / 2.0) as i32
                - slot.z as i32 * self.tile_layer_offset.y,
        ])
    }
}

fn get_image_offset(id: usize) -> usize {
    let set = id / 4;

    // Invalid ids
    if set >= 36 {
        unreachable!()
    }

    /* The bonus tiles have different images for each */
    if set == 33 {
        33 + id % 4
    } else if set == 35 {
        38 + id % 4
    } else if set == 34 {
        /* The white dragons are inbetween the bonus tiles just to be confusing */
        37
    } else {
        /* Everything else is in set order */
        set
    }
}

pub struct GameBuilder<'a> {
    window: &'a mut PistonWindow,
    theme_file: Option<PathBuf>,
    map: Map,
    background_color: [f32; 3],
}

impl<'a> GameBuilder<'a> {
    pub fn new(window: &'a mut PistonWindow) -> Self {
        Self {
            window,
            theme_file: None,
            map: map::default::EASY.clone(),
            background_color: [52.0 / 255.0, 56.0 / 255.0, 91.0 / 255.0],
        }
    }

    pub fn build(mut self) -> Result<Game> {
        let theme_file = self
            .theme_file
            .ok_or_else(|| anyhow!("Theme file not provided"))?;

        let theme_texture = if let Ok(buf) = render_svg(&theme_file) {
            Texture::from_image(
                &mut self.window.create_texture_context(),
                &buf,
                &TextureSettings::new(),
            )
            .map_err(|_| anyhow!("Failed to load texture"))?
        } else {
            Texture::from_path(
                &mut self.window.create_texture_context(),
                &theme_file,
                Flip::None,
                &TextureSettings::new(),
            )
            .map_err(|_| anyhow!("Failed to load texture"))?
        };

        // sort by draw order
        self.map
            .slots
            .sort_unstable_by(|a, b| a.z.cmp(&b.z).then_with(|| (a.y - b.y).cmp(&(a.x - b.x))));

        let mut tiles: Vec<_> = self
            .map
            .slots
            .iter()
            .map(|slot| Tile {
                id: 0,
                slot: slot.clone(),
                visible: true,
            })
            .collect();

        let mut rng = rand::thread_rng();
        fill_random_ids(&mut tiles, &mut rng)?;

        let game = Game {
            map: self.map,
            background_color: [
                self.background_color[0],
                self.background_color[1],
                self.background_color[2],
                1.0,
            ],
            tiles,
            theme_texture,
            selected: None,
            cursor_pos: None,
            history: Vec::new(),
        };
        Ok(game)
    }

    pub fn theme_file<P: AsRef<Path>>(mut self, theme_file: P) -> Self {
        self.theme_file = Some(theme_file.as_ref().to_path_buf());
        self
    }

    pub fn map(mut self, map: Map) -> Self {
        self.map = map;
        self
    }

    pub fn background_color(mut self, background_color: &[f32; 3]) -> Self {
        self.background_color = *background_color;
        self
    }
}

/// Generates random solvable configuration
fn fill_random_ids(tiles: &mut [Tile], rng: &mut ThreadRng) -> Result<()> {
    // it is based on the behavior of KMahjongg, not GNOME Mahjongg
    let pairs: Vec<usize> = std::iter::repeat_with(|| {
        let mut pairs: Vec<usize> = (0..144 / 2).collect();
        pairs.shuffle(rng);
        pairs
    })
    .flatten()
    .take(tiles.len() / 2)
    .collect();

    // GNOME Mahjongg version is:
    // let mut pairs: Vec<usize> = (0..tiles.len() / 2).collect();
    // pairs.shuffle(rng);
    // but it doesn't support #tiles > 144 and has more biased tile distribution

    let succeeded = fill_random_ids_impl(tiles, &pairs, 0, rng);
    for tile in tiles {
        tile.visible = true;
    }

    if succeeded {
        Ok(())
    } else {
        Err(anyhow!("No solvable configuration"))
    }
}

fn fill_random_ids_impl(
    tiles: &mut [Tile],
    pairs: &[usize],
    depth: usize,
    rng: &mut ThreadRng,
) -> bool {
    if depth == pairs.len() {
        return true;
    }

    let mut matches = find_all_matches(tiles);
    if matches.is_empty() {
        return false;
    }
    matches.shuffle(rng);
    for m in matches {
        tiles[m.0].id = 2 * pairs[depth];
        tiles[m.0].visible = false;
        tiles[m.1].id = 2 * pairs[depth] + 1;
        tiles[m.1].visible = false;

        if fill_random_ids_impl(tiles, pairs, depth + 1, rng) {
            return true;
        }

        tiles[m.0].id = 0;
        tiles[m.0].visible = true;
        tiles[m.1].id = 0;
        tiles[m.1].visible = true;
    }

    false
}

#[derive(Debug, Copy, Clone)]
struct Match(usize, usize);

impl PartialEq for Match {
    fn eq(&self, other: &Match) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl Eq for Match {}

impl std::hash::Hash for Match {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.0 < self.1 {
            self.0.hash(state);
            self.1.hash(state);
        } else {
            self.1.hash(state);
            self.0.hash(state);
        }
    }
}

fn find_all_matches(tiles: &[Tile]) -> Vec<Match> {
    let mut set = std::collections::HashSet::new();
    for i in 0..tiles.len() {
        if !tile_is_exposed(i, tiles) {
            continue;
        }
        for m in find_matches(i, tiles) {
            set.insert(m);
        }
    }
    set.drain().collect()
}

fn find_matches<'a>(index: usize, tiles: &'a [Tile]) -> impl Iterator<Item = Match> + 'a {
    (0..tiles.len())
        .filter(move |i| {
            *i != index
                && tiles[*i].visible
                && tiles[*i].matches(&tiles[index])
                && tile_is_exposed(*i, tiles)
        })
        .map(move |i| Match(i, index))
}

fn render_svg<P: AsRef<Path>>(path: P) -> Result<::image::RgbaImage> {
    let tree = usvg::Tree::from_file(path, &usvg::Options::default())?;
    let image = resvg::render(&tree, usvg::FitTo::Original, None)
        .ok_or_else(|| anyhow!("Failed to render SVG"))?;
    let buf = ::image::RgbaImage::from_raw(image.width(), image.height(), image.data().to_vec())
        .ok_or_else(|| anyhow!("Failed to construct image buffer from rendered SVG"))?;

    Ok(buf)
}

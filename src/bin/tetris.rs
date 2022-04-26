use icarus::color;
use icarus::input::{ButtonId, InputState, KeyId};
use icarus::math::Rect;
use icarus::platform::{Config, Platform};
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;
const TILES_X: usize = 10;
const TILES_Y: usize = 20;
const TILE_COUNT: usize = TILES_X * TILES_Y;
const TILE_SIZE: f32 = 30.0;

#[derive(Default, Copy, Clone)]
struct Tile {
    pos: (usize, usize),
    color: color::Color,
}
impl Tile {
    fn new(x: usize, y: usize, color: color::Color) -> Self {
        Self {
            pos: (x, y),
            color,
        }
    }
}

#[derive(Default)]
struct Piece {
    tiles: Vec<Tile>,
}

#[derive(Default)]
struct Game {
    paused: bool,
    tiles: Vec<Tile>,
    piece: Piece,
    timer: f32, // Goes from 0.0 to time_per_tile and then resets
    time_per_tile_sec: f32,
}
impl Game {
    fn init() -> Self {
        let mut tiles = vec![];
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
                tiles.push(Tile::new(col, row, color::DARK_GREY));
            }
        }
        Self {
            time_per_tile_sec: 0.5,
            tiles,
            piece: Self::spawn_piece(),
            ..Self::default()
        }
    }
    fn update(&mut self, input: &InputState, dt: f32) {
        if input.was_key_pressed(KeyId::P) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }

        self.timer += dt;
        if self.timer >= self.time_per_tile_sec {
            self.timer -= self.time_per_tile_sec;
            for tile in &mut self.piece.tiles {
                tile.pos.1 += 1;
                tile.pos.1 = tile.pos.1.min(TILES_Y - 1);
            }
            // Did we reach the bottom?
            if self.piece.tiles.iter().any(
                |Tile {
                     pos: (_, y),
                     ..
                 }| *y == TILES_Y - 1,
            ) {
                for tile in &mut self.piece.tiles {
                    let idx = pos_to_idx(tile.pos.0, tile.pos.1);
                    //println!("pos: ({},{}), idx: {}", tile.pos.0, tile.pos.1, idx);
                    self.tiles[idx].color = tile.color;
                }
                // Spawn a new piece
                self.piece = Self::spawn_piece();
            }
        }

        if input.was_key_pressed(KeyId::A) {
            if !self.piece.tiles.iter().any(
                |Tile {
                     pos: (x, y),
                     ..
                 }| *x == 0 || *y == TILES_Y - 1,
            ) {
                for tile in &mut self.piece.tiles {
                    if tile.pos.0 > 0 {
                        tile.pos.0 -= 1;
                    }
                }
            }
        }
        if input.was_key_pressed(KeyId::D) {
            if !self.piece.tiles.iter().any(
                |Tile {
                     pos: (x, y),
                     ..
                 }| *x == TILES_X - 1 || *y == TILES_Y - 1,
            ) {
                for tile in &mut self.piece.tiles {
                    tile.pos.0 += 1;
                    tile.pos.0 = tile.pos.0.min(TILES_X - 1);
                }
            }
        }
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        let start_x = WIDTH / 2.0 - TILE_SIZE * (TILES_X / 2) as f32;
        let start_y = HEIGHT / 2.0 - TILE_SIZE * (TILES_Y / 2) as f32;
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
                let idx = pos_to_idx(col, row);
                let tile = self.tiles[idx];
                vk_util::push_rect_color(
                    cmd,
                    Rect::offset_extent(
                        (start_x + col as f32 * TILE_SIZE, start_y + row as f32 * TILE_SIZE),
                        (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                    ),
                    0.1,
                    tile.color,
                );
            }
        }

        for tile in &self.piece.tiles {
            vk_util::push_rect_color(
                cmd,
                Rect::offset_extent(
                    (start_x + tile.pos.0 as f32 * TILE_SIZE, start_y + tile.pos.1 as f32 * TILE_SIZE),
                    (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                ),
                0.0,
                tile.color,
            );
        }
    }

    fn spawn_piece() -> Piece {
        Piece {
            tiles: vec![
                Tile::new(3, 0, color::BLUE),
                Tile::new(4, 0, color::BLUE),
                Tile::new(5, 0, color::BLUE),
                Tile::new(6, 0, color::BLUE),
            ],
        }
    }
}

fn pos_to_idx(x: usize, y: usize) -> usize {
    y * TILES_X + x
}
fn idx_to_pos(idx: usize) -> (usize, usize) {
    let x = idx % TILES_X;
    let y = idx / TILES_X;
    (x, y)
}

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Tetris"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(&platform, mem::size_of::<RenderCommand>() * MAX_ENTITIES, None);

    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    loop {
        platform.process_messages(&mut input);
        if input.was_key_pressed(KeyId::Esc) {
            break;
        }

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);

        let mut cmd = vec![];
        game.render(&mut cmd);

        vk_ctx.render(&cmd, None, &[], &[]);
    }
}

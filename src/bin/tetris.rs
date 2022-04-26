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

#[derive(Default)]
struct Piece {
    tiles: Vec<(usize, usize)>, // x, y
    color: color::Color,
}

#[derive(Default)]
struct Game {
    tiles: Vec<bool>,
    piece: Piece,
    timer: f32, // Goes from 0.0 to time_per_tile and then resets
    time_per_tile_sec: f32,
}
impl Game {
    fn init() -> Self {
        Self {
            time_per_tile_sec: 0.5,
            tiles: vec![true; TILE_COUNT],
            piece: Piece {
                tiles: vec![(3, 0), (4, 0), (5, 0), (6, 0)],
                color: color::BLUE,
            },
            ..Self::default()
        }
    }
    fn update(&mut self, input: &InputState, dt: f32) {
        self.timer += dt;
        if self.timer >= self.time_per_tile_sec {
            self.timer -= self.time_per_tile_sec;
            for tile in &mut self.piece.tiles {
                tile.1 += 1;
                tile.1 = tile.1.min(TILES_Y - 1);
            }
        }

        if input.was_key_pressed(KeyId::A) {
            if !self.piece.tiles.iter().any(|(x, y)| *x == 0 || *y == TILES_Y - 1) {
                for tile in &mut self.piece.tiles {
                    if tile.0 > 0 {
                        tile.0 -= 1;
                    }
                }
            }
        }
        if input.was_key_pressed(KeyId::D) {
            if !self.piece.tiles.iter().any(|(x, y)| *x == TILES_X - 1 || *y == TILES_Y - 1) {
                for tile in &mut self.piece.tiles {
                    tile.0 += 1;
                    tile.0 = tile.0.min(TILES_X - 1);
                }
            }
        }

        if input.was_button_pressed(ButtonId::Left) {
            let button = input.buttons[ButtonId::Left as usize];

            let start_x = WIDTH / 2.0 - TILE_SIZE * (TILES_X / 2) as f32;
            let start_y = HEIGHT / 2.0 - TILE_SIZE * (TILES_Y / 2) as f32;
            for row in 0..TILES_Y {
                for col in 0..TILES_X {
                    let idx = row * TILES_X + col;
                    let tile_rect = Rect::offset_extent(
                        (start_x + col as f32 * TILE_SIZE, start_y + row as f32 * TILE_SIZE),
                        (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                    );
                    if tile_rect.is_inside((button.x as f32, button.y as f32)) {
                        self.tiles[idx] = false;
                    }
                }
            }
        }
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        let start_x = WIDTH / 2.0 - TILE_SIZE * (TILES_X / 2) as f32;
        let start_y = HEIGHT / 2.0 - TILE_SIZE * (TILES_Y / 2) as f32;
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
                let idx = row * TILES_X + col;
                if self.tiles[idx] {
                    vk_util::push_rect_color(
                        cmd,
                        Rect::offset_extent(
                            (start_x + col as f32 * TILE_SIZE, start_y + row as f32 * TILE_SIZE),
                            (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                        ),
                        0.1,
                        color::DARK_GREY,
                    );
                }
            }
        }

        for (x, y) in &self.piece.tiles {
            vk_util::push_rect_color(
                cmd,
                Rect::offset_extent(
                    (start_x + *x as f32 * TILE_SIZE, start_y + *y as f32 * TILE_SIZE),
                    (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                ),
                0.0,
                self.piece.color,
            );
        }
    }
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

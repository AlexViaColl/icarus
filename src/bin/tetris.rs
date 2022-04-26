use icarus::color;
use icarus::input::{InputState, KeyId};
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
const TILE_SIZE: f32 = 30.0;

#[derive(Default)]
struct Game {}
impl Game {
    fn init() -> Self {
        Self::default()
    }
    fn update(&mut self, _input_state: &InputState, _dt: f32) {}
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        let start_x = WIDTH / 2.0 - TILE_SIZE * (TILES_X / 2) as f32;
        let start_y = HEIGHT / 2.0 - TILE_SIZE * (TILES_Y / 2) as f32;
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
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

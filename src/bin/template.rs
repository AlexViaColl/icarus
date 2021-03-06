use icarus::color;
use icarus::input::{InputState, KeyId};
use icarus::math::Rect;
use icarus::platform::{Config, Platform};
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::time::Instant;

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

struct Game {}
impl Game {
    fn init() -> Self {
        Self {}
    }
    fn update(&mut self, _input: &InputState, _dt: f32) {}
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        vk_util::push_rect_color(
            cmd,
            Rect::center_extent((WIDTH / 2.0, HEIGHT / 2.0), (200.0, 100.0)),
            0.0,
            color::GREEN,
        );
    }
}

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Template"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(&platform);

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

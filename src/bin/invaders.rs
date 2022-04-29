use icarus::color;
use icarus::color::Color;
use icarus::input::{InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;

const PLAYER_SPEED: f32 = 1000.0;
const PLAYER_WIDTH: f32 = 150.0;
const PLAYER_HEIGHT: f32 = 50.0;

const BULLET_SPEED: f32 = 1000.0;
const BULLET_WIDTH: f32 = 5.0;
const BULLET_HEIGHT: f32 = 20.0;

const BG_COLOR: Color = color!(rgb8(28, 28, 28));

#[derive(Default)]
struct Bullet {
    pos: Vec2,
}

#[derive(Default)]
struct Game {
    player: Vec2,
    player_vel: f32,
    bullets: Vec<Bullet>,
}
impl Game {
    fn init() -> Self {
        Self {
            player: Vec2::new(WIDTH / 2.0, HEIGHT - PLAYER_HEIGHT / 2.0),
            ..Self::default()
        }
    }
    fn update(&mut self, input: &InputState, dt: f32) {
        self.player_vel = 0.0;
        if input.is_key_down(KeyId::A) {
            self.player_vel = -PLAYER_SPEED;
        }
        if input.is_key_down(KeyId::D) {
            self.player_vel = PLAYER_SPEED;
        }

        if input.was_key_pressed(KeyId::Space) {
            self.bullets.push(Bullet {
                pos: self.player,
            });
        }

        self.player = self.player + Vec2::new(self.player_vel, 0.0) * dt;

        for bullet in &mut self.bullets {
            bullet.pos = bullet.pos + Vec2::new(0.0, -BULLET_SPEED) * dt;
        }
        self.bullets.retain(|b| b.pos.y > 0.0);
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        vk_util::push_rect(cmd, Rect::center_extent(self.player, (PLAYER_WIDTH, PLAYER_HEIGHT)), 0.9);

        for bullet in &self.bullets {
            vk_util::push_rect(cmd, Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT)), 0.9);
        }
    }
}

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Invaders"),
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
        vk_ctx.render(&cmd, Some(BG_COLOR), &[], &[]);
    }
}

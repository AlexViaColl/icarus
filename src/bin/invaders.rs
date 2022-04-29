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
const BULLET_HEIGHT: f32 = 10.0;

const ENEMY_ROWS: usize = 5;
const ENEMY_COLS: usize = 11;
const ENEMY_WIDTH: f32 = 40.0;
const ENEMY_HEIGHT: f32 = 32.0;

const BG_COLOR: Color = color!(rgb8(28, 28, 28));

#[derive(Default)]
struct Bullet {
    pos: Vec2,
}

#[derive(Default)]
struct Enemy {
    pos: Vec2,
    dead: bool,
}

#[derive(Default)]
struct Timer {
    elapsed: f32,
    duration: f32,
}

#[derive(Default)]
struct Game {
    player: Vec2,
    player_vel: f32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    enemies_moving_left: bool,
    enemies_offset: Vec2,
    seconds_timer: Timer,
}
impl Game {
    fn init() -> Self {
        let mut enemies = vec![];
        let start_x = WIDTH / 2.0 - (ENEMY_COLS as f32 / 2.0) * ENEMY_WIDTH * 2.0;
        let start_y = ENEMY_HEIGHT;
        for row in 0..ENEMY_ROWS {
            for col in 0..ENEMY_COLS {
                enemies.push(Enemy {
                    pos: Vec2::new(start_x + col as f32 * ENEMY_WIDTH * 2.0, start_y + row as f32 * ENEMY_HEIGHT * 2.0),
                    dead: false,
                });
            }
        }
        Self {
            player: Vec2::new(WIDTH / 2.0, HEIGHT - PLAYER_HEIGHT / 2.0),
            enemies,
            seconds_timer: Timer {
                elapsed: 0.0,
                duration: 1.0,
            },
            ..Self::default()
        }
    }
    fn update(&mut self, input: &InputState, dt: f32) {
        if input.is_key_down(KeyId::R) {
            *self = Self::init();
            return;
        }

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

        if self.enemies.is_empty() {
            return;
        }

        self.seconds_timer.elapsed += dt;
        if self.seconds_timer.elapsed >= self.seconds_timer.duration {
            self.seconds_timer.elapsed -= self.seconds_timer.duration;
            if self.enemies_moving_left {
                self.enemies_offset.x -= 50.0;
                if self.enemies_offset.x < -250.0 {
                    self.enemies_moving_left = false;
                }
            } else {
                self.enemies_offset.x += 50.0;
                if self.enemies_offset.x > 300.0 {
                    self.enemies_offset.x -= 50.0;
                    self.enemies_offset.y += 50.0;
                    self.enemies_moving_left = true;
                } else if self.enemies_offset.x > 250.0 {
                    //self.enemies_moving_left = true;
                    self.enemies_offset.x += 50.0;
                }
            }
        }

        self.player = self.player + Vec2::new(self.player_vel, 0.0) * dt;

        for bullet in &mut self.bullets {
            bullet.pos = bullet.pos + Vec2::new(0.0, -BULLET_SPEED) * dt;
        }

        for enemy in &mut self.enemies {
            let enemy_rect = Rect::center_extent(enemy.pos + self.enemies_offset, (ENEMY_WIDTH, ENEMY_HEIGHT));
            for bullet in &mut self.bullets {
                let bullet_rect = Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT));
                if enemy_rect.collides(&bullet_rect) {
                    enemy.dead = true;
                    bullet.pos.y = -10.0;
                }
            }
        }

        self.bullets.retain(|b| b.pos.y > 0.0);
        self.enemies.retain(|e| !e.dead);
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        vk_util::push_rect(cmd, Rect::center_extent(self.player, (PLAYER_WIDTH, PLAYER_HEIGHT)), 0.9);

        if self.enemies.is_empty() {
            vk_util::push_str_centered_color(
                cmd,
                "Victory",
                HEIGHT / 2.0 - 100.0,
                0.0,
                10.0,
                color::ORANGE,
                false,
                Rect::offset_extent((0.0, 0.0), (WIDTH, HEIGHT)),
            );
        } else {
            for bullet in &self.bullets {
                vk_util::push_rect(cmd, Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT)), 0.9);
            }

            for enemy in &self.enemies {
                vk_util::push_rect(
                    cmd,
                    Rect::center_extent(enemy.pos + self.enemies_offset, (ENEMY_WIDTH, ENEMY_HEIGHT)),
                    0.9,
                );
            }
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

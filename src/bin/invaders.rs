use icarus::color;
use icarus::color::Color;
use icarus::input::{InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

// TODO: Bunker destruction
// TODO: Misterious ship

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;

const PLAYER_SPEED: f32 = 1000.0;
const PLAYER_WIDTH: f32 = 60.0;
const PLAYER_HEIGHT: f32 = 32.0;
const PLAYER_COLOR: Color = color::GREEN;

const BUNKER_WIDTH: f32 = 80.0;
const BUNKER_HEIGHT: f32 = 64.0;

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
    vel: Vec2,
}

#[derive(Default)]
struct Enemy {
    pos: Vec2,
    material: u32,
    dead: bool,
}
#[derive(Default)]
struct Splat {
    pos: Vec2,
    duration: f32,
}

#[derive(Default)]
struct Timer {
    elapsed: f32,
    duration: f32,
}

#[derive(Default)]
struct Game {
    paused: bool,
    player: Vec2,
    player_vel: f32,
    health: u32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    enemies_moving_left: bool,
    enemies_offset: Vec2,
    splats: Vec<Splat>,
    bunkers: Vec<Vec2>,
    seconds_timer: Timer,
    rand: Rand,
}
impl Game {
    fn init() -> Self {
        let mut enemies = vec![];
        let start_x = WIDTH / 2.0 - (ENEMY_COLS as f32 / 2.0) * ENEMY_WIDTH * 2.0;
        let start_y = ENEMY_HEIGHT;
        for row in 0..ENEMY_ROWS {
            let material = match row {
                0 => 2,
                1 | 2 => 4,
                3 | 4 => 6,
                _ => unreachable!(),
            };
            for col in 0..ENEMY_COLS {
                enemies.push(Enemy {
                    pos: Vec2::new(start_x + col as f32 * ENEMY_WIDTH * 2.0, start_y + row as f32 * ENEMY_HEIGHT * 2.0),
                    dead: false,
                    material,
                });
            }
        }
        let bunker_count = 4;
        let bunker_spacing = 3.0 * BUNKER_WIDTH;
        let bunker_start_x = WIDTH / 2.0 - bunker_spacing * (bunker_count as f32) / 2.0 + bunker_spacing / 2.0;
        let mut bunkers = vec![];
        for i in 0..bunker_count {
            bunkers.push(Vec2::new(bunker_start_x + (i as f32) * bunker_spacing, HEIGHT - BUNKER_HEIGHT * 3.0));
        }
        Self {
            player: Vec2::new(WIDTH / 2.0, HEIGHT - 2.0 * PLAYER_HEIGHT),
            health: 3,
            enemies,
            bunkers,
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

        if input.was_key_pressed(KeyId::P) {
            self.paused = !self.paused;
        }
        if self.paused || self.health <= 0 {
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
                vel: Vec2::new(0.0, -BULLET_SPEED),
            });
        }

        if self.enemies.iter().filter(|e| !e.dead).count() == 0 {
            return;
        }

        self.seconds_timer.elapsed += dt;
        self.enemies.iter_mut().for_each(|e| {
            if self.seconds_timer.elapsed < 0.5 {
                if e.material % 2 == 0 {
                    e.material += 1;
                }
            } else {
                if e.material % 2 != 0 {
                    e.material -= 1;
                }
            }
        });
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
                    self.enemies_offset.x += 50.0;
                }
            }

            let idx = self.rand.next_usize() % self.enemies.iter().filter(|e| !e.dead).count();
            self.bullets.push(Bullet {
                pos: self.enemies.iter().filter(|e| !e.dead).nth(idx).unwrap().pos + self.enemies_offset,
                vel: Vec2::new(0.0, BULLET_SPEED),
            });
        }
        self.splats.iter_mut().for_each(|s| s.duration -= dt);
        self.splats.retain(|s| s.duration > 0.0);

        self.player = self.player + Vec2::new(self.player_vel, 0.0) * dt;
        self.player.x = self.player.x.clamp(PLAYER_WIDTH / 2.0, WIDTH - PLAYER_WIDTH / 2.0);

        for bullet in &mut self.bullets {
            bullet.pos = bullet.pos + bullet.vel * dt;
        }

        for bunker in &self.bunkers {
            let bunker_rect = Rect::center_extent((bunker.x, bunker.y), (BUNKER_WIDTH, BUNKER_HEIGHT));
            for bullet in &mut self.bullets {
                let bullet_rect = Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT));
                if bunker_rect.collides(&bullet_rect) {
                    // TODO: Damage bunker
                    bullet.pos.y = -10.0;
                }
            }
        }

        for enemy in &mut self.enemies.iter_mut().filter(|e| !e.dead) {
            let enemy_rect = Rect::center_extent(enemy.pos + self.enemies_offset, (ENEMY_WIDTH, ENEMY_HEIGHT));
            for bullet in &mut self.bullets.iter_mut().filter(|b| b.vel.y < 0.0) {
                let bullet_rect = Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT));
                if enemy_rect.collides(&bullet_rect) {
                    enemy.dead = true;
                    bullet.pos.y = -10.0;
                    self.splats.push(Splat {
                        pos: enemy.pos,
                        duration: 0.2,
                    });
                }
            }
        }

        let player_rect = Rect::center_extent(self.player, (PLAYER_WIDTH, PLAYER_HEIGHT));
        for bullet in &mut self.bullets.iter_mut().filter(|b| b.vel.y > 0.0) {
            let bullet_rect = Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT));
            if player_rect.collides(&bullet_rect) {
                self.health -= 1;
                bullet.pos.y = -10.0;
            }
        }

        self.bullets.retain(|b| b.pos.y > 0.0 && b.pos.y < HEIGHT);
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>, materials: &mut Vec<u32>) {
        // Render ship
        vk_util::push_rect_color(
            cmd,
            Rect::center_extent((WIDTH / 2.0, 50.0), (PLAYER_WIDTH, PLAYER_HEIGHT)),
            0.9,
            color::RED,
        );
        materials.push(10);

        vk_util::push_rect_color(
            cmd,
            Rect::center_extent(self.player, (PLAYER_WIDTH, PLAYER_HEIGHT)),
            0.9,
            PLAYER_COLOR,
        );
        materials.push(1);

        for bunker in &self.bunkers {
            vk_util::push_rect_color(
                cmd,
                Rect::center_extent((bunker.x, bunker.y), (BUNKER_WIDTH, BUNKER_HEIGHT)),
                0.9,
                PLAYER_COLOR,
            );
            materials.push(8);
        }

        for i in 0..self.health {
            vk_util::push_rect_color(
                cmd,
                Rect::center_extent(
                    (PLAYER_WIDTH + (i as f32) * PLAYER_WIDTH * 1.3, HEIGHT - PLAYER_HEIGHT * 0.8 / 2.0),
                    (PLAYER_WIDTH * 0.8, PLAYER_HEIGHT * 0.8),
                ),
                0.9,
                PLAYER_COLOR,
            );
            materials.push(1);
        }

        if self.health <= 0 {
            let count = cmd.len();
            vk_util::push_str_centered_color(
                cmd,
                "Game Over",
                HEIGHT / 2.0 - 100.0,
                0.0,
                10.0,
                color::ORANGE,
                false,
                Rect::offset_extent((0.0, 0.0), (WIDTH, HEIGHT)),
            );
            let count = cmd.len() - count;
            for _ in 0..count {
                materials.push(0);
            }
        } else if self.enemies.iter().filter(|e| !e.dead).count() == 0 {
            let count = cmd.len();
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
            let count = cmd.len() - count;
            for _ in 0..count {
                materials.push(0);
            }
        } else {
            for bullet in &self.bullets {
                vk_util::push_rect(cmd, Rect::center_extent(bullet.pos, (BULLET_WIDTH, BULLET_HEIGHT)), 0.9);
                materials.push(0);
            }

            for row in 0..ENEMY_ROWS {
                for col in 0..ENEMY_COLS {
                    let enemy = &self.enemies[row * ENEMY_COLS + col];
                    if !enemy.dead {
                        vk_util::push_rect(
                            cmd,
                            Rect::center_extent(enemy.pos + self.enemies_offset, (ENEMY_WIDTH, ENEMY_HEIGHT)),
                            0.9,
                        );
                        materials.push(enemy.material);
                    }
                }
            }

            for splat in &self.splats {
                vk_util::push_rect(
                    cmd,
                    Rect::center_extent(splat.pos + self.enemies_offset, (ENEMY_WIDTH, ENEMY_HEIGHT)),
                    0.9,
                );
                materials.push(9);
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
    let mut vk_ctx =
        VkContext::init(&platform, mem::size_of::<RenderCommand>() * MAX_ENTITIES, Some(String::from("sprite")));

    vk_ctx.vertex_buffer.destroy();
    let vertices: [(f32, f32); 4] = [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)];
    vk_ctx.create_vertex_buffer(&vertices);

    vk_ctx.load_texture_image("assets/textures/invaders/player.png"); // 1
    vk_ctx.load_texture_image("assets/textures/invaders/invader_01_1.png"); // 2
    vk_ctx.load_texture_image("assets/textures/invaders/invader_01_2.png"); // 3
    vk_ctx.load_texture_image("assets/textures/invaders/invader_02_1.png"); // 4
    vk_ctx.load_texture_image("assets/textures/invaders/invader_02_2.png"); // 5
    vk_ctx.load_texture_image("assets/textures/invaders/invader_03_1.png"); // 6
    vk_ctx.load_texture_image("assets/textures/invaders/invader_03_2.png"); // 7
    vk_ctx.load_texture_image("assets/textures/invaders/bunker.png"); // 8
    vk_ctx.load_texture_image("assets/textures/invaders/splat.png"); // 9
    vk_ctx.load_texture_image("assets/textures/invaders/ship.png"); // 10
    vk_ctx.update_descriptor_sets((WIDTH, HEIGHT));

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
        let mut materials = vec![];
        game.render(&mut cmd, &mut materials);
        let rotations = vec![0; cmd.len()];
        vk_ctx.render(&cmd, Some(BG_COLOR), &materials, &rotations);
    }
}

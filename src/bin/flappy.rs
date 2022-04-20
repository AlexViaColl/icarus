use icarus::input::{InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

// TODO: Implement collision detection
// TODO: Change pipe height randomly
// TODO: Animate bird tilt during jump

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 675.0;

const MAX_ENTITIES: usize = 200;

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Flappy Bird"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(
        &platform,
        mem::size_of::<RenderCommand>() * MAX_ENTITIES,
        8, //mem::size_of::<GlobalState>(),
        Some(String::from("snake")),
    );

    vk_ctx.vertex_buffer.drop();
    let vertices: [(f32, f32); 4] = [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)];
    vk_ctx.create_vertex_buffer(&vertices);

    vk_ctx.load_texture_image("assets/textures/flappy/background-day.png");
    vk_ctx.load_texture_image("assets/textures/flappy/base.png");
    vk_ctx.load_texture_image("assets/textures/flappy/pipe-green.png");
    vk_ctx.load_texture_image("assets/textures/flappy/bluebird-downflap.png");
    vk_ctx.load_texture_image("assets/textures/flappy/bluebird-midflap.png");
    vk_ctx.load_texture_image("assets/textures/flappy/bluebird-upflap.png");

    for i in 0..10 {
        vk_ctx.load_texture_image(format!("assets/textures/flappy/{}.png", i));
    }

    vk_ctx.update_descriptor_sets((platform.window_width, platform.window_height));

    // Main loop
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render(&mut vk_ctx);
    }

    vk_ctx.cleanup(&platform);
}

#[derive(Default)]
struct Game {
    running: bool,
    paused: bool,
    cmd: Vec<RenderCommand>,
    materials: Vec<u32>,
    rotations: Vec<u32>,

    sprites: Vec<Sprite>,

    bird_vel: Vec2,
    timer: f32,
    score: u32,
}

const BG_WIDTH: f32 = 288.0;
const BG_HEIGHT: f32 = 512.0;
const BASE_WIDTH: f32 = 336.0;
const BASE_HEIGHT: f32 = 112.0;
const BIRD_WIDTH: f32 = 34.0;
const BIRD_HEIGHT: f32 = 24.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;

const BG_Y_OFFSET: f32 = (HEIGHT - BG_HEIGHT) / 2.0;

impl Game {
    fn init() -> Self {
        let mut sprites = vec![];
        for i in 0..6 {
            // Background
            sprites.push(Sprite::new(
                Material::Background,
                ((i as f32) * BG_WIDTH, HEIGHT / 2.0),
                (BG_WIDTH, /*BG_HEIGHT*/ HEIGHT),
                0.9,
            ));
            // Base
            sprites.push(Sprite::new(
                Material::Base,
                ((i as f32) * BASE_WIDTH, HEIGHT - (BASE_HEIGHT / 2.0)),
                (BASE_WIDTH, BASE_HEIGHT),
                0.8,
            ));
        }
        // Pipes
        sprites.push(Sprite::new(
            Material::Pipe,
            (WIDTH / 2.0 + BG_WIDTH, HEIGHT - BASE_HEIGHT - (PIPE_HEIGHT / 2.0)),
            (PIPE_WIDTH, PIPE_HEIGHT),
            0.2,
        ));
        sprites.push(Sprite::with_rot(
            Material::Pipe,
            (WIDTH / 2.0 + BG_WIDTH, 0.0),
            (PIPE_WIDTH, PIPE_HEIGHT),
            0.2,
            2,
        ));
        // Bird
        let bird_pos = (WIDTH / 2.0, HEIGHT / 2.0);
        sprites.push(Sprite::new(Material::BirdMid, bird_pos, (BIRD_WIDTH, BIRD_HEIGHT), 0.1));
        Self {
            running: true,
            sprites,
            bird_vel: Vec2::new(0.0, 50.0),
            ..Self::default()
        }
    }

    fn update(&mut self, input: &InputState, dt: f32) {
        if input.was_key_pressed(KeyId::Esc) {
            self.running = false;
        }

        if input.was_key_pressed(KeyId::R) {
            *self = Self::init();
            return;
        }

        if input.was_key_pressed(KeyId::P) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }

        self.timer += dt;
        if self.timer >= 1.0 {
            self.timer -= 1.0;
        }

        if input.was_key_pressed(KeyId::Space) {
            self.bird_vel = Vec2::new(0.0, -400.0);
        }

        self.bird_vel = self.bird_vel + Vec2::new(0.0, 800.0 * dt);

        let bird_idx = self.sprites.len() - 1;
        let bird_pos = &mut self.sprites[bird_idx].pos;
        *bird_pos = *bird_pos + self.bird_vel * dt;

        if self.timer < 0.33 {
            self.sprites[bird_idx].material = Material::BirdDown;
        } else if self.timer < 0.66 {
            self.sprites[bird_idx].material = Material::BirdMid;
        } else {
            self.sprites[bird_idx].material = Material::BirdUp;
        }

        // Update base
        for i in 0..6 {
            self.sprites[2 * i + 1].pos.x = self.sprites[2 * i + 1].pos.x - 200.0 * dt;
            if self.sprites[2 * i + 1].pos.x < -BASE_WIDTH {
                self.sprites[2 * i + 1].pos.x += 6.0 * BASE_WIDTH;
            }
        }

        // Update pipes
        for i in 12..14 {
            self.sprites[i].pos.x = self.sprites[i].pos.x - 200.0 * dt;
            if self.sprites[i].pos.x < -PIPE_WIDTH {
                self.sprites[i].pos.x = WIDTH;
            }
        }

        if self.sprites[bird_idx].pos.x > self.sprites[12].pos.x
            && self.sprites[bird_idx].pos.x < self.sprites[12].pos.x + 200.0 * dt
        {
            self.score += 1;
        }
    }

    fn render(&mut self, vk_ctx: &mut VkContext) {
        self.cmd.clear();
        self.materials.clear();
        self.rotations.clear();

        for i in 0..self.sprites.len() {
            self.render_sprite(self.sprites[i]);
        }

        // render score
        self.render_number(self.score);

        vk_ctx.render(&self.cmd, None, &self.materials, &self.rotations);
    }

    fn render_sprite(&mut self, sprite: Sprite) {
        vk_util::push_rect(&mut self.cmd, Rect::center_extent(sprite.pos, sprite.size), sprite.depth);
        self.materials.push(sprite.material as u32);
        self.rotations.push(sprite.rotation);
    }

    fn render_number(&mut self, mut number: u32) {
        let mut start_x = WIDTH / 2.0;
        if number == 0 {
            self.render_digit(number, start_x);
        } else {
            while number > 0 {
                let digit = number % 10;
                number /= 10;
                self.render_digit(digit, start_x);
                start_x -= 24.0 * 1.5;
            }
        }
    }

    fn render_digit(&mut self, digit: u32, x: f32) {
        assert!(digit <= 9);
        let w = if digit == 1 {
            16.0
        } else {
            24.0
        };
        let h = 36.0;
        vk_util::push_rect(&mut self.cmd, Rect::center_extent((x, HEIGHT / 6.0), (w * 1.5, h * 1.5)), 0.0);
        self.materials.push(7 + digit);
        self.rotations.push(0);
    }
}

#[derive(Default, Copy, Clone)]
struct Sprite {
    material: Material,
    pos: Vec2,
    size: Vec2,
    depth: f32,
    rotation: u32,
}

impl Sprite {
    fn new<V: Into<Vec2>>(material: Material, pos: V, size: V, depth: f32) -> Self {
        Self::with_rot(material, pos, size, depth, 0)
    }
    fn with_rot<V: Into<Vec2>>(material: Material, pos: V, size: V, depth: f32, rotation: u32) -> Self {
        Self {
            material,
            pos: pos.into(),
            size: size.into(),
            depth,
            rotation,
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
enum Material {
    Default = 0,
    Background = 1,
    Base = 2,
    Pipe = 3,
    BirdDown = 4,
    BirdMid = 5,
    BirdUp = 6,
}
impl Default for Material {
    fn default() -> Self {
        Self::Default
    }
}

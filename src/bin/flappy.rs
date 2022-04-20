use icarus::color;
use icarus::input::{InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 675.0;

const MAX_ENTITIES: usize = 100;

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
    cmd: Vec<RenderCommand>,
    materials: Vec<u32>,
    rotations: Vec<u32>,

    sprites: Vec<Sprite>,

    bird_vel: Vec2,
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
        for i in 0..5 {
            // Background
            sprites.push(Sprite {
                material: Material::Background,
                pos: Vec2::new((i as f32) * BG_WIDTH, HEIGHT / 2.0),
                size: Vec2::new(BG_WIDTH, /*BG_HEIGHT*/ HEIGHT),
                depth: 0.9,
            });
            // Base
            sprites.push(Sprite {
                material: Material::Base,
                pos: Vec2::new(
                    (i as f32) * BASE_WIDTH,
                    HEIGHT - (BASE_HEIGHT / 2.0), /*BG_Y_OFFSET + BG_HEIGHT - BASE_HEIGHT + BASE_HEIGHT / 2.0*/
                ),
                size: Vec2::new(BASE_WIDTH, BASE_HEIGHT),
                depth: 0.8,
            });
        }
        // Pipe
        sprites.push(Sprite {
            material: Material::Pipe,
            pos: Vec2::new(WIDTH / 2.0 + BG_WIDTH, HEIGHT - BASE_HEIGHT - (PIPE_HEIGHT / 2.0)),
            size: Vec2::new(PIPE_WIDTH, PIPE_HEIGHT),
            depth: 0.2,
        });
        // Bird
        let bird_pos = Vec2::new(WIDTH / 2.0, HEIGHT / 2.0);
        sprites.push(Sprite {
            material: Material::BirdMid,
            pos: bird_pos,
            size: Vec2::new(BIRD_WIDTH, BIRD_HEIGHT),
            depth: 0.1,
        });
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

        if input.was_key_pressed(KeyId::Space) {
            self.bird_vel = Vec2::new(0.0, -200.0);
        }

        self.bird_vel = self.bird_vel + Vec2::new(0.0, 600.0 * dt);

        let bird_idx = self.sprites.len() - 1;
        let mut bird_pos = &mut self.sprites[bird_idx].pos;
        *bird_pos = *bird_pos + self.bird_vel * dt;
    }

    fn render(&mut self, vk_ctx: &mut VkContext) {
        self.cmd.clear();
        self.materials.clear();
        self.rotations.clear();

        for i in 0..self.sprites.len() {
            self.render_sprite(self.sprites[i]);
        }

        vk_ctx.render(&self.cmd, None, &self.materials, &self.rotations);
    }

    fn render_sprite(&mut self, sprite: Sprite) {
        vk_util::push_rect(&mut self.cmd, Rect::center_extent(sprite.pos, sprite.size), sprite.depth);
        self.materials.push(sprite.material as u32);
        self.rotations.push(0);
    }
}

#[derive(Default, Copy, Clone)]
struct Sprite {
    material: Material,
    pos: Vec2,
    size: Vec2,
    depth: f32,
}

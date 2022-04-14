use icarus::color;
use icarus::input::{InputState, KeyId};
use icarus::math::Rect;
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk::*;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::collections::VecDeque;
use std::mem;
use std::time::Instant;

const MAX_ENTITIES: usize = 1000;
const SNAKE_SIZE: f32 = 50.0;

fn main() {
    let mut platform = Platform::init(Config {
        width: 1600,
        height: 900,
        app_name: String::from("Snake"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(
        &platform,
        mem::size_of::<RenderCommand>() * MAX_ENTITIES,
        8, //mem::size_of::<GlobalState>(),
        Vertex::get_binding_description(),
        &Vertex::get_attribute_descriptions(),
    );
    #[rustfmt::skip]
    let vertices = [                                                            // CCW
        Vertex {pos: (-1.0, -1.0, 0.0), uv: (0.0, 0.0), color: (1.0, 1.0, 1.0)},  // Top left
        Vertex {pos: (-1.0,  1.0, 0.0), uv: (0.0, 0.5), color: (1.0, 1.0, 1.0)},  // Bottom left
        Vertex {pos: ( 1.0,  1.0, 0.0), uv: (0.5, 0.5), color: (1.0, 1.0, 1.0)},  // Bottom right
        Vertex {pos: ( 1.0, -1.0, 0.0), uv: (0.5, 0.0), color: (1.0, 1.0, 1.0)},  // Top right
    ];
    vk_ctx.vertex_buffer = vk_util::create_vertex_buffer(&vk_ctx, &vertices);
    vk_ctx.index_buffer = vk_util::create_index_buffer(&vk_ctx, &[0, 1, 2, 2, 3, 0]);

    // Main loop
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        input.reset_transitions();
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render(vk_ctx.frame_width, vk_ctx.frame_height);

        // https://coolors.co/palettes/trending
        //let color = color::srgb_to_linear(0xcb997e).into();
        //let color = color::srgb_to_linear(0xddbea9).into();
        //let color = color::srgb_to_linear(0xffe8d6).into();
        //let color = color::srgb_to_linear(0xb7b7a4).into();
        //let color = color::srgb_to_linear(0xa5a58d).into();
        //let color = color::srgb_to_linear(0x6b705c).into();

        //let color = color::srgb_to_linear(0x7400b8).into();

        let color = color::srgb_to_linear(0x1d1f21).into();
        vk_ctx.render(&game.cmd, 6, Some(color));
    }

    vk_ctx.cleanup(&platform);
}

#[derive(Debug, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

struct Game {
    running: bool,
    paused: bool,
    cmd: Vec<RenderCommand>,
    rand: Rand,
    state: GameState,
    rows: usize,
    cols: usize,
    snake: VecDeque<(usize, usize)>, // (row, col) Front is the head of the snake
    coin: (usize, usize),            // (row, col)
    dir: Direction,
    timer: f32,
    speed: f32, // seconds per tile
}

impl Game {
    fn init() -> Self {
        // 18 rows by 32 columns
        let mut snake = VecDeque::new();
        snake.push_back((9, 16)); // Head
        snake.push_back((9, 17));
        snake.push_back((9, 18)); // Tail

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time since UNIX_EPOCH")
            .as_secs() as usize;
        let mut game = Self {
            running: true,
            paused: false,
            cmd: vec![],
            rand: Rand::new(seed),
            state: GameState::Playing,
            rows: 18,
            cols: 32,
            snake,
            coin: (0, 0),
            dir: Direction::Left,
            timer: 0.0,
            speed: 0.2,
        };
        game.spawn_coin();
        game
    }

    fn update(&mut self, input: &InputState, dt: f32) {
        if input.is_key_down(KeyId::Esc) {
            self.running = false;
            return;
        }

        if self.state == GameState::GameOver {
            return;
        }

        if input.was_key_pressed(KeyId::P) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }

        if input.is_key_down(KeyId::A) {
            self.dir = Direction::Left;
        }
        if input.is_key_down(KeyId::D) {
            self.dir = Direction::Right;
        }
        if input.is_key_down(KeyId::W) {
            self.dir = Direction::Up;
        }
        if input.is_key_down(KeyId::S) {
            self.dir = Direction::Down;
        }

        self.timer += dt;
        if self.timer >= self.speed {
            let front = *self.snake.front().unwrap(); // The snake always grows, so it will always have a front.
            let front = (front.0 as isize, front.1 as isize);
            let new_pos = match self.dir {
                Direction::Left => (front.0, front.1 - 1),
                Direction::Right => (front.0, front.1 + 1),
                Direction::Down => (front.0 + 1, front.1),
                Direction::Up => (front.0 - 1, front.1),
            };

            if is_valid_pos(new_pos, self.rows, self.cols)
                && !self.snake.contains(&(new_pos.0 as usize, new_pos.1 as usize))
            {
                if (new_pos.0 as usize, new_pos.1 as usize) != self.coin {
                    self.snake.pop_back();
                } else {
                    self.spawn_coin();
                }
                self.snake.push_front((new_pos.0 as usize, new_pos.1 as usize));
            } else {
                self.state = GameState::GameOver;
            }
            self.timer -= self.speed;
        }
    }

    fn render(&mut self, width: f32, height: f32) {
        self.cmd.clear();

        match self.state {
            GameState::Playing => {
                // Draw coin
                vk_util::push_rect_color(
                    &mut self.cmd,
                    Rect::offset_extent(
                        ((self.coin.1 as f32) * SNAKE_SIZE, (self.coin.0 as f32) * SNAKE_SIZE),
                        (SNAKE_SIZE, SNAKE_SIZE),
                    ),
                    0.0,
                    color::YELLOW,
                );

                // Draw snake
                for (row, col) in &self.snake {
                    vk_util::push_rect_color(
                        &mut self.cmd,
                        Rect::offset_extent(
                            ((*col as f32) * SNAKE_SIZE, (*row as f32) * SNAKE_SIZE),
                            (SNAKE_SIZE, SNAKE_SIZE),
                        ),
                        0.0,
                        color::GREEN,
                    );
                }
            }
            GameState::GameOver => {
                // Draw snake
                for (row, col) in &self.snake {
                    vk_util::push_rect_color(
                        &mut self.cmd,
                        Rect::offset_extent(
                            ((*col as f32) * SNAKE_SIZE, (*row as f32) * SNAKE_SIZE),
                            (SNAKE_SIZE, SNAKE_SIZE),
                        ),
                        0.1,
                        color::GREEN,
                    );
                }
                vk_util::push_str_centered_color(
                    &mut self.cmd,
                    "Game Over!",
                    height / 2.0,
                    0.0,
                    10.0,
                    color::LIGHT_GREY,
                    true,
                    Rect::offset_extent((0.0, 0.0), (width, height)),
                );
            }
        }
    }

    fn spawn_coin(&mut self) {
        loop {
            let coin_pos = (self.rand.next_usize() % self.rows, self.rand.next_usize() % self.cols);
            if !self.snake.contains(&coin_pos) {
                self.coin = coin_pos;
                break;
            }
        }
    }
}

fn is_valid_pos(pos: (isize, isize), rows: usize, cols: usize) -> bool {
    let (row, col) = pos;
    (0..cols as isize).contains(&col) && (0..rows as isize).contains(&row)
}

#[repr(C)]
#[derive(Default)]
pub struct Vertex {
    pos: (f32, f32, f32),   // 12
    color: (f32, f32, f32), // 12
    uv: (f32, f32),         // 8
}

impl Vertex {
    fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Self>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 3] {
        [
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 0,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 3 * mem::size_of::<f32>() as u32,
            },
            VkVertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: (3 + 3) * mem::size_of::<f32>() as u32,
            },
        ]
    }
}

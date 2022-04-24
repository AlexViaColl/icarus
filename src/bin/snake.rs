use icarus::color;
use icarus::input::{InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::collections::VecDeque;
use std::mem;
use std::time::Instant;

// TODO: Encapsule AI agent logic into a struct or trait.
// TODO: Get a "score" for a particular AI agent implementation.
// TODO: Investigate A*

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;
const SNAKE_AI: bool = false;
const SNAKE_SIZE: f32 = 100.0;
//const SNAKE_SPEED: f32 = 0.0001;
const SNAKE_SPEED: f32 = 0.2;

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Snake"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx =
        VkContext::init(&platform, mem::size_of::<RenderCommand>() * MAX_ENTITIES, Some(String::from("sprite")));
    vk_ctx.vertex_buffer.destroy();
    let vertices: [(f32, f32); 4] = [(-1.0, -1.0), (-1.0, 1.0), (1.0, 1.0), (1.0, -1.0)];
    vk_ctx.create_vertex_buffer(&vertices);

    vk_ctx.load_texture_image("assets/textures/snake/snake_head.png");
    vk_ctx.load_texture_image("assets/textures/snake/snake_body_0.png");
    vk_ctx.load_texture_image("assets/textures/snake/snake_body_1.png");
    vk_ctx.load_texture_image("assets/textures/snake/snake_tail.png");
    vk_ctx.load_texture_image("assets/textures/snake/coin.png");
    let global_state = (platform.window_width, platform.window_height);
    vk_ctx.update_descriptor_sets(global_state);

    // https://coolors.co/palettes/trending
    let colors: [color::Color; 8] = [
        color::srgb_to_linear(0xcb997e).into(),
        color::srgb_to_linear(0xddbea9).into(),
        color::srgb_to_linear(0xffe8d6).into(),
        color::srgb_to_linear(0xb7b7a4).into(),
        color::srgb_to_linear(0xa5a58d).into(),
        color::srgb_to_linear(0x6b705c).into(),
        color::srgb_to_linear(0x7400b8).into(),
        color::srgb_to_linear(0x1d1f21).into(),
    ];

    // Main loop
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    let mut timer = 0.0;
    let mut color = None;
    while game.running {
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render(vk_ctx.frame_width, vk_ctx.frame_height);

        timer += seconds_elapsed;
        if color.is_none() || timer >= 1.0 {
            timer = 0.0;
            //color = Some(color::srgb_to_linear(game.rand.next_u32() % 0xffffff).into());
            //color = Some(colors[game.rand.next_usize() % colors.len()]);
            color = Some(colors[colors.len() - 1]);
        }
        vk_ctx.render(
            &game.cmd,
            color,
            &game.material_ids,
            &game.rotations.iter().map(|r| *r as u32).collect::<Vec<_>>(),
        );
    }

    vk_ctx.cleanup(&platform);
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u32)]
enum Direction {
    Left,  // Turn 0
    Down,  // Turn 90
    Right, // Turn 180
    Up,    // Turn 270
}

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

struct Game {
    running: bool,
    paused: bool,
    debug_mode: bool,

    ai: bool,
    towards_border: bool,

    cmd: Vec<RenderCommand>,
    material_ids: Vec<u32>,
    rotations: Vec<Direction>,
    rand: Rand,
    state: GameState,
    rows: usize,
    cols: usize,
    snake: VecDeque<((usize, usize), Direction)>, // (row, col) Front is the head of the snake
    apple: (usize, usize),                        // (row, col)
    dir: Direction,
    timer: f32,
    speed: f32, // seconds per tile
}

impl Game {
    fn init() -> Self {
        let cols = WIDTH / SNAKE_SIZE;
        let rows = HEIGHT / SNAKE_SIZE;

        let mut snake = VecDeque::new();
        let row = (rows / 2.0) as usize;
        let col = (cols / 2.0) as usize;
        snake.push_back(((row, col), Direction::Left)); // Head
        snake.push_back(((row, col + 1), Direction::Left));
        snake.push_back(((row, col + 2), Direction::Left)); // Tail

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time since UNIX_EPOCH")
            .as_secs() as usize;
        let mut game = Self {
            running: true,
            paused: false,
            debug_mode: false,
            ai: SNAKE_AI,
            towards_border: false,
            cmd: vec![],
            material_ids: vec![],
            rotations: vec![],
            rand: Rand::new(seed),
            state: GameState::Playing,
            rows: rows as usize,
            cols: cols as usize,
            snake,
            apple: (0, 0),
            dir: Direction::Left,
            timer: 0.0,
            speed: SNAKE_SPEED,
        };
        game.spawn_apple();
        game
    }

    fn update(&mut self, input: &InputState, dt: f32) {
        if input.is_key_down(KeyId::Esc) {
            self.running = false;
            return;
        }

        if input.was_key_pressed(KeyId::R) {
            *self = Self::init();
            return;
        }

        if input.was_key_pressed(KeyId::P) {
            self.paused = !self.paused;
        }

        if input.was_key_pressed(KeyId::Space) {
            self.debug_mode = !self.debug_mode;
            if self.debug_mode {
                self.speed = 0.8;
                self.ai = false;
            } else {
                self.speed = SNAKE_SPEED;
            }
        }

        if self.paused {
            return;
        }
        if self.state == GameState::GameOver {
            // TODO: Sleep for a bit...
            //*self = Self::init();
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

        if self.ai {
            let (head_pos, head_dir) = self.snake.front().unwrap();
            let delta =
                Vec2::new(self.apple.0 as f32, self.apple.1 as f32) - Vec2::new(head_pos.0 as f32, head_pos.1 as f32);
            let delta_len = delta.len();
            //println!("delta: {}", delta_len);

            let mut new_dir = *head_dir;
            let (row, col) = *head_pos;
            if row == self.apple.0 && col == 0 && delta_len < (self.cols as f32 / 2.0) {
                new_dir = Direction::Right;
            } else if row == self.apple.0 && col == self.cols - 1 && delta_len < (self.cols as f32 / 2.0) {
                new_dir = Direction::Left;
            }

            let directions = [Direction::Left, Direction::Right, Direction::Up, Direction::Down];
            for _ in 0..10 {
                if self.is_next_move_valid(new_dir) {
                    self.dir = new_dir;
                } else {
                    //self.paused = true;

                    if is_border(*head_pos, self.rows, self.cols) {
                        self.towards_border = false;
                        if !self.is_next_move_valid(new_dir) {
                            if row == 0 && col == 0 {
                                new_dir = Direction::Right;
                            } else if row == 0 && col == self.cols - 1 {
                                new_dir = Direction::Down;
                            } else if row == self.rows - 1 && col == 0 {
                                new_dir = Direction::Up;
                            } else if row == self.rows - 1 && col == self.cols - 1 {
                                new_dir = Direction::Left;
                            } else if col == 0 {
                                new_dir = Direction::Up;
                            } else if col == self.cols - 1 {
                                new_dir = Direction::Down;
                            } else {
                                new_dir = directions[self.rand.next_usize() % 4];
                            }
                        } else if row == self.apple.0 && col == 0 {
                            new_dir = Direction::Right;
                        } else if row == self.apple.0 && col == self.cols - 1 {
                            new_dir = Direction::Left;
                        } else {
                            new_dir = directions[self.rand.next_usize() % 4];
                        }
                    } else if !self.towards_border && row != self.apple.0 {
                        // Try to go to the border
                        if *head_dir == Direction::Up {
                            new_dir = Direction::Right;
                        } else if *head_dir == Direction::Down {
                            new_dir = Direction::Left;
                        } else if *head_dir == Direction::Left {
                            new_dir = Direction::Up;
                        } else if *head_dir == Direction::Right {
                            new_dir = Direction::Down;
                        }
                        self.towards_border = true;
                    }
                }
            }
        }

        self.timer += dt;
        if self.timer >= self.speed {
            let front = self.snake.front().unwrap().0; // The snake always grows, so it will always have a front.
            let front = (front.0 as isize, front.1 as isize);
            let new_pos = match self.dir {
                Direction::Left => (front.0, front.1 - 1),
                Direction::Right => (front.0, front.1 + 1),
                Direction::Down => (front.0 + 1, front.1),
                Direction::Up => (front.0 - 1, front.1),
            };

            if is_valid_pos(new_pos, self.rows, self.cols)
                && !self.snake.iter().map(|s| s.0).any(|x| x == (new_pos.0 as usize, new_pos.1 as usize))
            {
                if (new_pos.0 as usize, new_pos.1 as usize) != self.apple {
                    self.snake.pop_back();
                } else {
                    self.spawn_apple();
                }
                self.snake.push_front(((new_pos.0 as usize, new_pos.1 as usize), self.dir));
            } else {
                self.state = GameState::GameOver;
                //if is_valid_pos(new_pos, self.rows, self.cols) {
                //    self.snake.pop_back();
                //    self.snake.push_front(((new_pos.0 as usize, new_pos.1 as usize), self.dir));
                //}
            }
            self.timer -= self.speed;
        }
    }

    fn is_next_move_valid(&self, new_dir: Direction) -> bool {
        let front = self.snake.front().unwrap().0; // The snake always grows, so it will always have a front.
        let front = (front.0 as isize, front.1 as isize);
        let new_pos = match new_dir {
            Direction::Left => (front.0, front.1 - 1),
            Direction::Right => (front.0, front.1 + 1),
            Direction::Down => (front.0 + 1, front.1),
            Direction::Up => (front.0 - 1, front.1),
        };

        is_valid_pos(new_pos, self.rows, self.cols)
            && !self.snake.iter().map(|s| s.0).any(|x| x == (new_pos.0 as usize, new_pos.1 as usize))
    }

    fn render(&mut self, width: f32, height: f32) {
        self.cmd.clear();
        self.rotations.clear();
        self.material_ids.clear();

        match self.state {
            GameState::Playing => {
                self.render_snake();
                self.render_apple();

                if self.paused {
                    let prev_count = self.cmd.len();
                    vk_util::push_str_centered_color(
                        &mut self.cmd,
                        "Paused",
                        height / 2.0,
                        0.0,
                        10.0,
                        color::LIGHT_GREY,
                        true,
                        Rect::offset_extent((0.0, 0.0), (width, height)),
                    );
                    let count = self.cmd.len() - prev_count;
                    for _ in 0..count {
                        self.material_ids.push(0);
                        self.rotations.push(Direction::Left);
                    }
                }
            }
            GameState::GameOver => {
                self.render_snake();
                self.render_apple();

                let prev_count = self.cmd.len();
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
                let count = self.cmd.len() - prev_count;
                for _ in 0..count {
                    self.material_ids.push(0);
                    self.rotations.push(Direction::Left);
                }
            }
        }
    }

    fn render_snake(&mut self) {
        for (idx, ((row, col), d)) in self.snake.iter().enumerate() {
            vk_util::push_rect_color(
                &mut self.cmd,
                Rect::offset_extent(((*col as f32) * SNAKE_SIZE, (*row as f32) * SNAKE_SIZE), (SNAKE_SIZE, SNAKE_SIZE)),
                0.1,
                color::WHITE,
            );
            match idx {
                // Head
                0 => {
                    self.material_ids.push(1);
                    self.rotations.push(*d /*self.dir*/);
                }
                // Tail
                n if n == self.snake.len() - 1 => {
                    self.material_ids.push(4);
                    self.rotations.push(self.snake[idx - 1].1);
                }
                // Body
                _ => {
                    if self.snake[idx - 1].1 == self.snake[idx].1 {
                        self.material_ids.push(2);
                        self.rotations.push(*d);
                    } else {
                        self.material_ids.push(3);
                        match (*d, self.snake[idx - 1].1) {
                            (Direction::Up, Direction::Right)
                            | (Direction::Right, Direction::Down)
                            | (Direction::Down, Direction::Left)
                            | (Direction::Left, Direction::Up) => self.rotations.push(*d),
                            (Direction::Up, Direction::Left) => self.rotations.push(Direction::Right), //
                            (Direction::Left, Direction::Down) => self.rotations.push(Direction::Up),  //
                            (Direction::Down, Direction::Right) => self.rotations.push(Direction::Left),
                            (Direction::Right, Direction::Up) => self.rotations.push(Direction::Down),
                            _ => self.rotations.push(*d), //unreachable!(),
                        }
                    }
                }
            }

            // Debug code!
            if self.debug_mode {
                vk_util::push_rect_color(
                    &mut self.cmd,
                    Rect::offset_extent(((*col as f32) * SNAKE_SIZE, (*row as f32) * SNAKE_SIZE), (SNAKE_SIZE, 1.0)),
                    0.0,
                    color::GREEN,
                );
                self.material_ids.push(0);
                self.rotations.push(*d /*self.dir*/);
                vk_util::push_rect_color(
                    &mut self.cmd,
                    Rect::offset_extent(((*col as f32) * SNAKE_SIZE, (*row as f32) * SNAKE_SIZE), (1.0, SNAKE_SIZE)),
                    0.0,
                    color::GREEN,
                );
                self.material_ids.push(0);
                self.rotations.push(*d /*self.dir*/);
            }
        }
    }

    fn render_apple(&mut self) {
        vk_util::push_rect_color(
            &mut self.cmd,
            Rect::offset_extent(
                ((self.apple.1 as f32) * SNAKE_SIZE, (self.apple.0 as f32) * SNAKE_SIZE),
                (SNAKE_SIZE, SNAKE_SIZE),
            ),
            0.1,
            color::RED,
        );
        self.material_ids.push(5);
        self.rotations.push(Direction::Left);
    }

    fn spawn_apple(&mut self) {
        loop {
            let apple = (self.rand.next_usize() % self.rows, self.rand.next_usize() % self.cols);
            if !self.snake.iter().map(|s| s.0).any(|x| x == apple) {
                self.apple = apple;
                break;
            }
        }
    }
}

fn is_border(pos: (usize, usize), rows: usize, cols: usize) -> bool {
    let (row, col) = pos;
    row == 0 || row == rows - 1 || col == 0 || col == cols - 1
}

fn is_valid_pos(pos: (isize, isize), rows: usize, cols: usize) -> bool {
    let (row, col) = pos;
    (0..cols as isize).contains(&col) && (0..rows as isize).contains(&row)
}

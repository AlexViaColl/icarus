use icarus::glyph::{Glyph, GLYPHS, GLYPH_WIDTH};
use icarus::input::{InputState, KeyId};
use icarus::math::Vec2;
use icarus::platform::{Config, Platform};
use icarus::vk::*;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;
const MAX_ENTITIES: usize = 200;

// Entity ID's
pub const LEFT_PADDLE: usize = 0;
pub const RIGHT_PADDLE: usize = 1;
pub const BALL: usize = 2;

pub const BALL_SPEED: f32 = 700.0;
pub const PADDLE_SPEED: f32 = 700.0;
pub const BALL_SIZE: Vec2 = Vec2::new(50.0, 50.0);
pub const PADDLE_SIZE: Vec2 = Vec2::new(50.0, 200.0);

pub const RIGHT_PADDLE_AI: bool = true;
pub const WIN_SCORE: u32 = 2;
pub const SCORE_TIMEOUT: f32 = 1.0;
pub const GAMEOVER_TIMEOUT: f32 = 3.0;

pub struct Game {
    pub running: bool,

    pub state: GameState,
    pub timeout: Option<(f32, GameState)>,

    // Entities
    pub entity_count: usize,
    pub entities: [Entity; MAX_ENTITIES],

    pub render_commands: Vec<RenderCommand>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameState {
    Start,
    Pause,
    Playing,
    GameOver(usize),       // EntityID of the winner
    ScoreUpdate(u32, u32), // Left and Right score deltas
}
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Entity {
    pub transform: Transform,
    pub vel: Vec2,
    pub score: u32,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub size: Vec2,
}

//#[derive(Debug)]
//pub enum RenderCommand {
//    Quad(f32, f32, f32, f32),
//}

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

fn main() {
    #[rustfmt::skip]
    let vertices = [                                                            // CCW
        Vertex {pos: (-1.0, -1.0, 0.0), uv: (0.0, 0.0), color: (1.0, 1.0, 1.0), ..Vertex::default() },  // Top left
        Vertex {pos: (-1.0,  1.0, 0.0), uv: (0.0, 1.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Bottom left
        Vertex {pos: ( 1.0,  1.0, 0.0), uv: (1.0, 1.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Bottom right
        Vertex {pos: ( 1.0, -1.0, 0.0), uv: (1.0, 0.0), color: (1.0, 1.0, 1.0),..Vertex::default() },  // Top right
    ];
    let indices = [0, 1, 2, 2, 3, 0];

    let mut platform = Platform::init(Config {
        width: WINDOW_WIDTH as u32,
        height: WINDOW_HEIGHT as u32,
        app_name: String::from("Pong"),
    });
    let mut input = InputState::default();
    let mut game = Game::init();
    let mut vk_ctx = VkContext::init(
        &platform,
        mem::size_of::<Entity>() * MAX_ENTITIES,
        8, //mem::size_of::<GlobalState>(),
        Vertex::get_binding_description(),
        &Vertex::get_attribute_descriptions(),
    );
    vk_ctx.vertex_buffer = vk_util::create_vertex_buffer(&vk_ctx, &vertices);
    vk_ctx.index_buffer = vk_util::create_index_buffer(&vk_ctx, &indices);

    // Main loop
    let mut current_frame = 0;
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        input.reset_transitions();
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render();

        vk_ctx.render(game.render_commands.as_slice(), current_frame, indices.len());
        current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    vk_ctx.cleanup(&platform);
}

impl Game {
    fn init() -> Self {
        Self {
            state: GameState::Start,
            timeout: None,
            entities: [Entity::default(); MAX_ENTITIES],
            entity_count: 0,
            running: true,
            render_commands: vec![],
        }
    }

    // Advances the state of the game by dt seconds.
    fn update(&mut self, input: &InputState, dt: f32) {
        if input.was_pressed(KeyId::Esc) {
            self.running = false;
            return;
        }

        // If timeout is specified, don't update the state.
        if let Some((timeout, next_state)) = self.timeout {
            if timeout < dt {
                self.timeout = None;
                self.state = next_state;
            } else {
                self.timeout = Some((timeout - dt, next_state));
                return;
            }
        }

        match self.state {
            GameState::Start => {
                self.entity_count = 0;
                let paddle_y = WINDOW_HEIGHT / 2.0 - PADDLE_SIZE.y / 2.0;
                create_entity(self, (0.0, paddle_y, 50.0, 200.0));
                create_entity(self, (WINDOW_WIDTH - 50.0, paddle_y, PADDLE_SIZE.x, PADDLE_SIZE.y));

                // Ball
                let ball_x = WINDOW_WIDTH / 2.0 - BALL_SIZE.x / 2.0;
                let ball_y = WINDOW_HEIGHT / 2.0 - BALL_SIZE.y / 2.0;
                create_entity(self, (ball_x, ball_y, BALL_SIZE.x, BALL_SIZE.y));
                self.entities[BALL].vel = Vec2::new(-3.0, 1.0).normalize() * BALL_SPEED;

                if input.was_pressed(KeyId::Any) {
                    self.state = GameState::Playing;
                }
            }
            GameState::Pause => {
                if input.was_pressed(KeyId::P) {
                    self.state = GameState::Playing;
                }
                // TODO: Handle collissions and bounces when advancing/undoing a frame
                if input.was_pressed(KeyId::Right) {
                    // Advance by a frame
                    let ball = &mut self.entities[BALL];
                    ball.transform.pos.x += ball.vel.x * dt;
                    ball.transform.pos.y += ball.vel.y * dt;
                }
                if input.was_pressed(KeyId::Left) {
                    // Undo the last frame
                    let ball = &mut self.entities[BALL];
                    ball.transform.pos.x -= ball.vel.x * dt;
                    ball.transform.pos.y -= ball.vel.y * dt;
                }
            }
            GameState::ScoreUpdate(left_delta, right_delta) => {
                self.entities[LEFT_PADDLE].score += left_delta;
                self.entities[RIGHT_PADDLE].score += right_delta;
                if self.entities[LEFT_PADDLE].score >= WIN_SCORE {
                    self.state = GameState::GameOver(LEFT_PADDLE);
                } else if self.entities[RIGHT_PADDLE].score >= WIN_SCORE {
                    self.state = GameState::GameOver(RIGHT_PADDLE);
                } else {
                    self.timeout = Some((SCORE_TIMEOUT, GameState::Playing));
                }
            }
            GameState::GameOver(_) => {
                self.timeout = Some((GAMEOVER_TIMEOUT, GameState::Start));
            }
            GameState::Playing => {
                if input.was_pressed(KeyId::P) {
                    self.state = GameState::Pause;
                    return;
                }

                let left_paddle = &mut self.entities[LEFT_PADDLE];
                left_paddle.vel = Vec2::default();
                if input.is_down(KeyId::W) {
                    left_paddle.vel.y = -PADDLE_SPEED;
                }
                if input.is_down(KeyId::S) {
                    left_paddle.vel.y = PADDLE_SPEED;
                }

                let ball_pos = self.entities[BALL].transform.pos;
                let right_paddle = &mut self.entities[RIGHT_PADDLE];
                right_paddle.vel = Vec2::default();
                if RIGHT_PADDLE_AI {
                    if ball_pos.y < right_paddle.transform.pos.y {
                        right_paddle.vel.y = -PADDLE_SPEED;
                    } else {
                        right_paddle.vel.y = PADDLE_SPEED;
                    }
                } else {
                    if input.is_down(KeyId::Up) {
                        right_paddle.vel.y = -PADDLE_SPEED;
                    }
                    if input.is_down(KeyId::Down) {
                        right_paddle.vel.y = PADDLE_SPEED;
                    }
                }

                let ball_pos = self.entities[BALL].transform.pos;
                let left_paddle_pos = self.entities[LEFT_PADDLE].transform.pos;
                let right_paddle_pos = self.entities[RIGHT_PADDLE].transform.pos;

                let ball = &mut self.entities[BALL];
                if ball.vel.x < 0.0 && ball_pos.x < 0.0 {
                    // println!("Player 2 scores");
                    ball.transform.pos = Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);
                    ball.vel.x *= -1.0;
                    self.state = GameState::ScoreUpdate(0, 1);
                }
                if ball.vel.x > 0.0 && ball_pos.x + BALL_SIZE.x > WINDOW_WIDTH {
                    // println!("Player 1 scores");
                    ball.transform.pos = Vec2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);
                    ball.vel.x *= -1.0;
                    self.state = GameState::ScoreUpdate(1, 0);
                }

                // Ball vs. Left Paddle
                if ball.vel.x < 0.0
                    && ball_pos.x < PADDLE_SIZE.x
                    && (ball_pos.y + BALL_SIZE.y > left_paddle_pos.y && ball_pos.y < left_paddle_pos.y + PADDLE_SIZE.y)
                {
                    // println!("Left Collision");
                    ball.vel.x *= -1.0;
                }

                // Ball vs. Right Paddle
                if ball.vel.x > 0.0
                    && ball_pos.x + BALL_SIZE.x > (WINDOW_WIDTH - PADDLE_SIZE.x)
                    && (ball_pos.y + BALL_SIZE.y > right_paddle_pos.y
                        && ball_pos.y < right_paddle_pos.y + PADDLE_SIZE.y)
                {
                    // println!("Right Collision");
                    ball.vel.x *= -1.0;
                }

                // Bounce off of the top & bottom edges
                if (ball.vel.y < 0.0 && ball.transform.pos.y < 0.0)
                    || (ball.vel.y > 0.0 && (ball.transform.pos.y + BALL_SIZE.y) > WINDOW_HEIGHT)
                {
                    ball.vel.y *= -1.0;
                }

                // Apply velocity to update positions
                let ball = &mut self.entities[BALL];
                ball.transform.pos.x += ball.vel.x * dt;
                ball.transform.pos.y += ball.vel.y * dt;

                let left_paddle = &mut self.entities[LEFT_PADDLE];
                left_paddle.transform.pos.y =
                    (left_paddle.transform.pos.y + left_paddle.vel.y * dt).clamp(0.0, WINDOW_HEIGHT - PADDLE_SIZE.y);
                let right_paddle = &mut self.entities[RIGHT_PADDLE];
                right_paddle.transform.pos.y =
                    (right_paddle.transform.pos.y + right_paddle.vel.y * dt).clamp(0.0, WINDOW_HEIGHT - PADDLE_SIZE.y);
            }
        }
    }

    // Render the current state of the game.
    fn render(&mut self) {
        self.render_commands.clear();

        match self.state {
            GameState::Start => {
                push_str(&mut self.render_commands, "Press a key to start", 100.0, 100.0);
            }
            GameState::GameOver(entity_id) => {
                push_str(&mut self.render_commands, &format!("Player {} won", entity_id + 1), 0.0, 100.0);
            }
            _ => {
                // Score
                let score = format!("{} - {}", self.entities[LEFT_PADDLE].score, self.entities[RIGHT_PADDLE].score);
                push_str(&mut self.render_commands, &score, 0.0, 100.0);
            }
        }

        for i in 0..self.entity_count {
            let entity = self.entities[i];
            let Vec2 {
                x,
                y,
            } = entity.transform.pos;
            let Vec2 {
                x: w,
                y: h,
            } = entity.transform.size;
            self.render_commands.push(RenderCommand::Quad(x, y, w, h));
        }
    }
}

fn create_entity(game: &mut Game, transform: (f32, f32, f32, f32)) {
    assert!(game.entity_count < MAX_ENTITIES);
    game.entities[game.entity_count] = Entity {
        transform: Transform {
            pos: Vec2::new(transform.0, transform.1),
            size: Vec2::new(transform.2, transform.3),
        },
        ..Entity::default()
    };
    game.entity_count += 1;
}

// Renderer API
fn push_quad(render_commands: &mut Vec<RenderCommand>, x: f32, y: f32, w: f32, h: f32) {
    render_commands.push(RenderCommand::Quad(x, y, w, h));
}
pub const GLYPH_PIXEL_SIZE: f32 = 10.0;
fn push_glyph(cmd: &mut Vec<RenderCommand>, glyph: &Glyph, x: f32, y: f32) {
    for row in 0..7 {
        for col in 0..5 {
            if glyph[row * 5 + col] != 0 {
                push_quad(
                    cmd,
                    x + GLYPH_PIXEL_SIZE * (col as f32),
                    y + GLYPH_PIXEL_SIZE * (row as f32),
                    GLYPH_PIXEL_SIZE,
                    GLYPH_PIXEL_SIZE,
                );
            }
        }
    }
}
fn push_char(cmd: &mut Vec<RenderCommand>, c: char, x: f32, y: f32) {
    assert!(c >= ' ' && c <= '~');
    let glyph_idx = c as usize - ' ' as usize;
    push_glyph(cmd, &GLYPHS[glyph_idx], x, y);
}
fn push_str(cmd: &mut Vec<RenderCommand>, s: &str, _x: f32, y: f32) {
    let text_extent = (s.len() as f32) * 6.0 * GLYPH_PIXEL_SIZE;
    let x = WINDOW_WIDTH / 2.0 - text_extent / 2.0;
    for (idx, c) in s.chars().enumerate() {
        push_char(cmd, c, x + (idx as f32) * GLYPH_PIXEL_SIZE * (GLYPH_WIDTH as f32 + 1.0), y);
    }
}

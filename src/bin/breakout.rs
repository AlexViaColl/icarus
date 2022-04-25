use icarus::color;
use icarus::input::{ButtonId, InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;
const PADDLE_WIDTH: f32 = 200.0;
const PADDLE_HEIGHT: f32 = 50.0;
const PADDLE_VEL: f32 = 2000.0;
const BLOCK_SIZE: f32 = 50.0;
const BALL_SIZE: f32 = 50.0;
const BLOCK_PADDING: f32 = 2.0;
const ROW_COUNT: usize = 3;

#[derive(Default)]
struct Game {
    paused: bool,
    paddle: Vec2,
    ball: Vec2,
    ball_vel: Vec2,
    blocks: Vec<Vec2>,
    rect1: Rect,
    rect2: Rect,
}
impl Game {
    fn init() -> Self {
        let mut blocks = vec![];
        for row in 0..ROW_COUNT {
            for col in 0..(WIDTH / BLOCK_SIZE) as usize {
                blocks.push((col as f32 * BLOCK_SIZE + BLOCK_PADDING, row as f32 * BLOCK_SIZE + BLOCK_PADDING).into());
            }
        }
        Self {
            paddle: (WIDTH / 2.0, HEIGHT - PADDLE_HEIGHT / 2.0).into(),
            ball: (WIDTH / 2.0, HEIGHT / 2.0).into(),
            ball_vel: (300.0, 300.0).into(),
            blocks,
            rect1: Rect::center_extent((WIDTH / 2.0, HEIGHT / 2.0), (200.0, 200.0)),
            rect2: Rect::center_extent((WIDTH / 2.0 + 300.0, HEIGHT / 2.0), (200.0, 200.0)),
            ..Self::default()
        }
    }

    fn update(&mut self, input: &InputState, dt: f32) {
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

        if input.was_button_pressed(ButtonId::Left) {
            let button = input.buttons[ButtonId::Left as usize];
            self.rect1 = Rect::center_extent((button.x as f32, button.y as f32), self.rect1.extent);
        }
        let pos = input.get_mouse_pos();
        self.rect2 = Rect::center_extent((pos.x, pos.y), self.rect2.extent);

        if input.is_key_down(KeyId::A) {
            self.paddle.x -= PADDLE_VEL * dt;
        }

        if input.is_key_down(KeyId::D) {
            self.paddle.x += PADDLE_VEL * dt;
        }

        self.paddle.x = self.paddle.x.clamp(PADDLE_WIDTH / 2.0, WIDTH - PADDLE_WIDTH / 2.0);

        let mut new_ball = self.ball + self.ball_vel * dt;

        if new_ball.x <= 0.0 || new_ball.x + BALL_SIZE >= WIDTH {
            let revert_dt = if new_ball.x <= 0.0 {
                (new_ball.x / self.ball_vel.x).abs()
            } else {
                ((WIDTH - BALL_SIZE - new_ball.x) / self.ball_vel.x).abs()
            };
            new_ball = new_ball - self.ball_vel * revert_dt;
            self.ball_vel.x = -self.ball_vel.x;
            let reapply_dt = dt - revert_dt;
            new_ball = new_ball + self.ball_vel * reapply_dt;
        }
        if new_ball.y <= 0.0 || new_ball.y + BALL_SIZE >= HEIGHT {
            self.ball_vel.y = -self.ball_vel.y;
        }

        if Rect::center_extent(self.paddle, (PADDLE_WIDTH, PADDLE_HEIGHT))
            .collides(&Rect::offset_extent(new_ball, (BALL_SIZE, BALL_SIZE)))
        {
            if self.ball_vel.y > 0.0 {
                self.ball_vel.y = -self.ball_vel.y;
            }
        }

        self.ball = new_ball;
        //println!("{:?}", self.ball);
    }

    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        let mut color = color::WHITE;
        if self.rect1.collides(&self.rect2) {
            color = color::RED;
        }
        //vk_util::push_rect_color(cmd, self.rect1, 0.0, color);
        //vk_util::push_rect_color(cmd, self.rect2, 0.0, color);

        // Render player
        vk_util::push_rect_color(
            cmd,
            Rect::center_extent(self.paddle, (PADDLE_WIDTH, PADDLE_HEIGHT)),
            0.0,
            color::WHITE,
        );

        // Render ball
        vk_util::push_rect_color(cmd, Rect::offset_extent(self.ball, (BALL_SIZE, BALL_SIZE)), 0.0, color::WHITE);

        // Render blocks
        for block in &self.blocks {
            vk_util::push_rect(
                cmd,
                Rect::offset_extent(*block, (BLOCK_SIZE - BLOCK_PADDING, BLOCK_SIZE - BLOCK_PADDING)),
                0.0,
            );
        }
    }
}

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Breakout"),
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

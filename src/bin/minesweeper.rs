use icarus::color;
use icarus::color::*;
use icarus::glyph::{Glyph, GLYPH_PIXEL_SIZE};
use icarus::input::{ButtonId, InputState, KeyId};
use icarus::math::{Rect, Vec2};
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

const WINDOW_WIDTH: f32 = 1600.0;
const WINDOW_HEIGHT: f32 = 900.0;
const MAX_ENTITIES: usize = 10000;

const TITLE_COLOR: Color = color!(rgb(0.8, 0.7, 0.1)); // Light yellow

const TILE_SIZE: f32 = 42.0;

const MAX_TILE_COUNT: usize = 30 * 16;

const TILE_CLEAR_COLOR: Color = color!(rgb(0.7, 0.7, 0.7));
const TILE_ACTIVATED_COLOR: Color = color!(rgb(0.5, 0.5, 0.5));

// Layers from top to bottom
const TEXT_Z: f32 = 0.0;
//const OUTLINE_Z: f32 = 0.1;
const TILE_FOREGROUND_Z: f32 = 0.8;
const TILE_BACKGROUND_Z: f32 = 0.9;

pub struct Game {
    pub running: bool,
    pub seconds_elapsed: f32,

    pub level: usize,
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub tile_count: usize,
    pub mine_count: usize,
    pub mines_left: usize,

    pub state: GameState,
    pub tiles: [Tile; MAX_TILE_COUNT], // actual size: tile_count,
    pub render_commands: Vec<RenderCommand>,
    pub rand: Rand,
}
#[derive(PartialEq, Copy, Clone)]
pub enum Tile {
    Clear, // Tile has not been clicked and it does not contain a mine
    Mine,  // Tile has not been clicked and it contains a mine

    Flagged(bool), // Tile marked with a flag after "right click" and a bool indicating if there is a mine or not

    Neighbors(usize), // Tile has been clicked, show the number indicating neighboring mines
    MineExploded,     // Tile has been clicked and it contains a mine

    MineShown, // Another tile containing a mine has been clicked, show that this tile also had a mine
}
#[derive(PartialEq)]
pub enum GameState {
    Menu(usize),
    Playing,
    GameOver,
    Win,
}
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Entity {
    pub transform: Transform,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub size: Vec2,
}
#[repr(C)]
#[derive(Debug)]
pub struct GlobalState {
    width: u32,
    height: u32,
}

fn main() {
    let mut platform = Platform::init(Config {
        width: 1600,
        height: 900,
        app_name: String::from("MineSweeper"),
    });
    let mut input = InputState::default();
    let mut game = Game::init(0);
    let mut vk_ctx = VkContext::init(&platform, mem::size_of::<RenderCommand>() * MAX_ENTITIES, None);

    // Main loop
    let start_time = Instant::now();
    let mut prev_frame_time = start_time;
    while game.running {
        platform.process_messages(&mut input);

        let seconds_elapsed = prev_frame_time.elapsed().as_secs_f32();
        prev_frame_time = Instant::now();
        game.update(&input, seconds_elapsed);
        game.render();

        vk_ctx.render(&game.render_commands, None, &[], &[]);
    }

    vk_ctx.cleanup(&platform);
}

impl Game {
    fn init(level: usize) -> Self {
        let (tiles_x, tiles_y, mine_count) = match level {
            0 => (9, 9, 10),
            1 => (16, 16, 40),
            2 => (30, 16, 99),
            n => panic!("Level {} is not supported!", n),
        };

        //println!("Game::init()");
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time since UNIX_EPOCH")
            .as_secs() as usize;
        //println!("Seed: {}", seed);
        let mut rand = Rand::new(seed);
        let mut tiles = [Tile::Clear; MAX_TILE_COUNT];
        let mut placed_mines = 0;
        while placed_mines < mine_count {
            let row = (rand.next_u32() as usize) % tiles_y;
            let col = (rand.next_u32() as usize) % tiles_x;
            let idx = row * tiles_x + col;
            if tiles[idx] == Tile::Clear {
                tiles[idx] = Tile::Mine;
                placed_mines += 1;
            }
        }
        Self {
            running: true,
            seconds_elapsed: 0.0,
            state: GameState::Menu(0),
            level,
            tiles_x,
            tiles_y,
            tile_count: tiles_x * tiles_y,
            mine_count,
            mines_left: mine_count,
            tiles,
            render_commands: vec![],
            rand,
        }
    }

    // Advances the state of the game by dt seconds.
    fn update(&mut self, input: &InputState, dt: f32) {
        if input.was_key_pressed(KeyId::Esc) {
            self.running = false;
            return;
        }

        // Restart game
        if input.was_key_pressed(KeyId::R) {
            *self = Self::init(self.level);
            self.state = GameState::Playing;
            return;
        }
        // Go back to the menu
        if input.was_key_pressed(KeyId::M) {
            *self = Self::init(self.level);
            return;
        }

        if let GameState::Menu(level) = self.state {
            if input.was_key_pressed(KeyId::Enter) || input.was_key_pressed(KeyId::Space) {
                *self = Self::init(level);
                self.state = GameState::Playing;
                return;
            }

            if input.was_key_pressed(KeyId::Up) || input.was_key_pressed(KeyId::W) {
                self.state = GameState::Menu(level.saturating_sub(1));
            }
            if input.was_key_pressed(KeyId::Down) || input.was_key_pressed(KeyId::S) {
                self.state = GameState::Menu((level + 1).min(2));
            }
        }

        if self.state == GameState::Playing {
            self.seconds_elapsed += dt;

            if input.was_button_pressed(ButtonId::Right) {
                let button = input.buttons[ButtonId::Right as usize];
                if let Some(idx) = get_tile_from_pos(self, button.x, button.y) {
                    self.tiles[idx] = match self.tiles[idx] {
                        Tile::Clear => {
                            self.mines_left -= 1;
                            Tile::Flagged(false)
                        }
                        Tile::Mine => {
                            self.mines_left -= 1;
                            Tile::Flagged(true)
                        }
                        Tile::Flagged(false) => {
                            self.mines_left += 1;
                            Tile::Clear
                        }
                        Tile::Flagged(true) => {
                            self.mines_left += 1;
                            Tile::Mine
                        }
                        t => t,
                    };
                }
            }
            if input.was_button_pressed(ButtonId::Left) {
                let button = input.buttons[ButtonId::Left as usize];
                if let Some(idx) = get_tile_from_pos(self, button.x, button.y) {
                    activate_tile(self, idx);
                }
            }
        }
    }

    // Render the current state of the game.
    fn render(&mut self) {
        self.render_commands.clear();

        //push_rect(&mut self.render_commands, Rect::offset_extent((0.0, 25.0), (WINDOW_WIDTH, 2.0)), TILE_FOREGROUND_Z);
        //push_rect(&mut self.render_commands, Rect::offset_extent((0.0, 175.0), (WINDOW_WIDTH, 2.0)), TILE_FOREGROUND_Z);

        // Glyphs are 5x7 tiles, each tile is pixel_size x pixel_size
        // We want our text height to be 150. 150 / 7 = ~21.43
        const TITLE_Y: f32 = 25.0;
        const TITLE_PIXEL_SIZE: f32 = 21.43;

        match self.state {
            GameState::Menu(option) => {
                vk_util::push_str_centered_color(
                    &mut self.render_commands,
                    "Level",
                    TITLE_Y + 25.0,
                    TEXT_Z,
                    TITLE_PIXEL_SIZE,
                    TITLE_COLOR,
                    true,
                    Rect::offset_extent((0.0, 0.0), (WINDOW_WIDTH, WINDOW_HEIGHT)),
                );
                render_menu(self, option)
            }
            GameState::Playing => {
                vk_util::push_str(
                    &mut self.render_commands,
                    &format!("{:03}", self.mines_left),
                    3.0 * WINDOW_WIDTH / 4.0,
                    75.0,
                    TEXT_Z,
                    GLYPH_PIXEL_SIZE * 0.7,
                );
                vk_util::push_str(
                    &mut self.render_commands,
                    &format!("{:03}", self.seconds_elapsed as u32),
                    3.0 * WINDOW_WIDTH / 4.0 + 150.0,
                    75.0,
                    TEXT_Z,
                    GLYPH_PIXEL_SIZE * 0.7,
                );
                render_board(self);
            }
            GameState::Win => {
                render_board(self);
                vk_util::push_str_centered_color(
                    &mut self.render_commands,
                    "Victory!",
                    TITLE_Y,
                    TEXT_Z,
                    TITLE_PIXEL_SIZE,
                    TITLE_COLOR,
                    true,
                    Rect::offset_extent((0.0, 0.0), (WINDOW_WIDTH, WINDOW_HEIGHT)),
                );
            }
            GameState::GameOver => {
                render_board(self);
                vk_util::push_str_centered_color(
                    &mut self.render_commands,
                    "Game Over!",
                    TITLE_Y,
                    TEXT_Z,
                    TITLE_PIXEL_SIZE,
                    TITLE_COLOR,
                    true,
                    Rect::offset_extent((0.0, 0.0), (WINDOW_WIDTH, WINDOW_HEIGHT)),
                );
            }
        }
    }
}

fn render_menu(game: &mut Game, option: usize) {
    let cmd = &mut game.render_commands;
    //push_str_centered_color(cmd, "DIFFICULY", 100.0, TEXT_Z, GLYPH_PIXEL_SIZE * 1.3, WHITE, true);
    let x = WINDOW_WIDTH / 2.0;
    let mut y = 300.0;
    let w = 750.0;
    let h = 100.0;
    let padding = 25.0;
    let texts = ["Beginner", "Intermediate", "Expert"];
    for (i, text) in texts.iter().enumerate() {
        vk_util::push_rect_color(
            cmd,
            Rect::center_extent((x, y + 40.0), (w, h)),
            TILE_BACKGROUND_Z,
            if option == i {
                DARK_GREY
            } else {
                LIGHT_GREY
            },
        );
        vk_util::push_str_centered_color(
            cmd,
            text,
            y,
            TEXT_Z,
            GLYPH_PIXEL_SIZE * 1.0,
            if option == i {
                WHITE
            } else {
                DARK_GREY
            },
            true,
            Rect::offset_extent((0.0, 0.0), (WINDOW_WIDTH, WINDOW_HEIGHT)),
        );
        y += h + padding;
    }
}

fn render_board(game: &mut Game) {
    let cmd = &mut game.render_commands;
    let center_x = WINDOW_WIDTH / 2.0;
    let offset = 200.0;
    let center_y = offset + (WINDOW_HEIGHT - offset) / 2.0;

    let start_x = center_x - TILE_SIZE * (game.tiles_x as f32 / 2.0).floor();
    let start_y = center_y - TILE_SIZE * (game.tiles_y as f32 / 2.0).floor();
    for row in 0..game.tiles_y {
        for col in 0..game.tiles_x {
            let idx = row * game.tiles_x + col;
            let color = match game.tiles[idx] {
                Tile::Clear => TILE_CLEAR_COLOR,
                Tile::Mine => TILE_CLEAR_COLOR,
                Tile::Flagged(_) => TILE_ACTIVATED_COLOR, //(1.0, 0.2, 0.2),
                Tile::Neighbors(_) => TILE_ACTIVATED_COLOR,
                Tile::MineExploded => RED,
                Tile::MineShown => DARK_GREY,
            };
            vk_util::push_rect_color(
                cmd,
                Rect::center_extent(
                    (start_x + (col as f32) * TILE_SIZE, start_y + (row as f32) * TILE_SIZE),
                    (TILE_SIZE - 2.0, TILE_SIZE - 2.0),
                ),
                TILE_BACKGROUND_Z,
                color,
            );
            match game.tiles[idx] {
                Tile::MineShown | Tile::MineExploded => {
                    let offset = (
                        start_x + (col as f32) * TILE_SIZE - TILE_SIZE / 4.0,
                        start_y + (row as f32) * TILE_SIZE - 18.0,
                    );
                    vk_util::push_glyph_color(cmd, &MINE_GLYPH, offset, TILE_FOREGROUND_Z, 6.0, BLACK, false);
                }
                Tile::Flagged(_) => {
                    let offset = (
                        start_x + (col as f32) * TILE_SIZE - TILE_SIZE / 4.0,
                        start_y + (row as f32) * TILE_SIZE - 24.0,
                    );
                    vk_util::push_glyph_color(cmd, &FLAG_GLYPH, offset, TILE_FOREGROUND_Z, 6.0, BLACK, false);
                }
                Tile::Neighbors(0) => {}
                Tile::Neighbors(count) => {
                    let color = match count {
                        1 => BLUE,
                        2 => DARK_GREEN,
                        3 => RED,
                        4 => DARK_BLUE,
                        5 => BROWN,
                        6 => CYAN,
                        7 => BLACK,
                        8 => GREY,
                        _ => WHITE,
                    };
                    let offset = (
                        start_x + (col as f32) * TILE_SIZE - TILE_SIZE / 4.0,
                        start_y + (row as f32) * TILE_SIZE - 18.0,
                    );
                    vk_util::push_str_color(cmd, &format!("{}", count), offset, TILE_FOREGROUND_Z, 5.0, color, false);
                }
                _ => {}
            }
        }
    }
}

fn activate_tile(game: &mut Game, idx: usize) {
    let row = idx / game.tiles_x;
    let col = idx % game.tiles_x;
    match game.tiles[idx] {
        Tile::Clear => {
            // println!("Activating tile at ({}, {})", col, row);
            let neighbors = get_neighbors(game, row as isize, col as isize);
            let count = neighbors.iter().filter(|t| matches!(t.1, Tile::Mine | Tile::Flagged(true))).count();
            game.tiles[idx] = Tile::Neighbors(count);
            if count == 0 {
                for neighbor in neighbors {
                    activate_tile(game, neighbor.0);
                }
            }
            if game.tiles.iter().take(game.tiles_x * game.tiles_y).filter(|t| matches!(t, Tile::Clear)).count() == 0 {
                game.state = GameState::Win;
            }
        }
        Tile::Mine => {
            game.state = GameState::GameOver;
            for idx2 in 0..game.tiles_x * game.tiles_y {
                if idx2 == idx {
                    game.tiles[idx] = Tile::MineExploded;
                } else if game.tiles[idx2] == Tile::Mine {
                    game.tiles[idx2] = Tile::MineShown;
                }
            }
        }
        _ => {}
    }
}

fn get_tile_from_pos(game: &Game, x: i32, y: i32) -> Option<usize> {
    let center_x = WINDOW_WIDTH / 2.0;
    let offset = 200.0;
    let center_y = offset + (WINDOW_HEIGHT - offset) / 2.0;

    let start_x = center_x - TILE_SIZE * (game.tiles_x as f32 / 2.0).floor();
    let start_y = center_y - TILE_SIZE * (game.tiles_y as f32 / 2.0).floor();
    for row in 0..game.tiles_y {
        for col in 0..game.tiles_x {
            let idx = row * game.tiles_x + col;
            let rect = Rect::center_extent(
                (start_x + (col as f32) * TILE_SIZE, start_y + (row as f32) * TILE_SIZE),
                (TILE_SIZE - 2.0, TILE_SIZE - 2.0),
            );
            if rect.is_inside((x as f32, y as f32)) {
                return Some(idx);
            }
        }
    }
    None
}

fn get_neighbors(game: &Game, row: isize, col: isize) -> Vec<(usize, Tile)> {
    let mut neighbors = vec![];
    for j in row - 1..=row + 1 {
        for i in col - 1..=col + 1 {
            if j >= 0 && j < (game.tiles_y as isize) && i >= 0 && i < (game.tiles_x as isize) {
                let idx = (j as usize) * game.tiles_x + (i as usize);
                neighbors.push((idx, game.tiles[idx]));
            }
        }
    }
    neighbors
}

#[rustfmt::skip]
const MINE_GLYPH: Glyph = [
    0, 0, 0, 0, 0,
    1, 0, 1, 0, 1,
    0, 1, 1, 1, 0,
    1, 1, 1, 1, 1,
    0, 1, 1, 1, 0,
    1, 0, 1, 0, 1,
    0, 0, 0, 0, 0,
];
#[rustfmt::skip]
const FLAG_GLYPH: Glyph = [
    0, 0, 0, 0, 0,
    0, 1, 1, 0, 0,
    1, 1, 1, 0, 0,
    0, 0, 1, 0, 0,
    0, 0, 1, 0, 0,
    0, 1, 1, 1, 0,
    1, 1, 1, 1, 1,
];

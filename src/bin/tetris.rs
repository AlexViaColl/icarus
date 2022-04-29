use icarus::color;
use icarus::color::Color;
use icarus::input::{InputState, KeyId};
use icarus::math::Rect;
use icarus::platform::{Config, Platform};
use icarus::rand::Rand;
use icarus::vk_util::{self, RenderCommand, VkContext};

use std::mem;
use std::time::Instant;

// TODO: UI: render borders/boundaries of the board, preview of where the piece will fall, ...

const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 900.0;

const MAX_ENTITIES: usize = 1000;
const TILES_X: isize = 10;
const TILES_Y: isize = 20;
const TILE_SIZE: f32 = 30.0;
const TILE_BG_COLOR: Color = color!(rgb8(15, 15, 15));
const TEXT_COLOR: Color = color!(hex(0xE5E4E2)); //color::srgb_to_linear(0xE5E4E2).into();
const BG_COLOR: Color = color!(hex(0x28282B));

#[derive(Default, Copy, Clone, Debug)]
struct Tile {
    pos: (isize, isize),
    color: color::Color,
}
impl Tile {
    fn new(x: isize, y: isize, color: color::Color) -> Self {
        Self {
            pos: (x, y),
            color,
        }
    }
    fn is_empty(&self) -> bool {
        self.color == TILE_BG_COLOR
    }
}

#[derive(Default, Clone, Debug)]
struct Piece {
    tiles: Vec<Tile>,
}
impl Piece {
    fn is_valid(&self, board_tiles: &[Tile]) -> bool {
        // Are we out of bounds?
        if self.tiles.iter().any(
            |Tile {
                 pos: (x, y),
                 ..
             }| { *x < 0 || *x > TILES_X - 1 || *y < 0 || *y > TILES_Y - 1 },
        ) {
            return false;
        }

        // Are we colliding with another piece?
        if self.tiles.iter().any(
            |Tile {
                 pos: (x, y),
                 ..
             }| {
                let idx = pos_to_idx(*x, *y);
                !board_tiles[idx].is_empty()
            },
        ) {
            return false;
        }

        true
    }
}

#[derive(Default)]
struct Timer {
    elapsed: f32,
    duration: f32,
}

#[derive(Default)]
struct Game {
    paused: bool,
    game_over: bool,
    tiles: Vec<Tile>,
    piece: Piece,
    next_piece: Piece,
    timer: f32, // Goes from 0.0 to time_per_tile and then resets
    time_per_tile_sec: f32,
    seconds_timer: Timer,
    total_seconds: usize,
    score: usize,
    level: usize,
}
impl Game {
    fn init() -> Self {
        let mut tiles = vec![];
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
                tiles.push(Tile::new(col, row, TILE_BG_COLOR));
            }
        }
        let next_piece = Self::spawn_piece();
        Self {
            time_per_tile_sec: 0.5,
            tiles,
            piece: Self::spawn_piece(),
            next_piece,
            seconds_timer: Timer {
                elapsed: 0.0,
                duration: 1.0,
            },
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
        if self.paused || self.game_over {
            return;
        }

        self.level = self.score / 50; // Increase level after 50 points (5 rows)

        self.seconds_timer.elapsed += dt;
        if self.seconds_timer.elapsed >= self.seconds_timer.duration {
            self.seconds_timer.elapsed -= self.seconds_timer.duration;
            self.total_seconds += 1;
        }

        self.timer += dt;
        if self.timer >= self.time_per_tile_sec {
            self.timer -= self.time_per_tile_sec;
            let mut new_piece = self.piece.clone();
            for tile in &mut new_piece.tiles {
                tile.pos.1 = tile.pos.1 + 1;
            }

            if new_piece.is_valid(&self.tiles) {
                self.piece = new_piece;
            } else {
                // Convert piece tiles into fixed tiles in the board
                for tile in &mut self.piece.tiles {
                    let idx = pos_to_idx(tile.pos.0, tile.pos.1);
                    self.tiles[idx].color = tile.color;
                }

                // Check for complete rows
                let mut complete_rows = self.piece.tiles.iter().map(|t| t.pos.1).collect::<Vec<_>>();
                complete_rows.sort();
                complete_rows.dedup();
                for complete_row in complete_rows {
                    if TILES_X
                        == self
                            .tiles
                            .iter()
                            .filter(
                                |Tile {
                                     pos: (_, y),
                                     color,
                                 }| *y == complete_row && *color != TILE_BG_COLOR,
                            )
                            .count() as isize
                    {
                        // Remove complete row
                        self.tiles.iter_mut().filter(|t| t.pos.1 == complete_row).for_each(|t| t.color = TILE_BG_COLOR);
                        // Move occupied tiles above completed row
                        for row in (0..TILES_Y).rev() {
                            for col in (0..TILES_X).rev() {
                                let src_idx = pos_to_idx(col, row);
                                if row < complete_row && self.tiles[src_idx].color != TILE_BG_COLOR {
                                    let dst_idx = pos_to_idx(col, row + 1);
                                    self.tiles[dst_idx].color = self.tiles[src_idx].color;
                                    self.tiles[src_idx].color = TILE_BG_COLOR;
                                }
                            }
                        }
                        self.score += 10;
                    }
                }

                self.piece = self.next_piece.clone();
                self.next_piece = Self::spawn_piece();
                if !self.next_piece.is_valid(&self.tiles) {
                    self.game_over = true;
                    return;
                }
            }
        }

        if input.was_key_pressed(KeyId::Space) {
            let mut new_piece = self.piece.clone();
            Self::rotate_piece(&mut new_piece);
            if new_piece.is_valid(&self.tiles) {
                self.piece = new_piece;
            } else {
                // if we are out of bounds, adjust position
                if new_piece.tiles.iter().any(
                    |Tile {
                         pos: (x, _),
                         ..
                     }| { *x < 0 },
                ) {
                    let min = -new_piece.tiles.iter().map(|t| t.pos.0).min().unwrap();
                    new_piece.tiles.iter_mut().for_each(|t| t.pos.0 += min);
                    self.piece = new_piece;
                } else if new_piece.tiles.iter().any(
                    |Tile {
                         pos: (x, _),
                         ..
                     }| { *x > TILES_X - 1 },
                ) {
                    let max = new_piece.tiles.iter().map(|t| t.pos.0).max().unwrap() - (TILES_X - 1);
                    new_piece.tiles.iter_mut().for_each(|t| t.pos.0 -= max);
                    self.piece = new_piece;
                } else if new_piece.tiles.iter().any(
                    |Tile {
                         pos: (_, y),
                         ..
                     }| { *y < 0 },
                ) {
                    let min = -new_piece.tiles.iter().map(|t| t.pos.1).min().unwrap();
                    new_piece.tiles.iter_mut().for_each(|t| t.pos.1 += min);
                    self.piece = new_piece;
                }
            }
        }

        // Move Horizontally
        let x_delta = if input.was_key_pressed(KeyId::A) {
            -1
        } else if input.was_key_pressed(KeyId::D) {
            1
        } else {
            0
        };
        let mut new_piece = self.piece.clone();
        for tile in &mut new_piece.tiles {
            tile.pos.0 = tile.pos.0 + x_delta;
        }
        if new_piece.is_valid(&self.tiles) {
            self.piece = new_piece;
        }

        // Move Vertically
        if input.is_key_down(KeyId::S) {
            self.time_per_tile_sec = 0.05;
        } else {
            self.time_per_tile_sec = 0.5 - (0.05 * self.level as f32);
            //self.time_per_tile_sec = 0.5;
        }
        if input.was_key_pressed(KeyId::W) {
            while !self.piece.tiles.iter().any(
                |Tile {
                     pos: (x, y),
                     ..
                 }| *y == TILES_Y - 1 || !self.tiles[pos_to_idx(*x, *y + 1)].is_empty(),
            ) {
                for tile in &mut self.piece.tiles {
                    tile.pos.1 += 1;
                    tile.pos.1 = tile.pos.1.min(TILES_Y - 1);
                }
            }
            //self.paused = true;
        }
    }
    fn render(&self, cmd: &mut Vec<RenderCommand>) {
        let start_x = WIDTH / 2.0 - TILE_SIZE * (TILES_X / 2) as f32;
        let start_y = HEIGHT / 2.0 - TILE_SIZE * (TILES_Y / 2) as f32;
        for row in 0..TILES_Y {
            for col in 0..TILES_X {
                let idx = pos_to_idx(col, row);
                let tile = self.tiles[idx];
                vk_util::push_rect_color(
                    cmd,
                    Rect::offset_extent(
                        (start_x + col as f32 * TILE_SIZE, start_y + row as f32 * TILE_SIZE),
                        (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                    ),
                    0.2,
                    tile.color,
                );
            }
        }

        for tile in &self.piece.tiles {
            vk_util::push_rect_color(
                cmd,
                Rect::offset_extent(
                    (start_x + tile.pos.0 as f32 * TILE_SIZE, start_y + tile.pos.1 as f32 * TILE_SIZE),
                    (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                ),
                0.1,
                tile.color,
            );
        }

        vk_util::push_str_color(cmd, "Next", (start_x - 200.0, start_y), 0.0, 6.0, TEXT_COLOR, false);
        for tile in &self.next_piece.tiles {
            vk_util::push_rect_color(
                cmd,
                Rect::offset_extent(
                    (start_x - 300.0 + tile.pos.0 as f32 * TILE_SIZE, start_y + 100.0 + tile.pos.1 as f32 * TILE_SIZE),
                    (TILE_SIZE - 1.0, TILE_SIZE - 1.0),
                ),
                0.0,
                tile.color,
            );
        }
        vk_util::push_str_color(
            cmd,
            &format!("Score: {}", self.score),
            (start_x + 350.0, start_y),
            0.0,
            6.0,
            TEXT_COLOR,
            false,
        );
        vk_util::push_str_color(
            cmd,
            &format!("Time: {}", self.total_seconds),
            (start_x + 350.0, start_y + 100.0),
            0.0,
            6.0,
            TEXT_COLOR,
            false,
        );
        vk_util::push_str_color(
            cmd,
            &format!("Level: {}", self.level),
            (start_x + 350.0, start_y + 200.0),
            0.0,
            6.0,
            TEXT_COLOR,
            false,
        );

        if self.game_over {
            vk_util::push_str_centered_color(
                cmd,
                "Game Over",
                50.0,
                0.0,
                10.0,
                color::LIGHT_GREY,
                true,
                Rect::offset_extent((0.0, 0.0), (WIDTH, HEIGHT)),
            );
        }
    }

    fn rotate_piece(piece: &mut Piece) {
        match piece.tiles[0].color {
            c if c == color::CYAN => {
                if piece.tiles[0].pos.0 == piece.tiles[1].pos.0 {
                    // Vertical
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 2, piece.tiles[2].pos.1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                } else {
                    // Horizontal
                    piece.tiles[0].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 2);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                }
            }
            c if c == color::DARK_BLUE => {
                if piece.tiles[0].pos.0 < piece.tiles[3].pos.0 && piece.tiles[0].pos.1 < piece.tiles[3].pos.1 {
                    // Pointing up -> Pointing right
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else if piece.tiles[0].pos.0 > piece.tiles[3].pos.0 && piece.tiles[0].pos.1 < piece.tiles[3].pos.1 {
                    // Pointing right -> Pointing down
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 + 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                } else if piece.tiles[0].pos.0 > piece.tiles[3].pos.0 && piece.tiles[0].pos.1 > piece.tiles[3].pos.1 {
                    // Pointing down -> Pointing left
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 + 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                } else if piece.tiles[0].pos.0 < piece.tiles[3].pos.0 && piece.tiles[0].pos.1 > piece.tiles[3].pos.1 {
                    // Pointing left -> Pointing up
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                }
            }
            c if c == color::ORANGE => {
                if piece.tiles[0].pos.0 > piece.tiles[3].pos.0 && piece.tiles[0].pos.1 < piece.tiles[3].pos.1 {
                    // Pointing up -> Pointing right
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 + 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                } else if piece.tiles[0].pos.0 > piece.tiles[3].pos.0 && piece.tiles[0].pos.1 > piece.tiles[3].pos.1 {
                    // Pointing right -> Pointing down
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 + 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                } else if piece.tiles[0].pos.0 < piece.tiles[3].pos.0 && piece.tiles[0].pos.1 > piece.tiles[3].pos.1 {
                    // Pointing down -> Pointing left
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else if piece.tiles[0].pos.0 < piece.tiles[3].pos.0 && piece.tiles[0].pos.1 < piece.tiles[3].pos.1 {
                    // Pointing left -> Pointing up
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                }
            }
            c if c == color::RED => {
                if piece.tiles[0].pos.0 < piece.tiles[3].pos.0 {
                    // Horizontal
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else {
                    // Vertical
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                }
            }
            c if c == color::GREEN => {
                if piece.tiles[0].pos.0 > piece.tiles[3].pos.0 {
                    // Horizontal
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else {
                    // Vertical
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                }
            }
            c if c == color::PURPLE => {
                if piece.tiles[0].pos.1 < piece.tiles[2].pos.1 {
                    // Pointing up => Pointing right
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else if piece.tiles[0].pos.1 > piece.tiles[2].pos.1 {
                    // Pointing down => Pointing left
                    piece.tiles[0].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                } else if piece.tiles[0].pos.0 < piece.tiles[2].pos.0 {
                    // Pointing left => Pointing up
                    piece.tiles[0].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 - 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                } else if piece.tiles[0].pos.0 > piece.tiles[2].pos.0 {
                    // Pointing right => Pointing down
                    piece.tiles[0].pos = (piece.tiles[2].pos.0, piece.tiles[2].pos.1 + 1);
                    piece.tiles[1].pos = (piece.tiles[2].pos.0 - 1, piece.tiles[2].pos.1);
                    piece.tiles[3].pos = (piece.tiles[2].pos.0 + 1, piece.tiles[2].pos.1);
                }
            }
            _ => {}
        }
    }

    fn spawn_piece() -> Piece {
        #[rustfmt::skip]
        let colors = [
            color::CYAN,        // ____
            color::DARK_BLUE,   // |___
            color::ORANGE,      // ___|
            color::RED,         // __
                                //   |__
            color::GREEN,       //    __
                                // __|
            color::YELLOW,      // Square
            color::PURPLE,      // __|__
        ];
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time since UNIX_EPOCH")
            .as_secs() as usize;
        let mut rand = Rand::new(seed);
        let idx = rand.next_usize() % colors.len();
        let color = colors[idx];
        //println!("{}, {:?}", idx, color);
        match color {
            c if c == color::BLUE => Piece {
                tiles: vec![
                    Tile::new(3, 0, color),
                    Tile::new(4, 0, color),
                    Tile::new(5, 0, color),
                    Tile::new(6, 0, color),
                ],
            },
            c if c == color::DARK_BLUE => Piece {
                tiles: vec![
                    Tile::new(4, 0, color),
                    Tile::new(4, 1, color),
                    Tile::new(5, 1, color),
                    Tile::new(6, 1, color),
                ],
            },
            c if c == color::ORANGE => Piece {
                tiles: vec![
                    Tile::new(6, 0, color),
                    Tile::new(6, 1, color),
                    Tile::new(5, 1, color),
                    Tile::new(4, 1, color),
                ],
            },
            c if c == color::RED => Piece {
                tiles: vec![
                    Tile::new(4, 0, color),
                    Tile::new(5, 0, color),
                    Tile::new(5, 1, color),
                    Tile::new(6, 1, color),
                ],
            },
            c if c == color::GREEN => Piece {
                tiles: vec![
                    Tile::new(6, 0, color),
                    Tile::new(5, 0, color),
                    Tile::new(5, 1, color),
                    Tile::new(4, 1, color),
                ],
            },
            c if c == color::YELLOW => Piece {
                tiles: vec![
                    Tile::new(4, 0, color),
                    Tile::new(5, 0, color),
                    Tile::new(4, 1, color),
                    Tile::new(5, 1, color),
                ],
            },
            c if c == color::PURPLE => Piece {
                tiles: vec![
                    Tile::new(5, 0, color),
                    Tile::new(4, 1, color),
                    Tile::new(5, 1, color),
                    Tile::new(6, 1, color),
                ],
            },
            c => Piece {
                tiles: vec![Tile::new(3, 0, c), Tile::new(4, 0, c), Tile::new(5, 0, c), Tile::new(6, 0, c)],
            },
        }
    }
}

fn pos_to_idx(x: isize, y: isize) -> usize {
    (y * TILES_X + x) as usize
}

fn main() {
    let mut platform = Platform::init(Config {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        app_name: String::from("Tetris"),
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

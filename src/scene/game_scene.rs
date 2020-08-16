use crate::canvas::*;
use super::Scene;
use {rand, rand::Rng};
use libremarkable::input::{gpio, wacom, multitouch, ev, InputDevice, InputEvent};
use tetris_core::{Randomizer, Game, Size, Block, Action};
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::refresh::PartialRefreshMode;
use libremarkable::image::RgbImage;
use std::cell::RefCell;

struct OpionatedRandomizer {
    // Since the trait gives only immutable self,
    // we cant expect to modifiy any states easily.
    // This example is basicially a non thread-safe
    // Mutex that enforces rusts borrow-rules
    // dynamically at runtime.
    block_pool: RefCell<Vec<i32>>
}
impl OpionatedRandomizer {
    pub fn new() -> Self {
        Self { block_pool: RefCell::new(vec![]) }
    }
    fn actual_random_between(&self, first: i32, last: i32) -> i32 {
        rand::thread_rng().gen_range(first, last+1)
    }
    fn fillup(&self, sets: usize) {
        let mut block_pool = self.block_pool.borrow_mut();
        for _ in 0..sets {
            for random_number in 0..7 {
                block_pool.push(random_number);
            }
        }
    }
}
impl Randomizer for OpionatedRandomizer {
    fn random_between(&self, first: i32, last: i32) -> i32 {
        if first == 0 && last == 6 {
            // Basicially the only thing tetris_core will ever want

            // Fillup the pool again
            if self.block_pool.borrow().len() == 0 {
                self.fillup(2);
            }

            // Take and return random element from pool
            let len = self.block_pool.borrow().len();
            self.block_pool.borrow_mut().remove(self.actual_random_between(0, len as i32 - 1) as usize)
        }else {
            // Fallback
            self.actual_random_between(first, last)
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct StupidColor(u8, u8, u8);

impl StupidColor {
    pub fn from(r: f32, g: f32, b: f32) -> Self {
        StupidColor(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }
}

pub struct GameScene {
    game: Game,
    last_draw: Option<SystemTime>,
    game_size: Size,
    block_size: usize,
    last_blocks: HashMap<Point2<u8>, Block>,
    last_score: u64,
    textures: HashMap<StupidColor, RgbImage>,
}


impl GameScene {
    pub fn new(game_size: Size) -> Self {
        // Generate textures
        let block_size = 50;
        let mut textures: HashMap<StupidColor, RgbImage> = HashMap::new();
        let black = libremarkable::image::Rgb([0, 0, 0]);
        let white = libremarkable::image::Rgb([255, 255, 255]);
        let i: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if x * y % 5 == 0 { black } else { white }
        });
        let j: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if x % 5 == 0 { black } else { if y % 2 == 0 { black } else { white } }
        });
        let l: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if y % 5 == 0 { black } else { white }
        });
        let o: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if y * x % 10 > 3 { black } else { white }
        });
        let z: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if y % 5 == 0 { black } else { white }
        });
        let t: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if y * x * 3 % 10 == 0 { black } else { white }
        });
        let s: RgbImage = RgbImage::from_fn(block_size, block_size, |x,y| {
            if x * y % 5 != 0 { black } else { white }
        });
        textures.insert(StupidColor::from(108.0 / 255.0, 237.0 / 255.0, 238.0 / 255.0), i); // I-Block
        textures.insert(StupidColor::from(0.0, 33.0 / 255.0, 230.0 / 255.0), j); // J-Block
        textures.insert(StupidColor::from(229.0 / 255.0, 162.0 / 255.0, 67.0 / 255.0), l); // L-Block
        textures.insert(StupidColor::from(241.0 / 255.0, 238.0 / 255.0, 79.0 / 255.0), o); // O-Block
        textures.insert(StupidColor::from(110.0 / 255.0, 235.0 / 255.0, 71.0 / 255.0), z); // Z-Block
        textures.insert(StupidColor::from(146.0 / 255.0, 45.0 / 255.0, 231.0 / 255.0), t); // T-Block
        textures.insert(StupidColor::from(221.0 / 255.0, 47.0 / 255.0, 23.0 / 255.0), s); // S-Block


        Self {
            game: Game::new(&Size { width: 10, height: 22 }, Box::new(OpionatedRandomizer::new())),
            last_draw: None,
            game_size,
            block_size: block_size as usize,
            last_blocks: HashMap::new(),
            last_score: 0,
            textures,
        }
    }

    pub fn to_coords(&self, pos: (usize, usize)) -> (usize, usize) {
        let start_x = DISPLAYWIDTH as usize / 2 - self.game_size().width * self.block_size / 2;
        let start_y = DISPLAYHEIGHT as usize / 2 - self.game_size().height * self.block_size / 2;
        let offset = self.to_size(pos);
        (start_x + offset.0, start_y + offset.1)
    }
    
    pub fn to_size(&self, pos: (usize, usize)) -> (usize, usize) {
        (pos.0 * self.block_size, pos.1 * self.block_size)
    }

    pub fn game_size(&self) -> Size {
        self.game_size.clone()
    }

    pub fn field_start_i32(&self) -> Point2<i32> {
        let field_start = self.to_coords((0, 0));
        Point2 { x: field_start.0 as i32, y: field_start.1 as i32 }
    }

    pub fn field_start_u32(&self) -> Point2<u32> {
        let field_start = self.to_coords((0, 0));
        Point2 { x: field_start.0 as u32, y: field_start.1 as u32 }
    }

    pub fn field_size(&self) -> Vector2<u32> {
        let field_size = self.to_size((self.game_size.width, self.game_size.height));
        Vector2 { x: field_size.0 as u32, y: field_size.1 as u32 }
    }

    pub fn is_game_over(&self) -> bool {
        self.game.is_game_over()
    }

    pub fn get_score(&self) -> u64 {
        self.game.get_score()
    }

    fn draw_blocks(&mut self, canvas: &mut Canvas) -> bool {
        let mut blocks: HashMap<Point2<u8>, Block> = HashMap::new();
        for block in self.game.draw() {
            blocks.insert(Point2 { x: block.rect.origin.x as u8, y: block.rect.origin.y as u8 }, block);
        }

        let mut any_change = false;

        for y in 0..self.game_size().height {
            for x in 0..self.game_size().width {
                let pos = Point2 { x: x as u8, y: y as u8 };
                let was_filled = self.last_blocks.contains_key(&pos);
                let is_filled = blocks.contains_key(&pos);

                if was_filled != is_filled {
                    any_change = true;

                    // Change detected
                    let block_start = self.to_coords((x, y));
                    let block_size = self.to_size((1,1));

                    // Block went away
                    if is_filled {
                        let block = blocks.get(&pos).unwrap();
                        let color = StupidColor::from(block.color.red, block.color.green, block.color.blue);
                        if let Some(ref image) = self.textures.get(&color) {
                            canvas.framebuffer_mut().draw_image(image, Point2 { x: block_start.0 as i32, y: block_start.1 as i32 });
                        } else {
                            canvas.framebuffer_mut().fill_rect(
                                Point2 { x: block_start.0 as i32, y: block_start.1 as i32 },
                                Vector2 { x: block_size.0 as u32, y: block_size.1 as u32 },
                                color::BLACK
                            );
                        }
                    }else {
                        canvas.framebuffer_mut().fill_rect(
                            Point2 { x: block_start.0 as i32, y: block_start.1 as i32 },
                            Vector2 { x: block_size.0 as u32, y: block_size.1 as u32 },
                            color::WHITE
                        );
                    }
                }
            }
        }

        self.last_blocks = blocks;

        any_change
    }

    fn draw_score(&mut self, canvas: &mut Canvas) {
        let field_start = self.field_start_u32();
        let field_size = self.field_size();

        // Transition to white first because of fragments!
        let pos = Point2 {
            x: field_start.x as i32,
            y: (field_start.y + field_size.y + 3) as i32
        };
        let size = Vector2 {
            x: field_size.x,
            y: FONT_SIZE + 30
        };
        canvas.framebuffer_mut().fill_rect(pos, size, color::WHITE);
        canvas.update_partial(&mxcfb_rect::from(Point2 { x: pos.x as u32, y: pos.y as u32 }, size));

        const FONT_SIZE: u32 = 40;
        let affected_rect = canvas.draw_text(
            Point2 {
                x: Some((field_start.x + 10) as i32),
                y: Some((field_start.y + field_size.y + FONT_SIZE + 5) as i32)
            },
            &format!("Score: {}", self.get_score()),
            FONT_SIZE as f32,
        );
    }
}

impl Scene for GameScene {
    fn on_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::GPIO { event } => {
                if let gpio::GPIOEvent::Press { button } = event {
                    match button {
                        gpio::PhysicalButton::MIDDLE => self.game.perform(Action::Rotate),
                        gpio::PhysicalButton::LEFT => self.game.perform(Action::MoveLeft),
                        gpio::PhysicalButton::RIGHT => self.game.perform(Action::MoveRight),
                        gpio::PhysicalButton::POWER => self.game.perform(Action::MoveDown),
                        _ => { }
                    }
                }
            },
            _ => { }
        };
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if let Some(last_draw) = self.last_draw {
            // Advance physics
            self.game.update(last_draw.elapsed().unwrap().as_secs_f64());
        }else {
            // First frame
            canvas.clear();
            canvas.draw_text(Point2 { x: None, y: Some(self.field_start_i32().y - 50)}, "reTris", 200.0);

            let point = Point2 { x: self.field_start_i32().x - 2, y: self.field_start_i32().y - 2 };
            let vec = Vector2 { x: 2 + self.field_size().x + 2, y: 2 + self.field_size().y + 2 };
            canvas.framebuffer_mut().draw_rect(point, vec, 1, color::BLACK);
            canvas.update_full();
            self.draw_score(canvas);
        }
        self.last_draw = Some(SystemTime::now());

        let mut any_change = false;
        // Update score if changed
        if self.last_score != self.get_score() {
            self.last_score = self.get_score();
            self.draw_score(canvas);
            any_change = true;
        }
        
        any_change |= self.draw_blocks(canvas);

        if any_change {
            // If any change => A2 refresh the display
            canvas.update_partial(&mxcfb_rect { left: 0, top: 0, width: DISPLAYWIDTH as u32, height: DISPLAYHEIGHT as u32 });
        }
    }
}
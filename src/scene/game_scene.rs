use crate::canvas::*;
use super::Scene;
use {rand, rand::Rng};
use libremarkable::input::{gpio, wacom, multitouch, ev, InputDevice, InputEvent};
use tetris_core::{Randomizer, Game, Size, Block, Action};
use std::time::{SystemTime, Duration};
use std::collections::HashSet;
use libremarkable::framebuffer::common;
use libremarkable::framebuffer::refresh::PartialRefreshMode;

struct RandomizerImpl;
impl Randomizer for RandomizerImpl {
    fn random_between(&self, first: i32, last: i32) -> i32 {
        rand::thread_rng().gen_range(first, last)
    }
}

pub struct GameScene {
    game: Game,
    last_draw: Option<SystemTime>,
    game_size: Size,
    block_size: usize,
    last_blocks: HashSet<Point2<u8>>,
    last_score: u64,
}

impl GameScene {
    pub fn new(game_size: Size) -> Self {
        Self {
            game: Game::new(&Size { width: 10, height: 22 }, Box::new(RandomizerImpl)),
            last_draw: None,
            game_size,
            block_size: 50,
            last_blocks: HashSet::new(),
            last_score: 0,
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
        let mut blocks: HashSet<Point2<u8>> = HashSet::new();
        for block in self.game.draw() {
            blocks.insert(Point2 { x: block.rect.origin.x as u8, y: block.rect.origin.y as u8 });
        }

        let mut any_change = false;

        for y in 0..self.game_size().height {
            for x in 0..self.game_size().width {
                let pos = Point2 { x: x as u8, y: y as u8 };
                let was_filled = self.last_blocks.contains(&pos);
                let is_filled = blocks.contains(&pos);

                if was_filled != is_filled {
                    any_change = true;

                    // Change detected
                    let block_start = self.to_coords((x, y));
                    let block_size = self.to_size((1,1));

                    // Block went away
                    canvas.framebuffer_mut().fill_rect(
                        Point2 { x: block_start.0 as i32, y: block_start.1 as i32 },
                        Vector2 { x: block_size.0 as u32, y: block_size.1 as u32 },
                        if is_filled { color::BLACK } else { color::WHITE }
                    );
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
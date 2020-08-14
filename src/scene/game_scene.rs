use crate::canvas::*;
use super::Scene;
use {rand, rand::Rng};
use libremarkable::input::{gpio, wacom, multitouch, ev, InputDevice, InputEvent};
use tetris_core::{Randomizer, Game, Size, Block, Action};
use std::time::{SystemTime, Duration};

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
}

impl GameScene {
    pub fn new(game_size: Size) -> Self {
        Self {
            game: Game::new(&Size { width: 10, height: 22 }, Box::new(RandomizerImpl)),
            last_draw: None,
            game_size,
            block_size: 50,
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
}

impl Scene for GameScene {
    fn on_activate(&mut self, canvas: &mut Canvas) {

    }

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
        }
        self.last_draw = Some(SystemTime::now());


        canvas.clear();
        {
            let point = self.field_start_i32();
            let vec = self.field_size();
            canvas.framebuffer_mut().draw_rect(point, vec, 1, color::BLACK);
        }

        for block in self.game.draw() {
            let block_start = self.to_coords((block.rect.origin.x as usize, block.rect.origin.y as usize));
            let block_size = self.to_size((block.rect.size.width as usize, block.rect.size.height as usize));
            canvas.framebuffer_mut().fill_rect(
                Point2 { x: block_start.0 as i32, y: block_start.1 as i32 },
                Vector2 { x: block_size.0 as u32, y: block_size.1 as u32 },
                color::BLACK
            );
        }

        let field_start = self.field_start_u32();
        let field_size = self.field_size();
        const FONT_SIZE: u32 = 40;
        canvas.framebuffer_mut().draw_text(
            Point2 { x: (field_start.x + 10) as f32, y: (field_start.y + field_size.y + FONT_SIZE + 5) as f32 },
            format!("Score: {}", self.game.get_score()),
            FONT_SIZE as f32,
            color::BLACK,
            false
        );

        let point = Point2 { x: self.field_start_u32().x - 5, y: self.field_start_u32().y - 5 };
        let vec = self.field_size() + Vector2 { x: 5 + 5, y: 5 + FONT_SIZE + 5 };
        canvas.update_partial(&mxcfb_rect::from(point, vec));
    }
}
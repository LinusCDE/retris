use super::Scene;
use crate::canvas::*;
use crate::swipe::{SwipeTracker, Swipe, Trigger, Direction};
use fxhash::FxHashMap;
use libremarkable::image::RgbImage;
use libremarkable::input::{gpio, multitouch, multitouch::Finger, InputEvent};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::SystemTime;
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use {rand, rand::Rng};
use tetris_core::{Randomizer, Game, Size, Block, Action};

struct OpionatedRandomizer {
    /// Since the trait gives only immutable self,
    /// we cant expect to modifiy any states easily.
    /// This example is basicially a non thread-safe
    /// Mutex that enforces rusts borrow-rules
    /// dynamically at runtime.
    block_pool: RefCell<Vec<i32>>,
    /// A count of how often a value from block_pool
    /// was returned to keep track of differing blocks.
    block_id: Arc<AtomicU32>,
}
impl OpionatedRandomizer {
    pub fn new(block_id: Arc<AtomicU32>) -> Self {
        Self { block_pool: RefCell::new(vec![]), block_id }
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
            self.block_id.fetch_add(1, Ordering::Relaxed); // block_id += 1
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
    speed: f64,
    last_draw: Option<SystemTime>,
    game_size: Size,
    block_size: usize,
    last_blocks: HashMap<Point2<u8>, Block>,
    last_score: u64,
    textures: HashMap<StupidColor, RgbImage>,
    swipe_tracker: SwipeTracker,
    last_pressed_finger: Option<(Finger, SystemTime)>,
    play_pause_button_hitbox: Option<mxcfb_rect>,
    back_button_hitbox: Option<mxcfb_rect>,
    left_button_hitbox: Option<mxcfb_rect>,
    right_button_hitbox: Option<mxcfb_rect>,
    is_paused: bool,
    pub back_button_pressed: bool,
    block_id: Arc<AtomicU32>,
    finger_controls_which_block: FxHashMap<i32/* Tracking id */, u32/* Block id */>,
}


impl GameScene {
    pub fn new(game_size: Size, speed: f64) -> Self {
        // Generate textures
        let block_size = 50;
        let mut textures: HashMap<StupidColor, RgbImage> = HashMap::new();
        let black = libremarkable::image::Rgb([0, 0, 0]);
        let white = libremarkable::image::Rgb([255, 255, 255]);
        let img_i: RgbImage = RgbImage::from_fn(block_size, block_size, |x, y|
            if x * y % 5 == 0 { black } else { white }
        );
        let img_j: RgbImage = RgbImage::from_fn(block_size, block_size, |x, y|
            if x % 5 == 0 || y % 2 == 0 { black } else { white }
        );
        let img_l: RgbImage = RgbImage::from_fn(block_size, block_size, |_, y|
            if y % 5 == 0 { black } else { white }
        );
        let img_o: RgbImage = RgbImage::from_fn(block_size, block_size, |x, y|
            if y * x % 10 > 3 { black } else { white }
        );
        let img_z: RgbImage = RgbImage::from_fn(block_size, block_size, |_, y|
            if y % 5 == 0 { black } else { white }
        );
        let img_t: RgbImage = RgbImage::from_fn(block_size, block_size, |x, y|
            if y * x * 3 % 10 == 0 { black } else { white }
        );
        let img_s: RgbImage = RgbImage::from_fn(block_size, block_size, |x, y|
            if x * y % 5 != 0 { black } else { white }
        );
        textures.insert(StupidColor::from(108.0 / 255.0, 237.0 / 255.0, 238.0 / 255.0), img_i); // I-Block
        textures.insert(StupidColor::from(0.0, 33.0 / 255.0, 230.0 / 255.0), img_j); // J-Block
        textures.insert(StupidColor::from(229.0 / 255.0, 162.0 / 255.0, 67.0 / 255.0), img_l); // L-Block
        textures.insert(StupidColor::from(241.0 / 255.0, 238.0 / 255.0, 79.0 / 255.0), img_o); // O-Block
        textures.insert(StupidColor::from(110.0 / 255.0, 235.0 / 255.0, 71.0 / 255.0), img_z); // Z-Block
        textures.insert(StupidColor::from(146.0 / 255.0, 45.0 / 255.0, 231.0 / 255.0), img_t); // T-Block
        textures.insert(StupidColor::from(221.0 / 255.0, 47.0 / 255.0, 23.0 / 255.0), img_s); // S-Block

        let block_id = Arc::new(AtomicU32::new(0));
        Self {
            game: Game::new(&Size { width: 10, height: 22 }, Box::new(OpionatedRandomizer::new(block_id.clone()))),
            speed,
            last_draw: None,
            game_size,
            block_size: block_size as usize,
            last_blocks: HashMap::new(),
            last_score: 0,
            textures,
            swipe_tracker: SwipeTracker::new(),
            last_pressed_finger: None,
            play_pause_button_hitbox: None,
            back_button_hitbox: None,
            left_button_hitbox: None,
            right_button_hitbox: None,
            is_paused: false,
            back_button_pressed: false,
            block_id,
            finger_controls_which_block: FxHashMap::default(),
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

    /// Draws all blocks and returns a list of all rects that were changed
    /// and whether they are now filled or not.
    fn draw_blocks(&mut self, canvas: &mut Canvas) -> Vec<(mxcfb_rect, bool)> {
        let mut blocks: HashMap<Point2<u8>, Block> = HashMap::new();
        for block in self.game.draw() {
            blocks.insert(Point2 { x: block.rect.origin.x as u8, y: block.rect.origin.y as u8 }, block);
        }

        let mut changed_rects: Vec<(mxcfb_rect, bool)> = vec![];

        for y in 0..self.game_size().height {
            for x in 0..self.game_size().width {
                let pos = Point2 { x: x as u8, y: y as u8 };
                let was_filled = self.last_blocks.contains_key(&pos);
                let is_filled = blocks.contains_key(&pos);

                if was_filled != is_filled {
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

                    changed_rects.push((
                        mxcfb_rect {
                            left: block_start.0 as u32,
                            top: block_start.1 as u32,
                            width: block_size.0 as u32,
                            height: block_size.1 as u32,
                        },
                        is_filled
                    ));
                }
            }
        }

        self.last_blocks = blocks;

        changed_rects
    }

    fn draw_score(&mut self, canvas: &mut Canvas) -> mxcfb_rect {
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
        canvas.draw_text(
            Point2 {
                x: Some((field_start.x + 10) as i32),
                y: Some((field_start.y + field_size.y + FONT_SIZE + 5) as i32)
            },
            &format!("Score: {}", self.get_score()),
            FONT_SIZE as f32,
        )
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
            InputEvent::MultitouchEvent { event } => {

                // Taps and buttons
                match event {
                    multitouch::MultitouchEvent::Press { finger } => {
                        self.last_pressed_finger = Some((finger, SystemTime::now()));
                        // This finger can only control the current block with swipes
                        self.finger_controls_which_block.insert(finger.tracking_id, self.block_id.load(Ordering::Relaxed));
                    },
                    multitouch::MultitouchEvent::Release { finger: up_finger } => {
                        if let Some((down_finger, down_when)) = self.last_pressed_finger {
                            if down_finger.tracking_id == up_finger.tracking_id
                               && down_when.elapsed().unwrap().as_millis() < 300 {
                                let x_dist = up_finger.pos.x as i32 - down_finger.pos.x as i32;
                                let y_dist = up_finger.pos.y as i32 - down_finger.pos.y as i32;
                                let dist = (x_dist.pow(2) as f32 + y_dist.pow(2) as f32).sqrt();
                                if dist < 20.0 {
                                    // Short tap recognized

                                    if self.play_pause_button_hitbox.is_some() && Canvas::is_hitting(up_finger.pos, self.play_pause_button_hitbox.unwrap()) {
                                        // Button: Pause
                                        self.is_paused = ! self.is_paused;
                                    } else if self.back_button_hitbox.is_some() && Canvas::is_hitting(up_finger.pos, self.back_button_hitbox.unwrap()) {
                                        // Button: Main Menu
                                        self.back_button_pressed = true;
                                    } else if self.left_button_hitbox.is_some() && Canvas::is_hitting(up_finger.pos, self.left_button_hitbox.unwrap()) {
                                        // Button: «
                                        self.game.perform(Action::MoveLeft);
                                    } else if self.right_button_hitbox.is_some() && Canvas::is_hitting(up_finger.pos, self.right_button_hitbox.unwrap()) {
                                        // Button: »
                                        self.game.perform(Action::MoveRight);
                                    }else {
                                        // Somewhere else
                                        self.game.perform(Action::Rotate);
                                    }
                                }
                            }
                        }
                    },
                    _ => { }
                }

                // Movement (swipes)
                const SWIPES: [Swipe; 3] = [
                    Swipe { direction: Direction::Down, trigger: Trigger::Completed },
                    Swipe { direction: Direction::Left, trigger: Trigger::MinDistance(50) },
                    Swipe { direction: Direction::Right, trigger: Trigger::MinDistance(50) },
                ];

                let tracking_id = &event.finger().unwrap().tracking_id;
                if let Some(swipe) = self.swipe_tracker.detect(event, &SWIPES) {
                    if self.finger_controls_which_block.get(&tracking_id) == Some(&self.block_id.load(Ordering::Relaxed)) { // Is current?
                        match swipe.direction {
                            Direction::Left => self.game.perform(Action::MoveLeft),
                            Direction::Right => self.game.perform(Action::MoveRight),
                            Direction::Down => {
                                // Push all the way down
                                for _ in 0..self.game_size.height {
                                    self.game.perform(Action::MoveDown);
                                }
                            }
                            Direction::Up => self.game.perform(Action::Rotate),
                        }
                    }
                }

                if let multitouch::MultitouchEvent::Release { .. } = event {
                    self.finger_controls_which_block.remove(&tracking_id);
                }
            }
            _ => { }
        };
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if let Some(last_draw) = self.last_draw {
            // Advance physics
            if ! self.is_paused {
                if let Ok(elapsed) = last_draw.elapsed() { // May fail due to clock drift (see docs of elapsed())
                    self.game.update(elapsed.as_secs_f64() * self.speed);
                }
            }
        }else {
            // First frame
            canvas.clear();
            canvas.draw_text(Point2 { x: None, y: Some(self.field_start_i32().y - 50)}, "reTris", 200.0);

            let point = Point2 { x: self.field_start_i32().x - 2, y: self.field_start_i32().y - 2 };
            let vec = Vector2 { x: 2 + self.field_size().x + 2, y: 2 + self.field_size().y + 2 };
            canvas.framebuffer_mut().draw_rect(point, vec, 1, color::BLACK);

            self.play_pause_button_hitbox = Some(canvas.draw_button(Point2 { x: Some(50), y: Some(75) }, "Pause", 50.0, 10, 20));
            self.back_button_hitbox = Some(canvas.draw_button(Point2 {
                x: Some(self.play_pause_button_hitbox.unwrap().left as i32 + self.play_pause_button_hitbox.unwrap().width as i32 + 50),
                y: Some(75)
            }, "Main Menu", 50.0, 10, 20));

            let lr_y_pos = 1780;
            let lr_x_margin = 75;
            let lr_vgap = 50;
            let lr_hgap = 50;
            let lr_font_size = 100.0;
            if ! crate::CLI_OPTS.no_arrow_buttons {
                self.left_button_hitbox = Some(canvas.draw_button(Point2 { x: Some(lr_x_margin), y: Some(lr_y_pos) }, "«", lr_font_size, lr_vgap, lr_hgap));
                self.right_button_hitbox = Some(canvas.draw_button(Point2 {
                    x: Some(DISPLAYWIDTH as i32 + lr_hgap as i32 - (self.left_button_hitbox.unwrap().left as i32 + self.left_button_hitbox.unwrap().width as i32)),
                    y: Some(lr_y_pos)
                }, "»", lr_font_size, lr_vgap, lr_hgap));
            }

            canvas.update_full();
            self.draw_score(canvas);
        }
        self.last_draw = Some(SystemTime::now());

        // Update score if changed
        if self.last_score != self.get_score() {
            self.last_score = self.get_score();
            let rect = self.draw_score(canvas);
            canvas.update_partial(&rect);
        }

        let block_changes = self.draw_blocks(canvas);
        if block_changes.len() > 0 {
            // Not sure if doing seperate transitions is a good or bad thing on either
            // rM1 or rM2. On the rM1 it seems to reduce some artifacts.
            //
            // Waiting for refreshes on the rM2 is currently stubbed by rm2fb. On the rM1
            // it takes about 350ms for each, making the game lag when waiting.

            // Do all white -> black transitions first (they are faster)
            block_changes
                .iter()
                .filter(|(_, filled)| *filled)
                .for_each(|(rect, _)| canvas.update_partial_mono(rect));

            // Do all black -> white transitions (they take longer anyway)
            block_changes
                .iter()
                .filter(|(_, filled)| !*filled)
                .for_each(|(rect, _)| canvas.update_partial_mono(rect));
        }
    }
}

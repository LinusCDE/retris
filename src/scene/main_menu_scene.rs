use super::Scene;
use crate::canvas::*;
use libremarkable::input::{InputEvent, multitouch::MultitouchEvent};

pub struct MainMenuScene {
    drawn: bool,
    
    play_button_hitbox: Option<mxcfb_rect>,
    pub play_button_pressed: bool,

    exit_button_hitbox: Option<mxcfb_rect>,
    pub exit_button_pressed: bool,

    score: Option<u64>,
}

impl MainMenuScene {
    pub fn new(score: Option<u64>) -> Self {
        Self {
            drawn: false,
            play_button_hitbox: None,
            play_button_pressed: false,
            exit_button_hitbox: None,
            exit_button_pressed: false,
            score,
        }
    }
}

impl Scene for MainMenuScene {
    fn draw(&mut self, canvas: &mut Canvas) {
        if self.drawn {
            return;
        }
        self.drawn = true;

        canvas.clear();
        canvas.draw_text(Point2 { x: None, y: Some(500)}, "reTris", 400.0);

        if let Some(score) = self.score {
               canvas.draw_text(Point2 { x: None, y: Some(700)}, "Game Over!", 75.0);
               canvas.draw_text(Point2 { x: None, y: Some(775)}, &format!("Score: {}", score), 75.0);
        }
        
        self.play_button_hitbox = Some(canvas.draw_button(Point2 { x: None, y: Some(1100) }, "PLAY", 125.0, 25, 50));
        self.exit_button_hitbox = Some(canvas.draw_button(Point2 { x: None, y: Some(1300) }, "EXIT", 125.0, 25, 50));
        
        canvas.update_full();
    }

    fn on_input(&mut self, event: InputEvent) {
        if let InputEvent::MultitouchEvent { event } = event {
            if let MultitouchEvent::Press { finger, .. } = event {
                let position = finger.pos;
                if self.play_button_hitbox.is_some() && Canvas::is_hitting(position, self.play_button_hitbox.unwrap()) {
                    self.play_button_pressed = true;
                }
                if self.exit_button_hitbox.is_some() && Canvas::is_hitting(position, self.exit_button_hitbox.unwrap()) {
                    self.exit_button_pressed = true;
                }
            }
        }
    }
}
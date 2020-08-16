use crate::canvas::*;
use super::Scene;
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


    fn draw_button(&mut self, canvas: &mut Canvas, pos: Point2<Option<i32>>, text: &str) -> mxcfb_rect {
        let font_size = 125.0;
        let vgap = 25 as u32;
        let hgap = 50 as u32;
        let text_rect = canvas.draw_text(pos, text, font_size);
        canvas.draw_rect(
            Point2 { x: Some((text_rect.left - hgap) as i32), y: Some((text_rect.top - vgap) as i32) }, 
            Vector2 { x: hgap + text_rect.width + hgap, y: vgap + text_rect.height + vgap },
            5
        )
    }
}

fn is_hitting(pos: Point2<u16>, hitbox: mxcfb_rect) -> bool {
    (pos.x as u32) >= hitbox.left && (pos.x as u32) < (hitbox.left + hitbox.width) &&
    (pos.y as u32) >= hitbox.top && (pos.y as u32) < (hitbox.top + hitbox.height)
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
        
        self.play_button_hitbox = Some(self.draw_button(canvas, Point2 { x: None, y: Some(1100) }, "PLAY"));
        self.exit_button_hitbox = Some(self.draw_button(canvas, Point2 { x: None, y: Some(1300) }, "EXIT"));
        
        canvas.update_full();
    }

    fn on_input(&mut self, event: InputEvent) {
        if let InputEvent::MultitouchEvent { event } = event {
            if let MultitouchEvent::Press { finger, .. } = event {
                let position = finger.pos;
                if self.play_button_hitbox.is_some() && is_hitting(position, self.play_button_hitbox.unwrap()) {
                    self.play_button_pressed = true;
                }
                if self.exit_button_hitbox.is_some() && is_hitting(position, self.exit_button_hitbox.unwrap()) {
                    self.exit_button_pressed = true;
                }
            }
        }
    }
}
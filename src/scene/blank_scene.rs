use crate::canvas::Canvas;
use super::Scene;

pub struct BlankScene {
    blanked_out: bool
}

impl BlankScene {
    pub fn new() -> Self {
        Self { blanked_out: false }
    }
}

impl Scene for BlankScene {
    fn on_activate(&mut self, canvas: &mut Canvas) {
        self.blanked_out = false;
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if ! self.blanked_out {
            canvas.clear();
            canvas.update_full();
            self.blanked_out = true;
        }
    }
}
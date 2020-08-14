mod game_scene;
mod blank_scene;

pub use game_scene::GameScene;
pub use blank_scene::BlankScene;

use crate::canvas::Canvas;
use libremarkable::input::InputEvent;

pub trait Scene {
    fn on_activate(&mut self, canvas: &mut Canvas) { }
    fn on_input(&mut self, event: InputEvent) { }
    fn draw(&mut self, canvas: &mut Canvas);
}
mod game_scene;
mod blank_scene;
mod main_menu_scene;

pub use game_scene::GameScene;
pub use blank_scene::BlankScene;
pub use main_menu_scene::MainMenuScene;

use crate::canvas::Canvas;
use libremarkable::input::InputEvent;
use downcast_rs::Downcast;

pub trait Scene: Downcast {
    fn on_input(&mut self, event: InputEvent) { }
    fn draw(&mut self, canvas: &mut Canvas);
}
impl_downcast!(Scene);

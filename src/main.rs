#[macro_use]
extern crate downcast_rs;

mod canvas;
mod scene;


use libremarkable::input::{InputDevice, InputEvent, ev::EvDevContext};
use tetris_core::Size;
use std::time::{SystemTime, Duration};
use std::thread::sleep;

use scene::*;
use canvas::Canvas;


fn main() {
    let mut canvas = Canvas::new();

    let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();
    EvDevContext::new(InputDevice::GPIO, input_tx.clone()).start();
    EvDevContext::new(InputDevice::Multitouch, input_tx.clone()).start();
    //EvDevContext::new(InputDevice::Wacom, input_tx.clone()).start();
    const FPS: u16 = 20;
    const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

    let mut current_scene: Box<dyn Scene> = Box::new(MainMenuScene::new(None));

    loop {
        let before_input = SystemTime::now();
        for event in input_rx.try_iter() {
            current_scene.on_input(event);        
        }

        current_scene.draw(&mut canvas);
        current_scene = update(current_scene, &mut canvas);
        

        // Wait remaining frame time
        let elapsed = before_input.elapsed().unwrap();
        if elapsed < FRAME_DURATION {
            sleep(FRAME_DURATION - elapsed);
        }
    }
}

fn update(scene: Box<dyn Scene>, canvas: &mut Canvas) -> Box<dyn Scene> {
    if let Some(game_scene) = scene.downcast_ref::<GameScene>() {
        if game_scene.is_game_over() {
            return Box::new(MainMenuScene::new(Some(game_scene.get_score())))
        }
    }else if let Some(main_menu_scene) = scene.downcast_ref::<MainMenuScene>() {
        if main_menu_scene.play_button_pressed {
            return Box::new(GameScene::new(Size { width: 10, height: 22 }));
        }else if main_menu_scene.exit_button_pressed {
            canvas.clear();
            canvas.update_full();
            std::process::exit(0);
        }
    }
    scene
}
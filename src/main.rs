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
    /*EvDevContext::new(InputDevice::Wacom, input_tx.clone()).start();
    EvDevContext::new(InputDevice::Multitouch, input_tx.clone()).start();*/
    const FPS: u16 = 20;
    const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

    let mut current_scene: Box<dyn Scene> = Box::new(GameScene::new(Size { width: 10, height: 22 }));

    loop {
        let before_input = SystemTime::now();
        if let Ok(event) = input_rx.recv_timeout(FRAME_DURATION) {
            current_scene.on_input(event);
        }

        current_scene.draw(&mut canvas);

        // At least every 100 ms
        let elapsed = before_input.elapsed().unwrap();
        if elapsed < FRAME_DURATION {
            sleep(FRAME_DURATION - elapsed);
        }
    }
}
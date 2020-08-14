mod canvas;

use {rand, rand::Rng};
use libremarkable::input::{gpio, wacom, multitouch, ev, InputDevice, InputEvent};
use tetris_core::{Randomizer, Game, Size, Block, Action};
use canvas::{Canvas, mxcfb_rect};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{SystemTime, Duration};
use std::thread::sleep;

struct RandomizerImpl;
impl Randomizer for RandomizerImpl {
    fn random_between(&self, first: i32, last: i32) -> i32 {
        rand::thread_rng().gen_range(first, last)
    }
}

fn main() {
    let mut canvas = Canvas::new(Size { width: 10, height: 22 });
    let mut game = Game::new(&canvas.game_size(), Box::new(RandomizerImpl));

    let (input_tx, input_rx) = channel::<InputEvent>();
    ev::EvDevContext::new(InputDevice::GPIO, input_tx.clone()).start();
    /*ev::EvDevContext::new(InputDevice::Wacom, input_tx.clone()).start();
    ev::EvDevContext::new(InputDevice::Multitouch, input_tx.clone()).start();*/
    const FPS: u16 = 20;
    const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

    loop {
        let before_input = SystemTime::now();
        if let Ok(event) = input_rx.recv_timeout(FRAME_DURATION) {
            match event {
                InputEvent::GPIO { event } => {
                    if let gpio::GPIOEvent::Press { button } = event {
                        match button {
                            gpio::PhysicalButton::MIDDLE => game.perform(Action::Rotate),
                            gpio::PhysicalButton::LEFT => game.perform(Action::MoveLeft),
                            gpio::PhysicalButton::RIGHT => game.perform(Action::MoveRight),
                            gpio::PhysicalButton::POWER => game.perform(Action::MoveDown),
                            _ => { }
                        }
                    }
                },
                _ => { }
            }
        }

        // At least every 100 ms
        let elapsed = before_input.elapsed().unwrap();
        if elapsed < FRAME_DURATION {
            sleep(FRAME_DURATION - elapsed);
        }

        game.update(before_input.elapsed().unwrap().as_secs_f64());
        canvas.draw_game(&game);
    }
}
#[macro_use]
extern crate downcast_rs;

mod canvas;
mod scene;
mod swipe;

use clap::Parser;
use crate::canvas::Canvas;
use crate::scene::*;
use libremarkable::input::{InputDevice, InputEvent, ev::EvDevContext};
use std::process::Command;
use std::sync::LazyLock;
use std::time::{Instant, Duration};
use std::thread::sleep;
use tetris_core::Size;

#[derive(Parser)]
#[clap(version, author, help_template = "{before-help}{name} {version} - by {author}\n\n{all-args}{after-help}")]
pub struct Opts {
    /// Stop xochitl service when a xochitl process is found. Useful when running without any launcher.
    #[clap(long, short = 'X')]
    kill_xochitl: bool,

    /// Don't display the left and right software arrow buttons.
    #[clap(long, short = 'A')]
    no_arrow_buttons: bool,
}

pub static CLI_OPTS: LazyLock<Opts> = LazyLock::new(|| Opts::parse());

fn main() {
    let only_exit_to_xochitl = if ! CLI_OPTS.kill_xochitl {
        false
    }else if let Ok(status) = Command::new("pidof").arg("xochitl").status() {
        if status.code().unwrap() == 0 {
            Command::new("systemctl").arg("stop").arg("xochitl").status().ok();
            println!("Xochitl was found and killed. You may only exit by starting Xochitl again.");
            true
        }else { false }
    } else { false };

    let mut canvas = Canvas::new();

    let (input_tx, input_rx) = std::sync::mpsc::channel::<InputEvent>();
    EvDevContext::new(InputDevice::GPIO, input_tx.clone()).start();
    EvDevContext::new(InputDevice::Multitouch, input_tx).start();
    //EvDevContext::new(InputDevice::Wacom, input_tx.clone()).start();
    const FPS: u16 = 30;
    const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

    let mut current_scene: Box<dyn Scene> = Box::new(MainMenuScene::new(None, only_exit_to_xochitl));

    loop {
        let before_input = Instant::now();
        for event in input_rx.try_iter() {
            current_scene.on_input(event);        
        }

        current_scene.draw(&mut canvas);
        current_scene = update(current_scene, &mut canvas, only_exit_to_xochitl);

        // Wait remaining frame time
        let elapsed = before_input.elapsed();
        if elapsed < FRAME_DURATION {
            sleep(FRAME_DURATION - elapsed);
        }
    }
}

fn update(scene: Box<dyn Scene>, canvas: &mut Canvas, only_exit_to_xochitl: bool) -> Box<dyn Scene> {
    if let Some(game_scene) = scene.downcast_ref::<GameScene>() {
        if game_scene.is_game_over() {
            return Box::new(MainMenuScene::new(Some(game_scene.get_score()), only_exit_to_xochitl));
        }else if game_scene.back_button_pressed {
            return Box::new(MainMenuScene::new(None, only_exit_to_xochitl));
        }
    }else if let Some(main_menu_scene) = scene.downcast_ref::<MainMenuScene>() {
        if main_menu_scene.play_easy_button_pressed {
            return Box::new(GameScene::new(Size { width: 10, height: 22 }, 0.5));
        }else if main_menu_scene.play_normal_button_pressed {
            return Box::new(GameScene::new(Size { width: 10, height: 22 }, 1.0));
        }else if main_menu_scene.play_hard_button_pressed {
            return Box::new(GameScene::new(Size { width: 10, height: 22 }, 1.5));
        }else if main_menu_scene.exit_xochitl_button_pressed {
            canvas.clear();
            canvas.update_full();
            Command::new("systemctl").arg("start").arg("xochitl").status().ok();
            std::process::exit(0);
        }else if main_menu_scene.exit_button_pressed {
            canvas.clear();
            canvas.update_full();
            std::process::exit(0);
        }
    }
    scene
}

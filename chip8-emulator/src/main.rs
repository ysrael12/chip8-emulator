mod chip8;
mod inout;
mod memory;

use gilrs::{Button, Gilrs};
use minifb::{Key, Scale, Window, WindowOptions};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

//how many cpu cycles per 60hz frame (~600 instructions per second)
const CYCLES_PER_FRAME: usize = 10;

//chip8 hexadecimal keypad mapped to the left side of a qwerty keyboard
const KEY_MAP: [(Key, u8); 16] = [
    (Key::Key1, 0x1),
    (Key::Key2, 0x2),
    (Key::Key3, 0x3),
    (Key::Key4, 0xC),
    (Key::Q, 0x4),
    (Key::W, 0x5),
    (Key::E, 0x6),
    (Key::R, 0xD),
    (Key::A, 0x7),
    (Key::S, 0x8),
    (Key::D, 0x9),
    (Key::F, 0xE),
    (Key::Z, 0xA),
    (Key::X, 0x0),
    (Key::C, 0xB),
    (Key::V, 0xF),
];

//xbox controller mapped to the chip8 keypad (covers all 16 keys)
const GAMEPAD_MAP: [(Button, u8); 16] = [
    (Button::DPadUp, 0x2),
    (Button::DPadLeft, 0x4),
    (Button::DPadRight, 0x6),
    (Button::DPadDown, 0x8),
    (Button::South, 0x5),         //A
    (Button::East, 0x0),          //B
    (Button::West, 0x1),          //X
    (Button::North, 0x3),         //Y
    (Button::LeftTrigger, 0x7),   //LB
    (Button::RightTrigger, 0x9),  //RB
    (Button::LeftTrigger2, 0xC),  //LT
    (Button::RightTrigger2, 0xD), //RT
    (Button::Select, 0xE),        //view
    (Button::Start, 0xF),         //menu
    (Button::LeftThumb, 0xA),     //left stick click
    (Button::RightThumb, 0xB),    //right stick click
];

fn main() {
    //the rom path comes from the command line
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: chip8-emulator <rom file>");
        std::process::exit(1);
    });
    let rom = std::fs::read(&rom_path).unwrap_or_else(|error| {
        eprintln!("could not read rom '{}': {}", rom_path, error);
        std::process::exit(1);
    });

    let mut chip8 = chip8::Chip8::new();
    chip8.load_program(&rom);

    let mut window = Window::new(
        "Chip8 rust emulator",
        inout::DISPLAY_WIDTH,
        inout::DISPLAY_HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .expect("could not open the window");
    //run the loop at 60 frames per second
    window.set_target_fps(60);

    let mut frame_buffer = [0u32; inout::DISPLAY_WIDTH * inout::DISPLAY_HEIGHT];
    //true while a beep thread is playing, so we do not stack beeps
    let beeping = Arc::new(AtomicBool::new(false));

    //gamepad support is optional: without one the keyboard still works
    let mut gilrs = Gilrs::new()
        .map_err(|error| eprintln!("gamepad support disabled: {}", error))
        .ok();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        //update the keyboard state
        for (key, chip8_key) in KEY_MAP {
            chip8.keyboard.key[chip8_key as usize] = window.is_key_down(key);
        }

        //combine with the gamepad buttons of every connected controller
        if let Some(gilrs) = gilrs.as_mut() {
            //drain the event queue so the button state stays fresh
            while gilrs.next_event().is_some() {}
            for (_, gamepad) in gilrs.gamepads() {
                for (button, chip8_key) in GAMEPAD_MAP {
                    if gamepad.is_pressed(button) {
                        chip8.keyboard.key[chip8_key as usize] = true;
                    }
                }
            }
        }

        //run the cpu
        for _ in 0..CYCLES_PER_FRAME {
            chip8.emulate_cycle();
        }
        chip8.tick_timers();

        //sound timer active: beep for the remaining time on another thread
        if chip8.sound_timer > 0 && !beeping.load(Ordering::Relaxed) {
            beeping.store(true, Ordering::Relaxed);
            //the timer runs at 60hz, so each unit is ~16.7ms of sound
            let duration_ms = (chip8.sound_timer as u32 * 1000 / 60).max(50);
            let beeping = Arc::clone(&beeping);
            std::thread::spawn(move || {
                inout::beep(duration_ms);
                beeping.store(false, Ordering::Relaxed);
            });
        }

        //redraw only when a draw or clear opcode ran
        if chip8.display.modify_display {
            for (pixel, color) in chip8.display.pixel.iter().zip(frame_buffer.iter_mut()) {
                *color = if *pixel { 0x00FFFFFF } else { 0x00000000 };
            }
            chip8.display.modify_display = false;
        }

        window
            .update_with_buffer(&frame_buffer, inout::DISPLAY_WIDTH, inout::DISPLAY_HEIGHT)
            .expect("could not update the window");
    }
}

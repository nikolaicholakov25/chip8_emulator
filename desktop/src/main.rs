use chip8_core::*;

use std::fs::File;
use std::io::Read;
use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 20;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 2;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut game_speed: usize = TICKS_PER_FRAME;
    let game_file = &args[1];

    if args.len() > 2 {
        game_speed = args[2].parse().unwrap_or(TICKS_PER_FRAME);
    }


    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Emulator::new();

    let mut rom = File::open(game_file).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} |
                Event::KeyDown{
                    keycode: Some(Keycode::Escape), ..}=> {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                },
                Event::KeyUp{keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                },
                _ => ()
            }
        }

        for _ in 0..(game_speed | TICKS_PER_FRAME) {
            chip8.tick();
        }
        chip8.update_timers();
        draw_screen(&chip8, &mut canvas);
    }
}

fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    // clear canvas
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen = emulator.get_display();

    // set draw color to white
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    // go through screen pixels and add rect for each active pixel
    for (i, pixel_is_on) in screen.iter().enumerate() {
        if *pixel_is_on {
            // get pixel x and y position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // Draw a rectangle at (x,y), scaled up by our SCALE value
            let pixel = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(pixel).unwrap();
        }
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 =>    Some(0x1),
        Keycode::Num2 =>    Some(0x2),
        Keycode::Num3 =>    Some(0x3),
        Keycode::Num4 =>    Some(0xC),
        Keycode::Q =>       Some(0x4),
        Keycode::W =>       Some(0x5),
        Keycode::E =>       Some(0x6),
        Keycode::R =>       Some(0xD),
        Keycode::A =>       Some(0x7),
        Keycode::S =>       Some(0x8),
        Keycode::D =>       Some(0x9),
        Keycode::F =>       Some(0xE),
        Keycode::Z =>       Some(0xA),
        Keycode::X =>       Some(0x0),
        Keycode::C =>       Some(0xB),
        Keycode::V =>       Some(0xF),
        _ =>                None,
    }
}

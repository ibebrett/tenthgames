extern crate sdl2;

use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;

struct Letter {
    x: u32,
    y: u32,
    x2: u32,
    y2: u32,
}

fn load_letters(path: String) -> HashMap<char, Letter> {
    let mut letters: HashMap<char, Letter> = HashMap::new();

    for line in fs::read_to_string(path).unwrap().split("\n") {
        let s: Vec<&str> = line.trim().split(" ").collect();
        if s.len() == 5 {
            letters.insert(
                String::from(s[0]).chars().nth(0).unwrap(),
                Letter {
                    x: s[1].parse::<u32>().unwrap(),
                    y: s[2].parse::<u32>().unwrap(),
                    x2: s[3].parse::<u32>().unwrap(),
                    y2: s[4].parse::<u32>().unwrap(),
                },
            );
        }
    }
    letters
}

fn draw_text(
    text: String,
    letters: &HashMap<char, Letter>,
    font: &Texture,
    canvas: &mut Canvas<sdl2::video::Window>,
) -> Result<(), String> {
    let mut x = 0;
    let mut y = 0;

    // TODO: Get rid of uppercase when we do the lowercase letters.
    for c in text.to_ascii_uppercase().chars() {
        if c == ' ' {
            x += 8;
            continue;
        }
        match letters.get(&c) {
            Some(letter) => {
                let w = letter.x2 - letter.x;
                let h = letter.y2 - letter.y;
                canvas.copy(
                    &font,
                    Rect::new(letter.x as i32, letter.y as i32, w, h),
                    Rect::new(x * 10, y * 10, w * 10, h * 10),
                )?;
                x += w as i32 + 1;
            }
            None => {
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Text!", 1200, 1200)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let font = texture_creator.load_texture("crossfont.png").unwrap();
    let letters = load_letters("crossfont.txt".to_string());

    let mut events = sdl_context.event_pump()?;

    'mainloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {}
                _ => {}
            }
        }

        canvas.set_draw_color(pixels::Color::RGB(255, 255, 0));
        canvas.clear();

        draw_text("Hello Sally".to_string(), &letters, &font, &mut canvas)?;

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

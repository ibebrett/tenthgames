extern crate sdl2;

use std::env;
use std::path::Path;
use std::time::{Duration, SystemTime};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::pixels::Color;

static SCREEN_WIDTH : u32 = 160;
static SCREEN_HEIGHT : u32 = 144;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn run(font_path: &Path) -> Result<(), String> {

    let color_dark = Color::RGBA(15, 56, 15, 255);
    let color_dim = Color::RGBA(48, 98, 48, 255);
    let color_normal = Color::RGBA(139, 172, 15, 255);
    let color_light = Color::RGBA(155, 188, 15, 255);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let window = video_subsys.window("SDL2_TTF Example", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(color_dark);
    canvas.clear();

    // Load a font
    let font = ttf_context.load_font(font_path, 24)?;
    let subtitle = ttf_context.load_font(font_path, 18)?;
    let hint = ttf_context.load_font(font_path, 12)?;

    //Title Text
    let titleSurface = font.render("Nightcrawlers!")
        .blended(color_normal).map_err(|e| e.to_string())?;
    let titleTexture = texture_creator.create_texture_from_surface(&titleSurface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = titleTexture.query();
    let titleTarget = rect!((SCREEN_WIDTH-width)/2, 20, width, height);
    canvas.copy(&titleTexture, None, Some(titleTarget))?;

    //Subtitle Text
    let subtitleSurface = subtitle.render("Kill me now!")
        .blended(color_normal).map_err(|e| e.to_string())?;
    let subtitleTexture = texture_creator.create_texture_from_surface(&subtitleSurface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = subtitleTexture.query();
    let subtitleTarget = rect!((SCREEN_WIDTH-width)/2, 50, width, height);
    canvas.copy(&subtitleTexture, None, Some(subtitleTarget))?;

    //Hint Text
    let hintSurface = hint.render("Press any key to start...")
        .blended(color_normal).map_err(|e| e.to_string())?;
    let hintTexture = texture_creator.create_texture_from_surface(&hintSurface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = hintTexture.query();
    let hintTarget = rect!((SCREEN_WIDTH-width)/2, 120, width, height);
    canvas.copy(&hintTexture, None, Some(hintTarget))?;


    canvas.present();

    let mut last_sec = SystemTime::now();
    let one_sec = Duration::from_secs(1);
    let two_sec = Duration::from_secs(2);
    let mut drawHint = false;
    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} |
                Event::Quit {..} => break 'mainloop,
                _ => {}
            }
        }

        if (last_sec.elapsed().unwrap() > one_sec * 2 && drawHint) {
            last_sec = SystemTime::now();
            drawHint = false;
            
            println!("drawing!");
            canvas.set_draw_color(color_dark);
            canvas.clear();
            canvas.copy(&titleTexture, None, Some(titleTarget))?;
            canvas.copy(&subtitleTexture, None, Some(subtitleTarget))?;
            canvas.present();
        } else if (last_sec.elapsed().unwrap() > one_sec && !drawHint) {
            drawHint = true;

            println!("drawing!");
            canvas.set_draw_color(color_dark);
            canvas.clear();
            canvas.copy(&titleTexture, None, Some(titleTarget))?;
            canvas.copy(&subtitleTexture, None, Some(subtitleTarget))?;
            canvas.copy(&hintTexture, None, Some(hintTarget))?;
            canvas.present();
        }  

    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();

    println!("linked sdl2_ttf: {}", sdl2::ttf::get_linked_version());

    if args.len() < 2 {
        println!("Usage: ./demo font.[ttf|ttc|fon]")
    } else {
        let path: &Path = Path::new(&args[1]);
        run(path)?;
    }

    Ok(())
}
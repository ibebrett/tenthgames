extern crate sdl2;

use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use serde::{Deserialize, Serialize};
//use serde_json::Result;

#[derive(Debug)]
struct Tile {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

#[derive(Debug)]
struct Animation {
    w: u32,
    h: u32,
    tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize)]
struct MapTile {
    x: i32,
    y: i32,
    tile: String,
}

#[derive(Serialize, Deserialize)]
struct MapAnimation {
    x: i32,
    y: i32,
    animation: String,
    frame: usize,
}

#[derive(Serialize, Deserialize)]
struct Map {
    tiles: Vec<MapTile>,
    animations: Vec<MapAnimation>,
}

fn main() -> Result<(), String> {
    let mut tiles: HashMap<String, Tile> = HashMap::new();
    let mut anims: HashMap<String, Animation> = HashMap::new();

    let mut map: Map = serde_json::from_str(&fs::read_to_string("map.json").unwrap()).unwrap();

    for line in fs::read_to_string("tiles_list.txt").unwrap().split("\n") {
        let s: Vec<&str> = line.trim().split(" ").collect();
        if s.len() == 5 {
            println!("parsing tile {}", line);
            tiles.insert(
                String::from(s[0]),
                Tile {
                    x: s[1].parse::<i32>().unwrap(),
                    y: s[2].parse::<i32>().unwrap(),
                    w: s[3].parse::<u32>().unwrap(),
                    h: s[4].parse::<u32>().unwrap(),
                },
            );
        } else if s.len() == 6 {
            println!("parsing anim {}", line);
            let count = s[5].parse::<i32>().unwrap();
            let mut anim = Animation {
                w: s[3].parse::<u32>().unwrap(),
                h: s[4].parse::<u32>().unwrap(),
                tiles: Vec::new(),
            };
            let x = s[1].parse::<i32>().unwrap();
            let y = s[2].parse::<i32>().unwrap();

            for i in 0..count {
                anim.tiles.push(Tile {
                    x: x + (anim.w as i32) * i,
                    y: y,
                    w: anim.w,
                    h: anim.h,
                });
            }
            anims.insert(String::from(s[0]), anim);
        }
    }

    //for (title, a) in &anims {
    //    println!("{} {:?}", title, a);
    //}

    for (title, t) in &tiles {
        println!("{} {:?}", title, t);
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Tiles", 1200, 768)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("tiles.png")?;

    let mut anim_index = 0;
    let anim_keys: Vec<&String> = anims.keys().collect();

    let mut tile_index = 0;
    let tile_keys: Vec<&String> = tiles.keys().collect();

    let mut anim = &anims[anim_keys[anim_index]];
    let mut frame = 0;

    let mut frame_switch = Duration::from_millis(100);

    let mut t = Instant::now();
    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Up {
                        anim_index = (anim_index + 1) % anim_keys.len();
                        anim = &anims[anim_keys[anim_index]];
                        frame = 0;
                    } else if keycode == Keycode::Down {
                        anim_index = (anim_index + anim_keys.len() - 1) % anim_keys.len();
                        anim = &anims[anim_keys[anim_index]];
                        frame = 0;
                    }
                }
                _ => {}
            }
        }

        let nt = Instant::now();
        let dt = nt.duration_since(t);

        if frame_switch.checked_sub(dt) == None {
            frame_switch = Duration::from_millis(100);
            frame = (frame + 1) % anim.tiles.len();
            for map_anim in &mut map.animations {
                let anim = &anims[&map_anim.animation];
                map_anim.frame = (map_anim.frame + 1) % anim.tiles.len();
            }
        } else {
            frame_switch -= dt;
        }

        t = nt;

        canvas.clear();

        for map_tile in &map.tiles {
            let tile = &tiles[&map_tile.tile];
            canvas.copy(
                &texture,
                Rect::new(tile.x, tile.y, tile.w, tile.h),
                Rect::new(map_tile.x * 10, map_tile.y * 10, tile.w * 10, tile.h * 10),
            )?;
        }

        for map_anim in &map.animations {
            let anim = &anims[&map_anim.animation];
            let frame_tile = &anim.tiles[map_anim.frame];
            canvas.copy(
                &texture,
                Rect::new(frame_tile.x, frame_tile.y, anim.w, anim.h),
                Rect::new(map_anim.x * 10, map_anim.y * 10, anim.w * 10, anim.h * 10),
            )?;
        }

        canvas.copy(
            &texture,
            Rect::new(anim.tiles[frame].x, anim.tiles[frame].y, anim.w, anim.h),
            Rect::new(0, 0, anim.w * 10, anim.h * 10),
        )?;
        canvas.present();
    }

    Ok(())
}

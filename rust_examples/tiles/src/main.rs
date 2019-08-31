extern crate sdl2;

use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use serde::{Deserialize, Serialize};

use cgmath::InnerSpace;
use cgmath::Vector2;
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

enum Dir {
    Left,
    Right,
}

impl Dir {
    fn flip(&self) -> bool {
        match self {
            Dir::Left => true,
            Dir::Right => false,
        }
    }
}

struct Player {
    pos: Vector2<f32>,
    walking: bool,
    frame: usize,
    dir: Dir,
}

impl Player {
    fn new() -> Player {
        Player {
            pos: Vector2::new(0.0, 0.0),
            walking: false,
            frame: 0,
            dir: Dir::Right,
        }
    }

    fn update_move(&mut self, dir: Vector2<f32>) {
        if dir.x > 0.0 {
            self.dir = Dir::Right;
        }
        if dir.x < 0.0 {
            self.dir = Dir::Left;
        }

        // Set if we are moving.
        let walking_before = self.walking;
        self.walking = dir.x != 0.0 || dir.y != 0.0;

        if self.walking != walking_before {
            self.frame = 0;
        }
    }
}

struct PlayerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl PlayerInput {
    fn new() -> PlayerInput {
        return PlayerInput {
            up: false,
            down: false,
            left: false,
            right: false,
        };
    }

    fn to_dir(&self) -> Vector2<f32> {
        let mut x = 0.0;
        let mut y = 0.0;
        if self.up {
            y = -1.0;
        }
        if self.down {
            y = 1.0;
        }
        if self.left {
            x = -1.0;
        }
        if self.right {
            x = 1.0;
        }

        if x == 0.0 && y == 0.0 {
            return Vector2::new(x, y);
        }
        let v = Vector2::new(x, y);

        return v * (1.0 / v.dot(v).sqrt());
    }
}

fn parse_tiles() -> (HashMap<String, Tile>, HashMap<String, Animation>) {
    let mut tiles: HashMap<String, Tile> = HashMap::new();
    let mut anims: HashMap<String, Animation> = HashMap::new();

    for line in fs::read_to_string("tiles_list.txt").unwrap().split("\n") {
        let s: Vec<&str> = line.trim().split(" ").collect();
        if s.len() == 5 {
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

    return (tiles, anims);
}

fn main() -> Result<(), String> {
    let (tiles, anims) = parse_tiles();

    let mut map: Map = serde_json::from_str(&fs::read_to_string("map.json").unwrap()).unwrap();

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

    let mut frame = 0;

    let mut frame_switch = 0;

    let mut player = Player::new();
    let player_walk_anim = &anims["elf_m_run_anim"];
    let player_idle_anim = &anims["elf_m_idle_anim"];

    let mut events = sdl_context.event_pump()?;

    'mainloop: loop {
        let mut pi = PlayerInput::new();
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
        let keys: Vec<Keycode> = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        if keys.contains(&Keycode::W) {
            pi.up = true;
        }
        if keys.contains(&Keycode::S) {
            pi.down = true;
        }
        if keys.contains(&Keycode::A) {
            pi.left = true;
        }
        if keys.contains(&Keycode::D) {
            pi.right = true;
        }

        // Move our guy.
        let mov = pi.to_dir();

        player.pos += mov * 13.0;
        player.update_move(mov);

        let anim = if player.walking {
            player_walk_anim
        } else {
            player_idle_anim
        };

        if frame_switch >= 3 {
            player.frame = (player.frame + 1) % anim.tiles.len();
            frame = (frame + 1) % anim.tiles.len();
            for map_anim in &mut map.animations {
                let anim = &anims[&map_anim.animation];
                map_anim.frame = (map_anim.frame + 1) % anim.tiles.len();
            }
            frame_switch = 0;
        } else {
            frame_switch += 1;
        }

        canvas.clear();

        // Draw Map
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

        // Draw "player"
        canvas.copy_ex(
            &texture,
            Rect::new(
                anim.tiles[player.frame].x,
                anim.tiles[player.frame].y,
                anim.w,
                anim.h,
            ),
            Rect::new(
                player.pos.x as i32,
                player.pos.y as i32,
                anim.w * 10,
                anim.h * 10,
            ),
            0.0,
            None,
            player.dir.flip(),
            false,
        )?;
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

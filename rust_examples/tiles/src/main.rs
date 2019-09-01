extern crate rand;
extern crate sdl2;

use itertools::Itertools;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use rand::prelude::*;

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
    width: u32,
    height: u32,
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

struct Character<'a> {
    pos: Vector2<f32>,
    walking: bool,
    frame: usize,
    dir: Dir,
    idle_anim: &'a Animation,
    walk_anim: &'a Animation,
    frame_rate: usize,
    frame_counter: usize,
    player: bool,
    speed: f32,
}

impl<'a> Character<'a> {
    fn new(
        idle_anim: &'a Animation,
        walk_anim: &'a Animation,
        player: bool,
        speed: f32,
    ) -> Character<'a> {
        Character {
            pos: Vector2::new(0.0, 0.0),
            walking: false,
            frame: 0,
            dir: Dir::Right,
            idle_anim: idle_anim,
            walk_anim: walk_anim,
            frame_rate: 3,
            frame_counter: 0,
            player: player,
            speed: speed,
        }
    }

    fn update(&mut self, dir: Vector2<f32>) {
        // Called once per frame, with direction.
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

        if self.frame_counter >= self.frame_rate {
            self.frame = (self.frame + 1) % self.anim().tiles.len();
            self.frame_counter = 0;
        } else {
            self.frame_counter += 1;
        }
    }

    fn anim(&self) -> &'a Animation {
        if self.walking {
            self.walk_anim
        } else {
            self.idle_anim
        }
    }

    fn width_height(&self) -> (u32, u32) {
        let anim = self.anim();
        (anim.w, anim.h)
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
        return normalize(Vector2::new(x, y));
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

fn normalize(v: Vector2<f32>) -> Vector2<f32> {
    if v.x == 0.0 && v.y == 0.0 {
        return v;
    }
    return v * (1.0 / v.dot(v).sqrt());
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

    //let mut player = Character::new(&anims["elf_m_idle_anim"], &anims["elf_m_run_anim"]);
    //let mut imp = Character::new(&anims["elf_m_idle_anim"], &anims["elf_m_run_anim"]);

    let mut characters = HashMap::new(); //<String, Character>
    characters.insert(
        "player",
        Character::new(
            &anims["elf_m_idle_anim"],
            &anims["elf_m_run_anim"],
            true,
            13.0,
        ),
    );
    characters.insert(
        "imp",
        Character::new(
            &anims["goblin_idle_anim"],
            &anims["goblin_run_anim"],
            false,
            10.0,
        ),
    );
    characters.insert(
        "skelet",
        Character::new(
            &anims["skelet_idle_anim"],
            &anims["skelet_run_anim"],
            false,
            3.0,
        ),
    );
    characters.insert(
        "zombie",
        Character::new(
            &anims["zombie_idle_anim"],
            &anims["zombie_run_anim"],
            false,
            8.0,
        ),
    );

    let mut rng = thread_rng();

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
        let player_pos = characters["player"].pos;

        for (_, character) in &mut characters {
            let mov = if character.player {
                pi.to_dir()
            } else {
                let dir = player_pos - character.pos;
                if dir.dot(dir) < 5.0 {
                    Vector2::new(0.0, 0.0)
                } else {
                    normalize(dir) * 0.5
                }
            };
            character.pos += mov * character.speed;
            character.update(mov);
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
        // We want to sort by y index of their bottom.
        let characters_in_order = characters
            .iter()
            .map(|(k, c)| (k, c.pos.y + (c.width_height().1 as f32)))
            .sorted_by(|(_, y1), (_, y2)| y1.partial_cmp(y2).unwrap_or(Ordering::Equal));

        for (k, _) in characters_in_order {
            let character = &characters[k];
            canvas.copy_ex(
                &texture,
                Rect::new(
                    character.anim().tiles[character.frame].x,
                    character.anim().tiles[character.frame].y,
                    character.anim().w,
                    character.anim().h,
                ),
                Rect::new(
                    character.pos.x as i32,
                    character.pos.y as i32,
                    character.anim().w * 10,
                    character.anim().h * 10,
                ),
                0.0,
                None,
                character.dir.flip(),
                false,
            )?;
        }
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

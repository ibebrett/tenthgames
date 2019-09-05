extern crate rand;
extern crate sdl2;

use itertools::Itertools;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use rand::prelude::*;

use sdl2::mixer::{AUDIO_S16LSB, DEFAULT_CHANNELS};

use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Texture;

use serde::{Deserialize, Serialize};

use cgmath::InnerSpace;
use cgmath::Vector2;
//use serde_json::Result;
//

#[derive(Debug)]
struct Tile {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    texture: String,
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
struct MapForbidden {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

#[derive(Serialize, Deserialize)]
struct Map {
    tiles: Vec<MapTile>,
    animations: Vec<MapAnimation>,
    forbidden: Vec<MapForbidden>,
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
    w: u32,
    h: u32,
    moveable: bool,
}

impl<'a> Character<'a> {
    fn new(
        x: f32,
        y: f32,
        idle_anim: &'a Animation,
        walk_anim: &'a Animation,
        player: bool,
        speed: f32,
    ) -> Character<'a> {
        let w = idle_anim.w;
        let h = idle_anim.h;

        Character {
            pos: Vector2::new(x + (((w * 10) as f32) / 2.0), y + ((h * 10) as f32)),
            walking: false,
            frame: 0,
            dir: Dir::Right,
            idle_anim: idle_anim,
            walk_anim: walk_anim,
            frame_rate: 3,
            frame_counter: 0,
            player: player,
            speed: speed,
            w: idle_anim.w,
            h: idle_anim.h,
            moveable: true,
        }
    }

    fn from_anim(x: f32, y: f32, anim: &'a Animation) -> Character<'a> {
        let w = anim.w;
        let h = anim.h;

        Character {
            pos: Vector2::new(
                x * 10.0 + (((w * 10) as f32) / 2.0),
                y * 10.0 + ((h * 10) as f32),
            ),
            walking: false,
            frame: 0,
            dir: Dir::Right,
            idle_anim: anim,
            walk_anim: anim,
            frame_rate: 3,
            frame_counter: 0,
            player: false,
            speed: 0.0,
            w: anim.w,
            h: anim.h,
            moveable: false,
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

    fn top_left(&self) -> (f32, f32) {
        (
            self.pos.x - (self.w as f32 / 2.0) * 10.0,
            self.pos.y - (self.h as f32) * 10.0,
        )
    }
}

struct PlayerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

struct DebugState {
    draw_forbidden: bool,
}

impl DebugState {
    fn new() -> DebugState {
        return DebugState {
            draw_forbidden: false,
        };
    }
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

struct Camera {
    x: f32,
    y: f32,
    w: u32,
    h: u32,
}

impl Camera {
    fn new(w: u32, h: u32) -> Camera {
        Camera {
            w: w,
            h: h,
            x: 0.0,
            y: 0.0,
        }
    }
    fn update(&mut self, character: &Character) -> () {
        self.x = character.pos.x - (self.w as f32 / 2.0);
        self.y = character.pos.y - (self.h as f32 / 2.0);
    }
}

fn parse_tiles(path: &str, texture: String) -> (HashMap<String, Tile>, HashMap<String, Animation>) {
    let mut tiles: HashMap<String, Tile> = HashMap::new();
    let mut anims: HashMap<String, Animation> = HashMap::new();

    for line in fs::read_to_string(path).unwrap().split("\n") {
        let s: Vec<&str> = line.trim().split(" ").collect();
        if s.len() == 5 {
            tiles.insert(
                String::from(s[0]),
                Tile {
                    x: s[1].parse::<i32>().unwrap(),
                    y: s[2].parse::<i32>().unwrap(),
                    w: s[3].parse::<u32>().unwrap(),
                    h: s[4].parse::<u32>().unwrap(),
                    texture: texture.clone(),
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
                    texture: texture.clone(),
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

fn setup_sound(sdl: &sdl2::Sdl) -> Result<(sdl2::AudioSubsystem, sdl2::mixer::Music), String> {
    let audio = sdl.audio()?;

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;

    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::MP3)?;
    sdl2::mixer::allocate_channels(4);

    let music = sdl2::mixer::Music::from_file("music.mp3")?;
    music.play(-1)?;

    Ok((audio, music))
}

fn main() -> Result<(), String> {
    let (mut tiles, mut anims) = parse_tiles("tiles_list.txt", "generic".to_string());
    let (tiles2, anims2) = parse_tiles("slamslime.txt", "slamslime".to_string());

    tiles.extend(tiles2);
    anims.extend(anims2);

    let mut map: Map = serde_json::from_str(&fs::read_to_string("map.json").unwrap()).unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let (audio, music) = setup_sound(&sdl_context)?;

    let sound_chunk = sdl2::mixer::Chunk::from_file("coin.wav")?;
    let window = video_subsystem
        .window("Tiles", 1200, 1200)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut textures: HashMap<String, Texture> = HashMap::new();

    textures.insert(
        "generic".to_string(),
        texture_creator.load_texture("tiles.png").unwrap(),
    );
    textures.insert(
        "slamslime".to_string(),
        texture_creator.load_texture("slamslime.png").unwrap(),
    );

    let mut frame = 0;
    let mut frame_switch = 0;

    let mut debug_state = DebugState::new();

    let mut characters: HashMap<String, Character> = HashMap::new(); //<String, Character>

    characters.insert(
        "player".to_string(),
        Character::new(
            0.0,
            100.0,
            &anims["elf_m_idle_anim"],
            &anims["elf_m_run_anim"],
            true,
            13.0,
        ),
    );
    characters.insert(
        "imp".to_string(),
        Character::new(
            40.0,
            400.0,
            &anims["goblin_idle_anim"],
            &anims["goblin_run_anim"],
            false,
            10.0,
        ),
    );
    characters.insert(
        "slamslime".to_string(),
        Character::new(
            100.0,
            600.0,
            &anims["slamslime"],
            &anims["slamslime"],
            false,
            3.0,
        ),
    );
    characters.insert(
        "zombie".to_string(),
        Character::new(
            34.0,
            400.0,
            &anims["zombie_idle_anim"],
            &anims["zombie_run_anim"],
            false,
            8.0,
        ),
    );

    let mut anim_count = 0;
    for map_anim in &map.animations {
        let key = anim_count.to_string();
        characters.insert(
            key,
            Character::from_anim(
                map_anim.x as f32,
                map_anim.y as f32,
                &anims[&map_anim.animation],
            ),
        );
        anim_count += 1;
    }

    let mut rng = thread_rng();

    let mut events = sdl_context.event_pump()?;

    let mut camera = Camera::new(1200, 1200);

    'mainloop: loop {
        let mut pi = PlayerInput::new();
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    debug_state.draw_forbidden = !debug_state.draw_forbidden;
                }
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

        frame += 1;
        let update_map_anim = if frame == 3 {
            frame = 0;
            true
        } else {
            false
        };

        for map_anim in &mut map.animations {
            if update_map_anim {
                map_anim.frame = (map_anim.frame + 1) % anims[&map_anim.animation].tiles.len();
            }
        }

        // Move our guy.
        let player_pos = characters["player"].pos;

        for (_, character) in &mut characters {
            let mov = if character.player {
                pi.to_dir()
            } else {
                let dir = player_pos - character.pos;
                if dir.dot(dir) < 5.0 {
                    match sdl2::mixer::Channel::all().play(&sound_chunk, 1) {
                        Ok(_) => (),
                        Err(_) => ()
                    }
                    Vector2::new(0.0, 0.0)
                } else {
                    normalize(dir) * 0.5
                }
            } * character.speed;

            character.update(mov);

            // Don't allow movement into restricted areas.
            let new_pos = character.pos + mov;

            let mut allowed = true;

            for forbidden in &map.forbidden {
                if new_pos.x > (forbidden.x * 10) as f32
                    && new_pos.x < ((forbidden.x + forbidden.w as i32) * 10) as f32
                    && new_pos.y > (forbidden.y * 10) as f32
                    && new_pos.y < ((forbidden.y + forbidden.h as i32) * 10) as f32
                {
                    allowed = false;
                }
            }

            if allowed {
                character.pos += mov;
            }
        }

        camera.update(&characters["player"]);

        canvas.clear();

        // Draw Map
        for map_tile in &map.tiles {
            let tile = &tiles[&map_tile.tile];
            canvas.copy(
                &textures[&tile.texture],
                Rect::new(tile.x, tile.y, tile.w, tile.h),
                Rect::new(
                    map_tile.x * 10 - camera.x as i32,
                    map_tile.y * 10 - camera.y as i32,
                    tile.w * 10,
                    tile.h * 10,
                ),
            )?;
        }

        // Draw "player"
        // We want to sort by y index of their bottom.
        let characters_in_order = characters
            .iter()
            .map(|(k, c)| (k, c.pos.y))
            .sorted_by(|(_, y1), (_, y2)| y1.partial_cmp(y2).unwrap_or(Ordering::Equal));

        for (k, _) in characters_in_order {
            let character = &characters[k];
            let (x, y) = character.top_left();
            canvas.copy_ex(
                &textures[&character.anim().tiles[character.frame].texture],
                Rect::new(
                    character.anim().tiles[character.frame].x,
                    character.anim().tiles[character.frame].y,
                    character.anim().w,
                    character.anim().h,
                ),
                Rect::new(
                    (x - camera.x) as i32,
                    (y - camera.y) as i32,
                    character.anim().w * 10,
                    character.anim().h * 10,
                ),
                0.0,
                None,
                character.dir.flip(),
                false,
            )?;
            if debug_state.draw_forbidden {
                canvas.set_draw_color(pixels::Color::RGB(255, 255, 255));
                let r = Rect::new(
                    (x - camera.x) as i32,
                    (y - camera.y) as i32,
                    character.anim().w * 10,
                    character.anim().h * 10,
                );
                canvas.draw_rect(r)?;
            }
        }
        /*
        for map_anim in &map.animations {
            let anim = &anims[&map_anim.animation];
            let frame_tile = &anim.tiles[map_anim.frame];
            canvas.copy(
                &texture,
                Rect::new(frame_tile.x, frame_tile.y, anim.w, anim.h),
                Rect::new(map_anim.x * 10, map_anim.y * 10, anim.w * 10, anim.h * 10),
            )?;
        }*/

        // Draw debugging
        if debug_state.draw_forbidden {
            for forbidden in &map.forbidden {
                canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
                let r = sdl2::rect::Rect::new(
                    forbidden.x * 10 - camera.x as i32,
                    forbidden.y * 10 - camera.y as i32,
                    forbidden.w * 10,
                    forbidden.h * 10,
                );
                canvas.draw_rect(r)?;
            }
            for (_, character) in &characters {
                canvas.set_draw_color(pixels::Color::RGB(255, 255, 0));
                let r = sdl2::rect::Rect::new(
                    (character.pos.x - camera.x) as i32,
                    (character.pos.y - camera.y) as i32,
                    2,
                    2,
                );
                canvas.draw_rect(r)?;
            }
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

extern crate rand;
extern crate sdl2;

use std::cmp;
use std::time::Duration;

use rand::prelude::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;

use sdl2::gfx::primitives::DrawRenderer;

const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;

#[derive(Copy, Clone, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

fn random_block(snake: &Vec<Point>) -> Point {
    let mut rng = thread_rng();
    'search: loop {
        let block = Point {
            x: rng.gen_range(0, 32),
            y: rng.gen_range(0, 24),
        };
        if !snake.iter().any(|p| *p == block) {
            return block;
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum SnakeDir {
    Left,
    Right,
    Up,
    Down,
}

fn opp_dir(dir: SnakeDir) -> SnakeDir {
    return match dir {
        SnakeDir::Left => SnakeDir::Right,
        SnakeDir::Right => SnakeDir::Left,
        SnakeDir::Up => SnakeDir::Down,
        SnakeDir::Down => SnakeDir::Up,
    };
}

fn dir_to_delta(dir: SnakeDir) -> (i32, i32) {
    return match dir {
        SnakeDir::Left => (-1, 0),
        SnakeDir::Right => (1, 0),
        SnakeDir::Up => (0, -1),
        SnakeDir::Down => (0, 1),
    };
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("Snaek", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;

    let mut snake: Vec<Point> = vec![Point { x: 3, y: 3 }];
    let mut dir = SnakeDir::Right;
    let mut block = random_block(&snake);
    let mut dead = false;
    let mut frames = 10;

    'main: loop {
        let mut new_dir = dir;
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'main;
                    } else if keycode == Keycode::Left {
                        if SnakeDir::Left != opp_dir(dir) {
                            new_dir = SnakeDir::Left;
                        }
                    } else if keycode == Keycode::Right {
                        if SnakeDir::Right != opp_dir(dir) {
                            new_dir = SnakeDir::Right;
                        }
                    } else if keycode == Keycode::Up {
                        if SnakeDir::Up != opp_dir(dir) {
                            new_dir = SnakeDir::Up;
                        }
                    } else if keycode == Keycode::Down {
                        if SnakeDir::Down != opp_dir(dir) {
                            new_dir = SnakeDir::Down;
                        }
                    }
                }
                _ => {}
            }
        }

        let (dir_x, dir_y) = dir_to_delta(dir);
        dir = new_dir;

        // Do the logic.
        if !dead {
            let new_point = *snake.last().unwrap(); //[snake.len() - 1];
            let nx = snake[0].x + dir_x;
            let ny = snake[0].y + dir_y;

            for i in (1..snake.len()).rev() {
                snake[i].x = snake[i - 1].x;
                snake[i].y = snake[i - 1].y;
                if nx == snake[i].x && ny == snake[i].y {
                    dead = true;
                }
            }

            snake[0].x = nx;
            snake[0].y = ny;

            if snake[0].x < 0 || snake[0].x > 31 || snake[0].y < 0 || snake[0].y > 23 {
                dead = true;
            }

            if snake[0] == block {
                snake.push(new_point);
                block = random_block(&snake);
                frames += 2;
                frames = cmp::min(60, frames);
            }
        }

        // Do the drawing.
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        if dead {
            canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
            canvas.clear();
        } else {
            canvas.set_draw_color(pixels::Color::RGB(255, 0, 0));
            for p in snake.iter() {
                let r = sdl2::rect::Rect::new(p.x * 10, p.y * 10, 10, 10);
                canvas.draw_rect(r)?;
            }

            canvas.set_draw_color(pixels::Color::RGB(0, 255, 0));
            let r = sdl2::rect::Rect::new(block.x * 10, block.y * 10, 10, 10);
            canvas.draw_rect(r)?;
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / frames));
    }

    Ok(())
}

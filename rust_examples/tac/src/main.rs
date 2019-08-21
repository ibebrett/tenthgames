use memmap::MmapOptions;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::SeekFrom;

use std::io::prelude::*;

fn tac_simple(path: &String) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    for line in contents.rsplit("\n") {
        println!("{}", line);
    }

    Ok(())
}

fn tac_memmap(path: &String) -> std::io::Result<()> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let mmap_iter = mmap.iter().rev();

    let mut last_i = mmap.len();
    let mut curr_i = last_i;
    for c in mmap_iter {
        curr_i -= 1;
        if (*c as char) == '\n' || curr_i == 0 {
            println!("{}", String::from_utf8_lossy(&mmap[curr_i..last_i]));
            last_i = curr_i;
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];
    let algo = &args[2];
    match algo.as_ref() {
        "memmap" => tac_memmap(path)?,
        _ => tac_simple(path)?,
    }

    Ok(())
}

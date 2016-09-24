// Utility functions.
// Probably shouldn't be part of the public API.

use std::path;
use std::io;
use std::marker::Sized;

use sdl2::rwops;

use context::Context;


// This is actually very inconvenient 'cause sdl2::rwops
// can be created from bytes, or from a file path, but not
// from a std::io::Read
// Which is what we need to read from streams.
pub fn rwops_from_read<'a, T>(r: &mut T, buffer: &'a mut Vec<u8>) -> Result<rwops::RWops<'a>, String>
    where T: io::Read + Sized {
    // For now, we just rather messily slurp the whole thing into memory,
    // then hand that to from_bytes.
    r.read_to_end(buffer).unwrap();
    rwops::RWops::from_bytes(buffer)
}

pub fn rwops_from_path<'a>(context: &mut Context, path: &path::Path, buffer: &'a mut Vec<u8>) -> rwops::RWops<'a> {
    //let mut fs = &context.filesystem;
    let mut stream = context.filesystem.open(path).unwrap();
    let rw = rwops_from_read(&mut stream, buffer).unwrap();
    rw
}



// Patch submitted to rust-sdl2_mixer, see:
// https://github.com/andelf/rust-sdl2_mixer/pull/58
// Until it's accepted though, we gotta hack it ourselves.
extern crate sdl2_sys as sys;
use self::sys::rwops::SDL_RWops;
use std::os::raw::{c_int, c_void};
use sdl2;
use sdl2_mixer;
extern "C" {
    fn Mix_LoadMUS_RW(src: *mut SDL_RWops, freesrc: c_int) -> *mut Mix_Music;
}
#[allow(non_camel_case_types)]
type Struct__Mix_Music = c_void;
#[allow(non_camel_case_types)]
type Mix_Music = Struct__Mix_Music;
pub fn load_music(rwops: rwops::RWops) -> Result<sdl2_mixer::Music, String> {
    let raw = unsafe { Mix_LoadMUS_RW(rwops.raw(), 0) };
    if raw.is_null() {
        Err(sdl2::get_error())
    } else {
        Ok(sdl2_mixer::Music {
            raw: raw,
            owned: true,
        })
    }
}
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

pub fn rwops_from_path<'a>(context: &Context, path: &path::Path, buffer: &'a mut Vec<u8>) -> rwops::RWops<'a> {
    let fs = &context.filesystem;
    let mut stream = fs.open(path).unwrap();
    let rw = rwops_from_read(&mut stream, buffer).unwrap();
    rw
}


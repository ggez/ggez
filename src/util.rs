//! Utility functions.
//!
//! Generally not things end-users have to worry about.

use std::path;
use sdl2::rwops;

use context::Context;
use GameResult;


pub fn rwops_from_path<'a>(context: &mut Context,
                           path: &path::Path,
                           buffer: &'a mut Vec<u8>)
                           -> GameResult<rwops::RWops<'a>> {
    let mut stream = try!(context.filesystem.open(path));
    let rw = try!(rwops::RWops::from_read(&mut stream, buffer));
    Ok(rw)
}

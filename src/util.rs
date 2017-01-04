//! Utility functions.
//!
//! Generally not things end-users have to worry about.

use std::path;
use sdl2::rwops;
use sdl2::surface;
use sdl2::image::ImageRWops;

use context::Context;
use GameError;
use GameResult;

pub fn rwops_from_path<'a, P: AsRef<path::Path>>(context: &mut Context,
                                                 path: P,
                                                 buffer: &'a mut Vec<u8>)
                                                 -> GameResult<rwops::RWops<'a>> {
    let mut stream = try!(context.filesystem.open(path.as_ref()));
    let rw = try!(rwops::RWops::from_read(&mut stream, buffer));
    Ok(rw)
}


// Here you should just imagine me frothing at the mouth as I
// fight the lifetime checker in circles.
fn clone_surface<'a>(s: surface::Surface<'a>) -> GameResult<surface::Surface<'static>> {
    // let format = pixels::PixelFormatEnum::RGBA8888;
    let format = s.pixel_format();
    // convert() copies the surface anyway, so.
    let res = try!(s.convert(&format));
    Ok(res)
}

/// Loads a given surface.
/// This is here instead of in graphics because it's sorta private-ish
/// (since ggez never exposes a SDL surface directly)
/// but it gets used in context.rs to load and set the window icon.
pub fn load_surface<P: AsRef<path::Path>>(context: &mut Context,
                                          path: P)
                                          -> GameResult<surface::Surface<'static>> {
    let mut buffer: Vec<u8> = Vec::new();
    let rwops = try!(rwops_from_path(context, path.as_ref(), &mut buffer));
    // SDL2_image SNEAKILY adds the load() method to RWops
    // with the ImageRWops trait.
    let surface = try!(rwops.load().map_err(GameError::ResourceLoadError));
    // We *really really* need to clone this surface here because
    // otherwise lifetime interactions between rwops, buffer and surface become
    // intensely painful.
    clone_surface(surface)
}

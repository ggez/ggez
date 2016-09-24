extern crate sdl2;
extern crate sdl2_ttf;

use sdl2::pixels::Color;
use sdl2::rwops;

use std::path::Path;
use std::io::Read;
use std::fs::File;


pub fn make_text() -> sdl2::render::Texture {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2_ttf::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video.window("kerblammo", 800, 600)
                               .position_centered()
                               //.opengl()
                               .build().unwrap();

    let renderer = window.renderer()
        //.accelerated()
        .build().unwrap();

    // Load font
    let path = Path::new("DejaVuSerif.ttf");
    let size = 24;
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(path).unwrap();
    let _ = file.read_to_end(&mut buffer).unwrap();
    let rwops = rwops::RWops::from_bytes(&buffer).unwrap();

    // Explodes when I uncomment this line.
    //let ttf_font = ttf_context.load_font_from_rwops(rwops, size).unwrap();

    // Works fine when I create the font with this instead
    let ttf_font = ttf_context.load_font(path, 24).unwrap();

    println!("Make sure we still have this vec: {:?}", buffer.len());
    let surf = ttf_font.render("Hello world")
        // If I change .solid() to .blended() it works fine.
        .solid(Color::RGB(255,255,255))
        .unwrap();
    // SEGFAULTS HERE!  But only when using solid() to make the surface,
    // not blended()!
    // Does it have anything to do with the font lifetime thing???
    // It shouldn't, 'cause we still have the buffer we read into here!
    let texture = renderer.create_texture_from_surface(surf).unwrap();
    //println!("Make sure we still have this vec: {:?}", buffer);
    texture

}


pub fn main() {
    println!("Loading");
    let t = make_text();
    println!("Got texture");
}

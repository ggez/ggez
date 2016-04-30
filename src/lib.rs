extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event::*;
use sdl2::keyboard::Keycode::*;
use std::thread;
use std::time::Duration;

pub fn ralf() {
    println!("ralf");
    let sdl_context = sdl2::init().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video = sdl_context.video().unwrap();
    
    let window = video.window("Ruffel", 800, 600)
        .position_centered().opengl()
        .build().unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build().unwrap();

    let mut done = false;
    let mut delta = Duration::new(0, 0);
    while !done {
        let start_time = timer.ticks();
        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();
        renderer.present();

        thread::sleep_ms(1000/60);
        for event in event_pump.poll_iter() {
            match event {
                Quit { .. } => done = true,
                KeyDown { keycode, .. } => match keycode {
                    Some(Escape) => done = true,
                    _ => {}
                },
                _ => {}
            }
        }

        let end_time = timer.ticks();
        delta = Duration::from_millis((end_time - start_time) as u64);
        println!("{:?}", delta);
    }
}

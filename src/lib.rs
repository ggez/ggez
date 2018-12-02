//! # What is this?
//! [![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez)
//! [![Build status](https://ci.appveyor.com/api/projects/status/3v9lsq6n9li7kxim/branch/master?svg=true)](https://ci.appveyor.com/project/svenstaro/ggez/branch/master)
//! [![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez)
//! [![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/ggez/blob/master/LICENSE)
//! [![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez)
//! [![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)
//!
//! ggez is a Rust library to create a Good Game Easily.
//!
//! More specifically, ggez is a lightweight game framework for making
//! 2D games with minimum friction.  It aims to implement an API based
//! on (a Rustified version of) the [LÖVE](https://love2d.org/) game
//! framework.  This means it contains basic and portable 2D
//! drawing, sound, resource loading and event handling.
//!
//! ggez is not meant to be everything to everyone, but rather a good
//! base upon which to build.  Thus it takes a fairly
//! batteries-included approach without needing a million additions
//! and plugins for everything imaginable, but also does not dictate
//! higher-level functionality such as physics engine or entity
//! component system.  Instead the goal is to allow you to use
//! whichever libraries you want to provide these functions, or build
//! your own libraries atop ggez.
//!
//! ## Features
//!
//! * Filesystem abstraction that lets you load resources from folders or zip files
//! * Hardware-accelerated 2D rendering built on the `gfx-rs` graphics engine
//! * Loading and playing .ogg, .wav and .flac files via the `rodio` crate
//! * TTF font rendering with `rusttype`, as well as bitmap fonts.
//! * Interface for handling keyboard and mouse events easily through callbacks
//! * Config file for defining engine and game settings
//! * Easy timing and FPS measurement functions.
//! * Math integration with nalgebra
//! * Some more advanced graphics options: shaders, sprite batches and render targets
//!
//! ## Usage
//!
//! ggez is built on the latest stable Rust compiler and distributed on
//! crates.io.  To include it in your project, just add the dependency
//! line to your `Cargo.toml` file:
//!
//! ```text
//! ggez = "0.4"
//! ```
//!
//! However you also need to have the SDL2 libraries installed on your
//! system.  The best way to do this is documented [by the SDL2
//! crate](https://github.com/AngryLawyer/rust-sdl2#user-content-requirements).
//!
//! ggez consists of three main parts: A [`Context`](struct.Context.html) object
//! which contains all the state required to interface with the computer's
//! hardware, an [`EventHandler`](event/trait.EventHandler.html) trait that the
//! user implements to register callbacks for events, and various sub-modules such as
//! [`graphics`](graphics/index.html) and [`audio`](audio/index.html) that provide
//! the functionality to actually get stuff done. 
//!
//! The general pattern is to create a struct holding your game's data which implements
//! the `EventHandler` trait. Create a new `Context` object with default objects from a
//! [`ContextBuilder`](struct.ContextBuilder.html) or [`Conf`](conf/struct.Conf.html) object,
//! and then call [`event::run()`](event/fn.run.html) with the `Context` and an instance of
//! your `EventHandler` to run your game's main loop.
//!
//! ```compile
//! use ggez::{Context, ContextBuilder, GameResult};
//! use ggez::event::{self, EventHandler};
//!
//! fn main() {
//!     // Make a Context.
//!     let ctx = &mut /* ContextBuilder params */
//! #       ContextBuilder::new("doc_template", "ggez")
//! #           .build()
//! #           .unwrap();
//!
//!     // Create an instance of your event handler.
//!     // Usually, you want to provide it with the Context object to use it when setting
//!     // your game up.
//!     let mut my_game = MyGame::new(ctx);
//!
//!     // Run!
//!     match event::run(ctx, &mut my_game) {
//!         Ok(_) => println!("Exited cleanly."),
//!         Err(e) => println!("Error occured: {}", e)
//!     }
//! }
//!
//! struct MyGame {
//!     // Your state here...
//! }
//!
//! impl EventHandler for MyGame {
//!     fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
//!         // Update code here...
//! #       Ok(())
//!     }
//!
//!     fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
//!         // Draw code here...
//! #       Ok(())
//!     }
//! }
//! #
//! # impl MyGame {
//! #   pub fn new(_ctx: &mut Context) -> MyGame {
//! #       MyGame { }
//! #   }
//! # }
//! ```
//!
//! ## Examples
//!
//! See the [`examples/`](https://github.com/ggez/ggez/tree/master/examples) directory
//! in the source.  Most examples show off a single feature of ggez, while 
//! [`astroblasto`](https://github.com/ggez/ggez/blob/master/examples/astroblasto.rs)
//! is a small  but complete Asteroids-like game.
//!
//! To run the examples, just check out the source and execute `cargo run --example`
//! in the root directory:
//!
//! ```text
//! cargo run --example astroblasto
//! ```
//!
//! If this doesn't work, see the
//! [FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) for solutions
//! to common problems.
//!
//! ## Implementation details
//!
//! ggez is built upon SDL2 for windowing and events, `rodio` for sound,
//! and a 2D drawing engine implemented in `gfx` using the OpenGL backend
//! (which currently defaults to use OpenGL 3.2).  It *should* be
//! entirely thread-safe outside of the basic event-handling loop, and
//! portable to Windows, Linux and Mac.
//!
//! The goal is to eventually have ggez be pure Rust, but we're not there
//! yet.
//!
//! ## Help!
//!
//! Sources of information:
//!
//!  * The [FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) has answers to common questions and problems.
//!  * The [API docs](https://docs.rs/ggez/), a lot of design stuff is explained there.
//!  * Check out the [examples](https://github.com/ggez/ggez/tree/master/examples).
//!
//! If you still have problems, feel free to [open an issue](https://github.com/ggez/ggez/issues) or say hi in the `#rust-gamedev` IRC channel on the `irc.mozilla.org` server.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate app_dirs2;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_glyph;
extern crate gfx_window_sdl;
extern crate image;
#[macro_use]
extern crate log;
extern crate lyon;
pub extern crate nalgebra;
extern crate rodio;
extern crate rusttype;
extern crate sdl2;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate smart_default;
extern crate toml;
extern crate zip;

pub mod audio;
pub mod conf;
mod context;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod graphics;
pub mod input;
pub mod mouse;
pub mod timer;
mod vfs;

pub use context::{Context, ContextBuilder};
pub use error::*;

//! # What is this?
//!
//! ggez is a Rust library to create a Good Game Easily.
//!
//! More specifically, ggez is a lightweight game framework for making
//! 2D games with minimum friction.  It aims to implement an API based
//! on (a Rustified version of) the [LÖVE](https://love2d.org/) game
//! framework.  This means it contains basic and portable 2D
//! drawing, sound, resource loading and event handling.
//!
//! For a fuller outline, see the [README.md](https://github.com/ggez/ggez/)
//!
//! ## Usage
//!
//! ggez consists of three main parts: A `Context` object which
//! contains all the state required to interface with the computer's
//! hardware, an `EventHandler` trait that the user implements to
//! register callbacks for events, and various sub-modules such as
//! `graphics` and `audio` that provide the functionality to actually
//! get stuff done.  The general pattern is to create a struct holding
//! your game's data which implements the `EventHandler` trait.
//! Create a new `Context` object with default settings from a `ContextBuilder`
//! or `Conf` object, and then call `event::run()` with
//! the `Context` and an instance of your `EventHandler` to run your game's
//! main loop.  See the [examples](https://github.com/ggez/ggez/blob/master/examples/hello_world.rs)
//! for a number of full demos.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]

#[macro_use]
extern crate bitflags;
extern crate app_dirs2;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;
#[macro_use]
extern crate log;
extern crate lyon;
pub extern crate mint;
pub extern crate nalgebra;
extern crate rodio;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate smart_default;
extern crate gilrs;
extern crate toml;
extern crate winit;
extern crate zip;

pub mod audio;
pub mod conf;
mod context;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod graphics;
pub mod input;
pub mod timer;
mod vfs;

pub use context::{Context, ContextBuilder};
pub use error::*;

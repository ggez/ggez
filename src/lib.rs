//! # ggez
//!
//! ggez is a framework for creating 2D video games.  Its API is
//! heavily based on the Love2D game engine, though inevitably
//! with some differences due to the fact that Rust is not Lua.
//!
//! ggez consists of three main parts: A `Context` object which contains
//! all the state requierd to interface with the computer's hardware,
//! a `GameState` trait that the user implements to register callbacks
//! for events, and various sub-modules such as `graphics` and `audio`
//! that provide the functionality to actually get stuff done.
//!
//! Note that ggez isn't intended to be everything to everyone; it is
//! deliberately opinionated and tries to provide a good basic framework
//! for getting stuff done rather than having a million options for everything
//! imaginable.
//!
//! # Implementation notes
//!
//! ggez is a fairly thin wrapper around SDL2 and a few other
//! libraries, which does influence some of the API and impose some
//! restrictions.  For example, thread safety.
//!
//! ## Thread safety
//!
//! SDL2 is generally speaking NOT thread-safe.  It uses lots of globals
//! internally, and just isn't designed with thread safety in mind.  This
//! isn't generally a huge restriction in C code, but in Rust the practical
//! result is that none of the types derived from SDL2 are `Send` or `Sync`,
//! such as the `ggez::graphics::Image` type.

extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate rand;
extern crate rustc_serialize;
extern crate rusttype;
extern crate toml;
extern crate zip;


pub mod audio;
pub mod conf;
mod context;
mod error;
pub mod event;
pub mod filesystem;
pub mod game;
pub mod graphics;
pub mod timer;
mod util;

pub use game::Game;
pub use context::Context;
pub use error::*;

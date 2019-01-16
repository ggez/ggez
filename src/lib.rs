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
//! ## Basic Project Template
//!
//! ```rust,no_run
//! use ggez::{Context, ContextBuilder, GameResult};
//! use ggez::event::{self, EventHandler};
//!
//! fn main() {
//!     // Make a Context and an EventLoop.
//!     let (mut ctx, mut event_loop) = /* ContextBuilder params */
//! #       ContextBuilder::new("doc_template", "ggez")
//! #           .build()
//! #           .unwrap();
//!
//!     // Create an instance of your event handler.
//!     // Usually, you should provide it with the Context object to
//!     // use when setting your game up.
//!     let mut my_game = MyGame::new(&mut ctx);
//!
//!     // Run!
//!     match event::run(&mut ctx, &mut event_loop, &mut my_game) {
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

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
// This is not as strong a constraint as `#![forbid(unsafe_code)]` but is good enough.
// It means the only place we use unsafe is then in the modules noted as allowing it.
#![deny(unsafe_code)]
#![warn(bare_trait_objects)]
#![warn(missing_copy_implementations)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate gfx;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate smart_default;

pub extern crate mint;
pub extern crate nalgebra;

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

#[cfg(test)]
pub mod tests;

pub use crate::context::{quit, Context, ContextBuilder};
pub use crate::error::*;

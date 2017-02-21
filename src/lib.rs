//! # What is this?
//! [![Build Status](https://!travis-ci.org/ggez/ggez.svg?branch=master)](https://!travis-ci.org/ggez/ggez) [![Docs Status](https://!docs.rs/ggez/badge.svg)](https://!docs.rs/ggez) [![license](http://!img.shields.io/badge/license-MIT-blue.svg)](https://!github.com/svenstaro/ggez/blob/master/LICENSE) [![Crates.io](https://!img.shields.io/crates/v/ggez.svg)](https://!crates.io/crates/ggez) [![Crates.io](https://!img.shields.io/crates/d/ggez.svg)](https://!crates.io/crates/ggez)
//!
//! ggez is a Rust library to create a Good Game Easily.
//!
//! More specifically, ggez is a lightweight game framework for making 2D
//! games with minimum friction.  It aims to implement an API quite
//! similar to (a Rustified version of) the Love2D game engine.  This
//! means it will contain basic and portable 2D drawing, sound, resource
//! loading and event handling.
//!
//! ggez is not meant to be everything to everyone, but rather a good base
//! upon which to build.  Thus it takes a fairly batteries-included
//! approach without needing a million options and plugins for everything
//! imaginable, but also does not dictate higher-level functionality such
//! as physics engine or ECS.  Instead the goal is to allow you to use
//! whichever libraries you want to provide these functions, or build your
//! own libraries atop ggez such as the
//! [ggez-goodies](https://!github.com/ggez/ggez-goodies) crate.
//!
//! ## Features
//!
//! * Filesystem abstraction that lets you load resources from folders or zip files
//! * Hardware-accelerated rendering of bitmaps
//! * Playing and loading sounds through SDL2_mixer
//! * TTF font rendering with rusttype, as well as bitmap fonts.
//! * Interface for handling keyboard and mouse events easily through callbacks
//! * Config file for defining engine and game settings
//! * Easy timing and FPS measurement functions.
//!
//! ## Usage
//!
//! ggez is built on the latest stable Rust compiler and distributed on
//! crates.io.  To include it in your project, just add the dependency
//! line to your `Cargo.toml` file:
//!
//! ```text
//! ggez = "0.2.0"
//! ```
//!
//! However you also need to have the SDL2, SDL2_mixer and SDL2_image
//! libraries installed on your system.  The best way to do this is
//! documented [by the SDL2
//! crate](https://!github.com/AngryLawyer/rust-sdl2#user-content-requirements).
//!
//! ggez consists of three main parts: A `Context` object which contains
//! all the state requierd to interface with the computer's hardware, a
//! `GameState` trait that the user implements to register callbacks for
//! events, and various sub-modules such as `graphics` and `audio` that
//! provide the functionality to actually get stuff done.
//!
//! ## Examples
//!
//! See the `examples/` directory in the source.  `hello_world` is exactly
//! what it says.  `imageview` is a simple program that shows off a number
//! of features such as sound and drawing.  `astroblasto` is a small
//! Asteroids-like game.
//!
//! To run the examples, you have to copy or symlink the `resources`
//! directory to a place the running game can find it.  Cargo does not
//! have an easy way of doing this itself at the moment, so the procedure
//! is (on Linux):
//!
//! ```text
//! cargo build --example astroblasto
//! cp -R resources target/debug/
//! cargo run --example astroblasto
//! ```
//!
//! Either way, if it can't find the resources it will give you an error
//! along the lines of `ResourceNotFound("'resources' directory not
//! found!  Should be in "/home/foo/src/ggez/target/debug/resources")`.
//! Just copy or symlink the `resources/` directory to where the error says it's
//! looking.
//!
//! ## Implementation details
//!
//! ggez is a fairly thin wrapper around SDL2 and a few other
//! libraries, which does influence some of the API and impose some
//! restrictions.  For example, thread safety.
//!
//! SDL2 is generally speaking NOT thread-safe.  It uses lots of
//! globals internally, and just isn't designed with thread safety in
//! mind.  This isn't generally a huge restriction in C code, but in
//! Rust the practical result is that none of the types derived from
//! SDL2 are `Send` or `Sync`, such as the `ggez::graphics::Image`
//! type.  It's inconvenient and we want to work around it eventually,
//! but for now, them's the breaks.




extern crate sdl2;
extern crate app_dirs;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate image;
extern crate rand;
extern crate rodio;
extern crate rustc_serialize;
extern crate rusttype;
extern crate toml;
extern crate zip;


pub mod audio;
pub mod conf;
mod context;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod graphics;
pub mod timer;

pub use context::Context;
pub use error::*;

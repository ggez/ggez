//! # What is this?
//! [![Build Status](https://!travis-ci.org/ggez/ggez.svg?branch=master)](https://!travis-ci.org/ggez/ggez) [![Docs Status](https://!docs.rs/ggez/badge.svg)](https://!docs.rs/ggez) [![license](http://!img.shields.io/badge/license-MIT-blue.svg)](https://!github.com/svenstaro/ggez/blob/master/LICENSE) [![Crates.io](https://!img.shields.io/crates/v/ggez.svg)](https://!crates.io/crates/ggez) [![Crates.io](https://!img.shields.io/crates/d/ggez.svg)](https://!crates.io/crates/ggez)
//! 
//! ggez is a Rust library to create a Good Game Easily.
//! 
//! More specifically, ggez is a lightweight game framework for making 2D
//! games with minimum friction.  It aims to implement an API based on (a
//! Rustified version of) the Love2D game framework.  This means it will
//! contain basic and portable 2D drawing, sound, resource loading and
//! event handling.
//! 
//! ggez is not meant to be everything to everyone, but rather a good base
//! upon which to build.  Thus it takes a fairly batteries-included
//! approach without needing a million additions and plugins for everything
//! imaginable, but also does not dictate higher-level functionality such
//! as physics engine or ECS.  Instead the goal is to allow you to use
//! whichever libraries you want to provide these functions, or build your
//! own libraries atop ggez.
//! 
//! ## Features
//! 
//! * Filesystem abstraction that lets you load resources from folders or zip files
//! * Hardware-accelerated rendering engine built on the `gfx-rs` graphics engine
//! * Playing and loading .ogg, .wav and .flac files via the `rodio` crate
//! * TTF font rendering with `rusttype`, as well as bitmap fonts.
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
//! ggez = "0.3.0"
//! ```
//! 
//! However you also need to have the SDL2 libraries installed on your
//! system.  The best way to do this is documented [by the SDL2
//! crate](https://github.com/AngryLawyer/rust-sdl2#user-content-requirements).
//! 
//! ggez consists of three main parts: A `Context` object which contains
//! all the state required to interface with the computer's hardware, an
//! `EventHandler` trait that the user implements to register callbacks for
//! events, and various sub-modules such as `graphics` and `audio` that
//! provide the functionality to actually get stuff done.
//! 
//! 
//! ## Examples
//! 
//! See the `examples/` directory in the source.  `hello_world` is exactly
//! what it says.  `imageview` is a simple program that shows off a number
//! of features such as sound and drawing.  `astroblasto` is a small
//! but complete Asteroids-like game.
//! 
//! To run the examples, you have to tell your program where to find the
//! `resources` directory included in the git repository.  The easy way is
//! to enable `cargo-resource-root` flag to tell ggez to look for a
//! `resources` directory next to your `Cargo.toml`, or copy or symlink
//! the `resources` directory to a place the running game can find it
//! (such as next to the game executable).
//! 
//! ```text
//! cargo build --example astroblasto
//! cargo run --example astroblasto --features=cargo-resource-root
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
//! ggez is built upon SDL2 for windowing and events, `rodio` for sound,
//! and a 2D drawing engine implemented in `gfx` using the OpenGL backend
//! (which currently defaults to use OpenGL 3.2).  It *should* be
//! entirely thread-safe outside of the basic event-handling loop, and
//! portable to Windows, Linux and Mac.
//! 
//! The goal is to eventually have ggez be pure Rust, but we're not there
//! yet.  The main blocker appears to be cross-platform
//! joystick/controller input; once that exists we can drop SDL2 for
//! `glutin`.





extern crate sdl2;
extern crate app_dirs;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_sdl;
extern crate image;
extern crate rand;
extern crate rodio;
#[macro_use]
extern crate serde_derive;
extern crate rusttype;
extern crate toml;
extern crate zip;
extern crate lyon;

pub mod audio;
pub mod conf;
mod context;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod graphics;
pub mod timer;
mod vfs;

pub use context::Context;
pub use error::*;

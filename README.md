# What is this?
[![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez)
[![Build status](https://ci.appveyor.com/api/projects/status/3v9lsq6n9li7kxim/branch/master?svg=true)](https://ci.appveyor.com/project/svenstaro/ggez/branch/master)
[![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/ggez/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez)
[![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)

ggez is a Rust library to create a Good Game Easily.

More specifically, ggez is a lightweight game framework for making 2D
games with minimum friction.  It aims to implement an API based on (a
Rustified version of) the Love2D game framework.  This means it will
contain basic and portable 2D drawing, sound, resource loading and
event handling.

ggez is not meant to be everything to everyone, but rather a good base
upon which to build.  Thus it takes a fairly batteries-included
approach without needing a million additions and plugins for everything
imaginable, but also does not dictate higher-level functionality such
as physics engine or ECS.  Instead the goal is to allow you to use
whichever libraries you want to provide these functions, or build your
own libraries atop ggez.

## Features

* Filesystem abstraction that lets you load resources from folders or zip files
* Hardware-accelerated rendering engine built on the `gfx-rs` graphics engine
* Playing and loading .ogg, .wav and .flac files via the `rodio` crate
* TTF font rendering with `rusttype`, as well as bitmap fonts.
* Interface for handling keyboard and mouse events easily through callbacks
* Config file for defining engine and game settings
* Easy timing and FPS measurement functions.

## Usage

ggez is built on the latest stable Rust compiler and distributed on
crates.io.  To include it in your project, just add the dependency
line to your `Cargo.toml` file:

```text
ggez = "0.2.2"
```

However you also need to have the SDL2 libraries installed on your
system.  The best way to do this is documented [by the SDL2
crate](https://github.com/AngryLawyer/rust-sdl2#user-content-requirements).

ggez consists of three main parts: A `Context` object which contains
all the state required to interface with the computer's hardware, an
`EventHandler` trait that the user implements to register callbacks for
events, and various sub-modules such as `graphics` and `audio` that
provide the functionality to actually get stuff done.


## Examples

See the `examples/` directory in the source.  `hello_world` is exactly
what it says.  `imageview` is a simple program that shows off a number
of features such as sound and drawing.  `astroblasto` is a small
but complete Asteroids-like game.

To run the examples, you have to tell your program where to find the
`resources` directory included in the git repository.  The easy way is
to enable `cargo-resource-root` flag to tell ggez to look for a
`resources` directory next to your `Cargo.toml`, or copy or symlink
the `resources` directory to a place the running game can find it
(such as next to the game executable).

```text
cargo build --example astroblasto
cargo run --example astroblasto --features=cargo-resource-root
```

Either way, if it can't find the resources it will give you an error
along the lines of `ResourceNotFound("'resources' directory not
found!  Should be in "/home/foo/src/ggez/target/debug/resources")`.
Just copy or symlink the `resources/` directory to where the error says it's
looking.

## Implementation details

ggez is built upon SDL2 for windowing and events, `rodio` for sound,
and a 2D drawing engine implemented in `gfx` using the OpenGL backend
(which currently defaults to use OpenGL 3.2).  It *should* be
entirely thread-safe outside of the basic event-handling loop, and
portable to Windows, Linux and Mac.

The goal is to eventually have ggez be pure Rust, but we're not there
yet.  The main blocker appears to be cross-platform
joystick/controller input; once that exists we can drop SDL2 for
`glutin`.


### 0.3.0

* Remove unused example assets
* Go through `timer` and clean things up a little; it should provide nice functions to do everything you want as accurately as you want using only `Duration`s.  Remove the rest, though still have convenience functions to convert to seconds or such.
* The usual cleanup: go through looking for TODO's, unwrap's, run clippy over it.

Changelog:

* Almost everything is now pure rust; the only C dependency is libsdl2.
* Graphics:
 * Entirely new rendering engine using `gfx-rs` backed by OpenGL 3.2
 * New (if limited) 2D drawing primitives using `lyon`
 * Font rendering still uses `rusttype` but it's still cool
 * New option to enable/disable vsync
* Other stuff
 * New sound system using `rodio`, supporting pure Rust loading of WAV, Vorbis and FLAC files
 * Configuration system now uses `serde` rather than `rustc_serialize`
 * Refactored event loop handling somewhat to make it less magical and more composable.
 * New filesystem indirection code using `app_dirs`.  There's also a new `cargo-resource-root` feature flag that will make the file loader look for a `resources` directory next to your `Cargo.toml`; worse than useless for release, but great for development.

So this has been a pretty revolutionary change; I think the only part that hasn't been significantly rewritten is the timing utility functions.  The drawing API is much more powerful and flexible, as well as more rusty, and there's been a million tiny ergonomic improvements.  I'm also willing to call most of the current API more or less stable; I expect to make additions, but not many breaking changes.

As always, thanks to all who contributed: svenstaro, onelson, vickenty, and whoever I don't remember!  And thanks to everyone who makes the libraries we rely on, especially `rust-sdl2`, `rodio` and all its dependencies, `gfx-rs` and all its dependencies, `serde`, `image`, as well as all the tiny but vital cogs like `app_dirs` and `zip`.  None of this would be possible without you guys.

Of course, there's plans for 0.4: I mainly want to improve the graphics functionality with sprite batches, better 2D drawing, and exposing the gfx-rs innards a little to allow the adventurous to write their own rendering pipelines.  It should be a *much* less massive change, and hopefully won't take four months to write.

Annoyances with rust I've found: Paths need to be decoupled from OsString (and strings in general) because that's the only sane way to operate.  Features aren't featureful enough -- can't be defined for examples, can't be set for debug mode and not for release mode, etc.

For now though... if you'll excuse me, I've got some games to make.


## Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

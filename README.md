[![ggez logo](docs/ggez-logo-maroon-full.svg)](http://ggez.rs/)
# What is this?
[![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez)
[![Build status](https://ci.appveyor.com/api/projects/status/3v9lsq6n9li7kxim/branch/master?svg=true)](https://ci.appveyor.com/project/svenstaro/ggez/branch/master)
[![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/ggez/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez)
[![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)

ggez is a Rust library to create a Good Game Easily.

More specifically, ggez is a lightweight cross-platform game framework
for making 2D games with minimum friction.  It aims to implement an
API based on (a Rustified version of) the [LÖVE](https://love2d.org/)
game framework.  This means it contains basic and portable 2D
drawing, sound, resource loading and event handling.

ggez is not meant to be everything to everyone, but rather a good
base upon which to build.  Thus it takes a fairly
batteries-included approach without needing a million additions
and plugins for everything imaginable, but also does not dictate
higher-level functionality such as physics engine or entity
component system.  Instead the goal is to allow you to use
whichever libraries you want to provide these functions, or build
your own libraries atop ggez.

## Features

* Filesystem abstraction that lets you load resources from folders or zip files
* Hardware-accelerated 2D rendering built on the `gfx-rs` graphics engine
* Loading and playing .ogg, .wav and .flac files via the `rodio` crate
* TTF font rendering with `rusttype`, as well as bitmap fonts.
* Interface for handling keyboard and mouse events easily through callbacks
* Config file for defining engine and game settings
* Easy timing and FPS measurement functions.
* Math integration with nalgebra
* Some more advanced graphics options: shaders, sprite batches and render targets


## Supported platforms

 * Fully supported: Windows, Linux, macOS
 * Work in progress: Web/WASM/Emscripten
 * Not officially supported yet (but maybe you can help!): Android, iOS

For details, see [docs/BuildingForEveryPlatform.md](docs/BuildingForEveryPlatform.md)

## Who's using ggez?

Check out the [projects list!](docs/Projects.md)

## Usage

ggez is requires rustc >= 1.28 and distributed on
crates.io.  To include it in your project, just add the dependency
line to your `Cargo.toml` file:

```text
ggez = "0.5"
```
See the [API docs](https://docs.rs/ggez/) for full documentation, or the [examples](/examples) directory for a number of commented examples of varying complexity.  Most examples show off
a single feature of ggez, while `astroblasto` and `snake` are a small but complete games.

To run the examples, just check out the source and execute `cargo run --example`
in the root directory:

```text
git clone https://github.com/ggez/ggez.git
cd ggez
cargo run --example astroblasto
```

If this doesn't work, see the
[FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) for solutions
to common problems.

### Basic Project Template

```rust
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};

fn main() {
    // Make a Context.
    let ctx = &mut /* ContextBuilder params */

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(ctx);

    // Run!
    match event::run(ctx, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

struct MyGame {
    // Your state here...
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Draw code here...
    }
}
```

## Implementation details

ggez is built upon `winit` for windowing and events, `rodio` for sound,
and a 2D drawing engine implemented in `gfx` using the OpenGL backend
(which currently defaults to use OpenGL 3.2).  It *should* be
entirely thread-safe outside of the basic event-handling loop, and
portable to Windows, Linux and Mac.

ggez is Pure Rust(tm).

## Help!

Sources of information:

 * The [FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) has answers to common questions and problems.
 * The [API docs](https://docs.rs/ggez/), a lot of design stuff is explained there.
 * Check out the [examples](https://github.com/ggez/ggez/tree/master/examples).

 If you still have problems or questions, feel free to ask!  Easiest ways are:

 * open an issue on [the Github issue tracker](https://github.com/ggez/ggez/issues)
 * say hi in the `#rust-gamedev` IRC channel on the `irc.mozilla.org` server
 * or in the [unofficial Rust Discord server](http://bit.ly/rust-community)

# ggez

[![ggez logo](docs/ggez-logo-maroon-full.svg)](http://ggez.rs/)

## What is this?

![Build status](https://github.com/ggez/ggez/workflows/CI/badge.svg)
[![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ggez/ggez/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez)
[![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)
![Maintenance](https://img.shields.io/badge/maintenance-actively--maintained-green.svg)

ggez is a Rust library to create a Good Game Easily.

More specifically, ggez is a lightweight cross-platform game framework
for making 2D games with minimum friction.  It aims to implement an
API based on (a Rustified version of) the [LÖVE](https://love2d.org/)
game framework.  This means it contains basic and portable 2D
drawing, sound, resource loading and event handling, but finer details
and performance characteristics may be different than LÖVE.

ggez is not meant to be everything to everyone, but rather a good
base upon which to build.  Thus it takes a fairly
batteries-included approach without needing a million additions
and plugins for everything imaginable, but also does not dictate
higher-level functionality such as physics engine or entity
component system.  Instead the goal is to allow you to use
whichever libraries you want to provide these functions, or build
your own libraries atop ggez.

### Features

* Filesystem abstraction that lets you load resources from folders or zip files
* Hardware-accelerated 2D rendering built on the `gfx-rs` graphics engine
* Loading and playing .ogg, .wav and .flac files via the `rodio` crate
* TTF font rendering with `rusttype` and `glyph_brush`.
* Interface for handling keyboard and mouse events easily through callbacks
* Config file for defining engine and game settings
* Easy timing and FPS measurement functions.
* Math library integration with `mint`.
* Some more advanced graphics options: shaders, sprite batches and render targets

### Non-Features (i.e. things to add from elsewhere if needed)

* [Physics](https://arewegameyet.rs/ecosystem/physics/)
* Animation (check out [keyframe](https://github.com/HannesMann/keyframe); [it works pretty well with ggez](https://psteinhaus.github.io/ggez/web-examples/) ([source](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/ggez/web-examples)))
* [GUI](https://arewegameyet.rs/ecosystem/ui/)
* [Assets manager](https://github.com/a1phyr/assets_manager)
* [AI](https://arewegameyet.rs/ecosystem/ai/)
* [ECS](https://arewegameyet.rs/ecosystem/ecs/)
* [Networking](https://arewegameyet.rs/ecosystem/networking/)

### Supported platforms

 * Fully supported: Windows, Linux
 * Not officially supported but might work anyway: Mac

For details, see [docs/BuildingForEveryPlatform.md](docs/BuildingForEveryPlatform.md)

If you want to run ggez on Android, iOS or the web using WebAssembly take a look at [good-web-game](https://github.com/ggez/good-web-game).

### Who's using ggez?

Check out the [projects list!](docs/Projects.md)

### Usage

ggez requires rustc >= 1.42 and is distributed on
crates.io. To include it in your project, just add the dependency
line to your `Cargo.toml` file:

```
ggez = "0.7"
```

ggez consists of three main parts: A `Context` object which
contains all the state required to interface with the computer's
hardware, an `EventHandler` trait that the user implements to
register callbacks for events, and various sub-modules such as
`graphics` and `audio` that provide the functionality to actually
get stuff done.  The general pattern is to create a struct holding
your game's data which implements the `EventHandler` trait.
Create a new `Context` object with default objects from a `ContextBuilder`
or `Conf` object, and then call `event::run()` with
the `Context` and an instance of your `EventHandler` to run your game's
main loop.

See the [API docs](https://docs.rs/ggez/) for full documentation, or the [examples](/examples) directory for a number of commented examples of varying complexity.  Most examples show off
a single feature of ggez, while `astroblasto` and `snake` are small but complete games.

### Getting started

For a quick tutorial on ggez, see the [Hello ggez](https://github.com/ggez/ggez/blob/master/docs/guides/HelloGgez.md) guide in the `docs/` directory.

### Examples

See the `examples/` directory in the source.  Most examples show off
a single feature of ggez, while `astroblasto` is a small  but
complete Asteroids-like game.

To run the examples, just check out the source and execute `cargo run --example`
in the root directory:

```
git clone https://github.com/ggez/ggez.git
cd ggez
cargo run --example 05_astroblasto
```

If this doesn't work, see the
[FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) for solutions
to common problems.

#### Basic Project Template

```rust,no_run
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color};
use ggez::event::{self, EventHandler};

fn main() {
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    // Your state here...
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        MyGame {
            // ...
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);
        // Draw code here...
        graphics::present(ctx)
    }
}
```

### Implementation details

ggez is built upon `winit` for windowing and events, `rodio` for
sound, and a 2D drawing engine implemented in `gfx` using the OpenGL
backend (which currently defaults to use OpenGL 3.2).  It is entirely
thread-safe (though platform constraints mean the event-handling loop
and drawing must be done in the main thread), and portable to Windows
and Linux.

ggez is pure Rust™.

### Help!

Sources of information:

 * The [FAQ](https://github.com/ggez/ggez/blob/master/docs/FAQ.md) has answers to common questions and problems.
 * The [API docs](https://docs.rs/ggez/), a lot of design stuff is explained there.
 * Check out the [examples](https://github.com/ggez/ggez/tree/master/examples).

 If you still have problems or questions, feel free to ask!  Easiest ways are:

 * Open an issue on [the Github issue tracker](https://github.com/ggez/ggez/issues)
 * Say hi on the [unofficial Rust Discord server](http://bit.ly/rust-community), the [Rust Gamedev server](https://discord.gg/yNtPTb2) or the [good-web-game Discord server](https://discord.gg/jum3Fjek2A)

License: MIT

# What is this?
[![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez)
[![Build status](https://ci.appveyor.com/api/projects/status/3v9lsq6n9li7kxim/branch/master?svg=true)](https://ci.appveyor.com/project/svenstaro/ggez/branch/master)
[![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/ggez/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez)
[![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)

ggez is a Rust library to create a Good Game Easily.

More specifically, ggez is a lightweight game framework for making 2D
games with minimum friction.  It aims to implement an API quite
similar to (a Rustified version of) the Love2D game engine.  This
means it will contain basic and portable 2D drawing, sound, resource
loading and event handling.

ggez is not meant to be everything to everyone, but rather a good base
upon which to build.  Thus it takes a fairly batteries-included
approach without needing a million options and plugins for everything
imaginable, but also does not dictate higher-level functionality such
as physics engine or ECS.  Instead the goal is to allow you to use
whichever libraries you want to provide these functions, or build your
own libraries atop ggez such as the
[ggez-goodies](https://github.com/ggez/ggez-goodies) crate.

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
Asteroids-like game.

To run the examples, you have to copy or symlink the `resources`
directory to a place the running game can find it.  Cargo does not
have an easy way of doing this itself at the moment, so the procedure
is (on Linux):

```text
cargo build --example astroblasto
cp -R resources target/debug/examples
cargo run --example astroblasto
```

Either way, if it can't find the resources it will give you an error
along the lines of `ResourceNotFound("'resources' directory not
found!  Should be in "/home/foo/src/ggez/target/debug/resources")`.
Just copy or symlink the `resources/` directory to where the error says it's
looking.

## Implementation details

ggez is built upon SDL2 for windowing and events, `rodio` for sound,
and a 2D drawing engine implemented in `gfx` using the OpenGL backend
(which is currently hardwired to use OpenGL 3.2).  It *should* be
entirely thread-safe outside of the basic event-handling loop, and
portable to Windows, Linux and Mac.

The goal is to eventually have ggez be pure Rust, but we're not there
yet.  The main blocker appears to be cross-platform
joystick/controller input; once that exists we can drop SDL2 for
`glutin`.


### 0.3.0

* Make screen transforms a bit more transparent
* Make non-power-of-2 textures work (issue #59)
* Make text drawing work
* Document **everything**
* Make it always possible to load resources from raw data instead of files. (which might make testing easier) (issue #38)
* Start integrating ncollide?
* Remove unused example assets
* Go through `timer` and clean things up a little; it should provide nice functions to do everything you want as accurately as you want using only `Duration`s.  Deprecate the rest.
* The usual cleanup: go through looking for TODO's, unwrap's, run clippy over it.
* Make `app_dirs` work (issue #56)


## Future work

* Sprite batching
* Exposing GFX
* Make it work with non-GL backends
* Make subsystems modular, so we don't *have* to initialize sound if we don't need to and it's not a hard error if we can't use it.  See https://www.idolagames.com/piston-sdl-window-with-sound/ perhaps.
* Possibly related, see if it's possible to make the event::run() function optional; provide tools with which to roll your own game loop.
* Interpolation for the mainloop timing stuff?  Or at least be able to support the user doing it.
* Need to add more tests

## Useful goodies

* ggez-goodies for things that are useful but not fundamental and generally don't depend on each other
* specs for entity-component system (alternatives: ecs or recs crates)
* nalgebra or cgmath for vector math.
* physics/collision???  ncollide and nphysics; there's ports/wrappers of box2d and chipmunk physics engines but they're young.

## Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

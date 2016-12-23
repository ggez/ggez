# ggez
[![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez) [![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez) [![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/ggez/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez) [![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)

A Rust library to create a Good Game Easily.

More specifically, ggez is a lightweight game framework for making 2D games.  It is built on SDL2, and aims to
implement an API quite similar to (a Rustified version of) the Love2D game engine.  This means it will contain
basic and portable 2D drawing, sound, resource loading and event handling.

It's not meant to be everything to everyone, but rather a good base upon which to build.  However, eventually
there is also a [ggez-goodies](https://github.com/ggez/ggez-goodies) crate that aims to implement higher-level 
tools atop this, such as a resource cache, basic GUI/debugger, scene manager, and more sophisticated drawing 
systems such as sprites, layer, tiled maps, etc.


## Features

* Filesystem abstraction that lets you load resources from folders or zip files
* Hardware-accelerated rendering of bitmaps
* Playing and loading sounds through SDL2_mixer
* TTF font rendering with rusttype, as well as bitmap fonts.
* Interface for handling keyboard and mouse events easily through callbacks
* Config file for defining engine and game settings
* Easy timing and FPS measurement functions.

## Usage

ggez is built on the latest stable Rust compiler and distributed on crates.io.  To include it in your project, just
add the dependency line to your `Cargo.toml` file:

```
ggez = "0.2.0"
```

However you also need to have the SDL2, SDL2_mixer and SDL2_image libraries installed on your system.  The best way to do this is documented
[by the SDL2 crate](https://github.com/AngryLawyer/rust-sdl2#user-content-requirements).


## Examples

See the examples.  `imageview` is a simple hello-world-y program that shows off a number of things, badly.
`astroblasto` is a small Asteroids-like game.

To run the examples, you have to copy or symlink the `resources` directory to a
place the running game can find it.  Cargo does not have an easy way
of doing this itself at the moment, so the procedure is (on Linux):

```
cargo build --example astroblasto
cp -R resources target/debug/
cargo run --example astroblasto
```

Either way, if it can't find the resources it will give you an error
along the lines of `ResourceNotFound("'resources' directory not
found!  Should be in "/home/foo/src/ggez/target/debug/resources")`.
Just copy or symlink the `resources` directory to where the error says it's
looking.

## Extant things to do

### 0.2.x

Enhancements that don't actually change the API or compatibility

* Crate-level docs (so you get an intro instead of just a list of modules on the root page)
* Document SDL's thread constraints!  It's mentioned in Context struct docs but maybe should be in other places.  The
Game trait would be a good place to do it perhaps?  Or just a mention in the docs for each resource type?
* Submit an update to the zip crate to make it possible to check whether a directory exists.

### 0.3.0

API-breaking or altering changes

* Get rid of the Option in the event callback function signatures... why does SDL2 even have that there anyway?
* Better timing for update and draw in the mainloop would be nice so you don't have to delay manually
* Replace `try!()` with `?` everywhere (so we stop working on older versions of rustc)
* Make it always possible to load resources from raw data instead of files. (which might make testing easier)
* Clean up and consistentify GameError a bit, rename it to GgezError perhaps?  I think there might be an unused case
or two in there.
* Start integrating ncollide?
* Remove unused example assets


## Future work

* Make subsystems modular, so we don't *have* to initialize sound if we don't need to and it's not a hard error if we can't use it.  See https://www.idolagames.com/piston-sdl-window-with-sound/ perhaps.
* Possibly related, see if it's possible to make the GameState trait optional; provide tools with which to roll your own game loop.
* Interpolation for the mainloop timing stuff?  Or at least be able to support the user doing it.
* Include vector math?
* Play with GFX more
* Play with audio more: the ears crate looks rather good, rust-portaudio might be an option???, perhaps alto.  Love2D
apparently directly wraps OpenAL.  Or tomaka has a library, `rodio`.  Or rsoundio?
* Need to add more tests, somehow

It *would* be nice to have a full OpenGL-y backend like Love2D does, with things like shaders, render targets,
etc.  `gfx` might be the best option there, maaaaaaybe.  Right now the API is mostly limited to Love2D 0.7 or so.  Using OpenAL (through the `ears` crate perhaps?)
for sound would get us positional audio too.  

## Useful goodies

* ggez-goodies for things that are useful but not fundamental and generally don't depend on each other
* specs for entity-component system (alternatives: ecs or recs crates)
* cgmath or vecmath for math operations?  nalgebra does great too.
* physics/collision???  ncollide and nphysics; there's ports/wrappers of box2d and chipmunk physics engines but they're young.

## Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

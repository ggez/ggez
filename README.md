# ggez

A Rust library to create a Good Game Easily.

It is built on SDL2, and aims to implement an API quite similar to (a simplified version of) the Love2D game
engine.  This means it will contain basic and portable drawing and sound, resource loading and event handling.

It's not meant to be everything to everyone, but rather a good base upon which to build.  However, eventually
there should be a ggez-goodies crate that implements higher-level systems atop this, such as a resource cache,
basic GUI/debugger, scene manager, and more sophisticated drawing tools such as sprites, layered and tiled maps,
etc.


# Features

* Filesystem abstraction that lets you load resources from folders or zip files
* Hardware-accelerated rendering of bitmaps
* Playing and loading sounds through SDL2_mixer
* TTF font rendering through SDL2_ttf, as well as (eventually) bitmap fonts.
* Interface for handling keyboard and mouse events easily through callbacks
* Config file for defining engine and game settings

# Examples

See example/imageview.rs

# Status

* Make timing module; frame timing is kinda primitive and narsty, make it better.
* Need to make the example's resource paths work properly with `cargo run --example`
* Need more documentation
* Need to implement bitmap fonts
* Need to add more tests, somehow
* Need to figure out exiting cleanly.  THIS IS SOLVED, but blocked by a bug in rust-sdl!  Issue #530.

# Things to add atop it

* Resource loader/cache
* Scene stack
* GUI
* particle system (or put that in with it like LOVE?)
* Input indirection layer and input state tracking
* Sprites with ordering, animation, atlasing, tile mapping, etc.

# Useful goodies

* specs for entity-component system (alternatives: ecs or recs crates)
* cgmath or vecmath for math operations?
* physics/collision???

# Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

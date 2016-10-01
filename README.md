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
* Easy timing and time-tracking functions.

# Examples

See `example/imageview.rs`

To run, you have to copy (or symlink) the `resources` directory to a
place the running game can find it.  Cargo does not have an easy way
of doing this itself at the moment, so the procedure is (on Linux):

```
cargo build --example imageview
cp -R resources target/debug/
cargo run --example imageview
```

Either way, if it can't find the resources it will give you an error
along the lines of `ResourceNotFound("'resources' directory not
found!  Should be in "/home/foo/src/ggez/target/debug/resources")`.
Just copy the `resources` directory to where the error says it's
looking.

# Status

* Need to add more tests, somehow
* Need to figure out exiting cleanly.  THIS IS SOLVED, but blocked by a bug in rust-sdl!  Issue #530.

# Future work

It *would* be nice to have a full OpenGL-y backend like Love2D does, with things like shaders, render targets,
etc.  `gfx` might be the best option there, maaaaaaybe.  Right now the API is mostly limited to Love2D 0.7 or so.  Using OpenAL (through the `ears` crate perhaps?)
for sound would get us positional audio too.  

# Useful goodies

* specs for entity-component system (alternatives: ecs or recs crates)
* cgmath or vecmath for math operations?
* physics/collision???

# Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

# ggez
[![Build Status](https://travis-ci.org/ggez/ggez.svg?branch=master)](https://travis-ci.org/ggez/ggez) [![Docs Status](https://docs.rs/ggez/badge.svg)](https://docs.rs/ggez) [![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/ggez/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/ggez.svg)](https://crates.io/crates/ggez) [![Crates.io](https://img.shields.io/crates/d/ggez.svg)](https://crates.io/crates/ggez)

A Rust library to create a Good Game Easily.

It is built on SDL2, and aims to implement an API quite similar to (a simplified version of) the Love2D game
engine.  This means it will contain basic and portable drawing and sound, resource loading and event handling.

It's not meant to be everything to everyone, but rather a good base upon which to build.  However, eventually
there should be a ggez-goodies crate that implements higher-level systems atop this, such as a resource cache,
basic GUI/debugger, scene manager, and more sophisticated drawing tools such as sprites, layered and tiled maps,
etc.


## Features

* Filesystem abstraction that lets you load resources from folders or (eventually) zip files
* Hardware-accelerated rendering of bitmaps
* Playing and loading sounds through SDL2_mixer
* TTF font rendering through SDL2_ttf, as well as (eventually) bitmap fonts.
* Interface for handling keyboard and mouse events
* Config file for defining engine and game settings

## Examples

See example/imageview.rs

## Status

* Still implementing sound
* Need to make the example's resource paths work properly
* Need to unify Context type better
* Need to implement the ability to replace the game state with the same context
* Need to figure out pipeline for creating contexts and gamestates
* Need to implement draw_rect and stuff
* Need more documentation
* Need to implement bitmap fonts and zip file loading
* Need to add more tests, somehow

## Useful goodies

* cgmath for math operations
* specs for entity-component system
* cgmath or vecmath for math operations?
* physics/collision???

## Credits

* http://opengameart.org/content/flappy-dragon-sprite-sheets
* http://opengameart.org/content/cozy-endless-game-background

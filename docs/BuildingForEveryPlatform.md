# Introduction

Greetings, one and all.  Today we shall explore how to build and
deploy a `ggez` game for every possible platform.  For platforms like
Linux and Mac it's pretty darn simple.  For ones like Android it gets
harder and you have to jump through hoops.  The purpose of this is to
document the hoops and give you a cookbook on the best jumping methods
and trajectories.  We will progress generally from the easiest to
hardest jumps.

## Project setup

We will use the `hello_world` example project from ggez for all these
examples.  To do the initial setup, assuming you have cargo installed:

```sh
cargo init --bin hello_world
cd hello_world
```

Now copy-paste the contents of
<https://raw.githubusercontent.com/ggez/ggez/master/examples/hello_world.rs>
into `hello_world/src/main.rs`, or just wget it:

```sh
wget https://raw.githubusercontent.com/ggez/ggez/master/examples/hello_world.rs
mv hello_world.rs src/main.rs
```

You'll need a font to print "Hello world!" with, so we need to fetch one and
put it in a subdirectory called `resources` in your project root:

```sh
mkdir resources
cd resources
wget https://raw.githubusercontent.com/ggez/ggez/master/resources/DejaVuSerif.ttf
```

Then edit your `Cargo.toml` with your favorite super duper editor and under `[dependencies]` add:

```
ggez = "0.4"
```

Now run `cargo run` and it should build
and run!  ...maybe.  It depends on what platform you're on and what
libraries you have installed.  To make super-duper sure you have all
the bits and pieces in the right places to make this always work, read
on!

# Linux

## Debian

Very easy, just install the required dev packages:

```sh
apt install libasound2-dev libsdl2-dev pkg-config
```

Then you should be able to build with `cargo run`

## Redhat

Same libraries as Debian, slightly different names.  On CentOS 7 at least you can install them with:

```sh
yum install alsa-lib-devel SDL2-devel
```

## Distributing

As documented in more depth [here](https://aimlesslygoingforward.com/blog/2014/01/19/bundling-shared-libraries-on-linux/).

The *right* way is to get your users to install SDL2 themselves, and make sure it's the same one you built your game against.  This is often not practical unless you're delivering your game as source.

Another other option is to take the `libSDL2.so` library and distribute it with your game, then include a startup script that sets the `LD_LIBRARY_PATH` to include wherever you put it.

So if our game directory structure is:

```
my_game
lib/libSDL2.so
```

our launcher script would go in the same directory and look like:

```sh
#!/bin/sh
GAMEDIR=$(dirname "$0")
LD_LIBRARY_PATH=$GAMEDIR/lib $GAMEDIR/my_game
```

Or, and this is a new one to me, you can use the `rpath` linker option to include a relative `LD_LIBRARY_PATH`-ish path to search in the executable itself, by setting the `LD_RUN_PATH` environment variable when you build your game:

```
env LD_RUN_PATH='$ORIGIN/lib' cargo build
```

Then, when the executable is run, it will look in `lib/` relative to the executable location for shared objects along with wherever the system says it should look.

Note that distributing your own libSDL2.so will still fail if the user has a significantly different of `glibc` than you do.  If anyone comes up with a good way to build everything statically with musl it would be interesting to be able to do that.

# Mac

Install SDL2 with the [brew](https://brew.sh/) package manager like so:

```sh
brew install sdl2
```

which should build and install SDL2, header files and any dependencies.

## Distributing

???

# Windows

All you need to install is the SDL2 libraries but it's a pain in the butt.  The instructions here are from the [sdl2](https://github.com/AngryLawyer/rust-sdl2#user-content-windows-msvc) crate for building with MSVC, which is what I've found to be simplest:

1. Download MSVC development libraries from http://www.libsdl.org/ (SDL2-devel-2.0.x-VC.zip).
2. Unpack SDL2-devel-2.0.x-VC.zip to a folder of your choosing (You can delete it afterwards).
3. Copy all lib files from
    > SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\
    
    to the Rust library folder.  For Rustup users (most common), this folder will be in
    > C:\\Users\\{Your Username}\\.rustup\\toolchains\\{current toolchain}\\lib\\rustlib\\{current toolchain}\\lib
    
    or, if not using Rustup, to (for Rust 1.6 and above)
    > C:\\Program Files\\Rust\\**lib**\\rustlib\\x86_64-pc-windows-msvc\\lib

    or to (for Rust versions 1.5 and below)
    > C:\\Program Files\\Rust\\**bin**\\rustlib\\x86_64-pc-windows-msvc\\lib

    or to your library folder of choice, and ensure you have a system environment variable of
    > LIB = C:\your\rust\library\folder

  Where current toolchain is likely `stable-x86_64-pc-windows-msvc`.

4. Copy SDL2.dll from
    > SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\

    into your cargo project, right next to your Cargo.toml.

However once this is done you should be good to go.

## Distributing

Just copy SDL2.dll to the same directory that your compiled exe is in and distribute it along with.

# Android

Not officially supported yet. ;_; See https://github.com/ggez/ggez/issues/70

# iOS

Not officially supported yet. ;_; See https://github.com/ggez/ggez/issues/70

# Web/wasm/emscripten

Not officially supported yet. ;_; See https://github.com/ggez/ggez/issues/71

However, there are some WIP notes for getting things running on
emscripten:

Known blockers for wasm: threads (rodio), gfx-rs (OpenGL backend
should work but is touchy to set up, we don't yet have a good way to
specify WebGL), probably app_dirs/resources, SDL2

For the sake of (my) simplicity, I am going to assume this is being done on Linux or something like it for now.

First fetch and install the emscripten SDK, which is a compiler and set of libraries for turning LLVM code into asm.js or wasm.  You should get the latest version from [the emscripten website](http://kripken.github.io/emscripten-site/docs/getting_started/downloads.html).  It appears to be distributed as a portable package since it contains a lot of stuff you don't want mucking up your system with anyway; it's basically a self-contained cross-compiler, and you usually want to make sure those live in their own little world.

Follow the emscripten install instructions.  Unpack it somewhere convenient, install dependencies, set environment variables, etc.

Now install the Rust emscripten toolchain:

```sh
rustup target install wasm32-unknown-emscripten
```

(We're using wasm here 'cause it's awesome.)

Build the thing:

```
embuilder.py build sdl2
export EMMAKEN_CFLAGS="-s USE_SDL=2"
cargo build --target=wasm32-unknown-emscripten
```


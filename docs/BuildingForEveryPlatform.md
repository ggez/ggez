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

Then edit your `Cargo.toml` with your favorite super duper editor and under `[dependencies]` add:

```
ggez = "0.3"
```

Now run `cargo run` and it should build and run!  ...maybe.  It
depends on what platform you're on and what libraries you have
installed.  To make super-duper sure you have all the bits and pieces
in the right places to make this always work, read on!

# Linux

# Mac

# Windows

# Android

# iOS

# Web

# Table of Contents

* **[Errors](#errors)**
  * [I get `ResourceNotFound("/myfile", ...)` even though it's in the resource dir!](#errors_resource)
  * [Why do I get `WindowError("Could not create GL context")` when I try to run my game?](#errors_window)
* **[Graphics and GUIs](#gfx)**
  * [Can I do 3D stuff?](#gfx_3d)
  * [How do I make a GUI?](#gfx_gui)
  * [Resolution independence (or "Why do things not end up where I want them to be?")](#gfx_resolution)
  * [Relative and absolute offsets with `DrawParam::offset`](#offsets)
* **[Input](#input)**
  * [Returned mouse coordinates are wrong!](#mouse_coords)
* **[Libraries](#libraries)**
  * [Can I use `specs`, `legion` or another entity-component system?](#library_ecs)
  * [What is mint and how do I use `Into<mint::Point2<f32>>` and other `Into<mint::T>` types?](#library_mint)
* **[Performance](#performance)**
  * [Image/sound loading and font rendering is slow!](#perf_slow1)
  * [That's lame, can't I just compile my game in debug mode but ggez with optimizations on?](#perf_debug)
  * [Drawing a few hundred images or shapes is slow!](#perf_drawing)
* **[Platform-specific](#platforms)**
  * [How do I build on platform X?](#platform_build)
  * [Is Mac/iOS supported?](#platform_mac)
* **[Contributing to ggez](#contributing)**
  * [If I write X, will you include it in ggez?](#contribute_inclusion)
* **[Miscellaneous](#misc)**
  * [How do I load my `conf.toml` file?](#misc_conf)
  * [I get a console window when I launch my executable on Windows](#misc_win_console)

---

# Errors

<a name="errors_resource">

## I get `ResourceNotFound("/myfile", ...)` even though it's in the resource dir!

Okay, first, look at [the docs](https://docs.rs/ggez/) for the
`filesystem` module.  That should say exactly where it should look for
files.  Note that paths **must start with leading slash**; relative
paths are not allowed!  Also note that it expects the `resources/`
directory to be beside the *executable*, not in the cargo root dir,
which is annoying because cargo tends to put the executable in
`target/debug/whatever`.  You can add the cargo root dir to the
filesystem lookup path by pulling it from the environment variable, see
the examples for how.  Sorry, there's no especially good way of doing it
automatically; we've tried.

If that doesn't help, call `Context::print_resource_stats()`.  That
should print out all the files it can find, and where it is finding
them.

If you want to add a non-standard location to the resources lookup
path, you can use `Filesystem::mount()` or
`ContextBuilder::add_resource_path()`; see the examples for examples.

<a name="errors_window">

## Why do I get `WindowError("Could not create GL context")` when I try to run my game?

Basically this means "the graphics driver couldn't give ggez the
graphics settings it's asking for".  This usually means "the graphics
driver doesn't support OpenGL 3.2", which is the default version of
OpenGL ggez asks for.  Other possible causes include things like "It
doesn't support the level of multisampling you are asking for".

Also check the list of
[known driver bugs](https://github.com/ggez/ggez/issues?utf8=%E2%9C%93&q=is%3Aissue+label%3A%22driver+bug%22)
on the issue tracker.

Great, how do you troubleshoot it?

On Linux, the program `glxinfo` will give you more info than you ever
wanted about exactly what features your graphics driver supports, and if
you dig enough through it you can find what version of OpenGL it has
available.

To request different graphics settings you can change the appropriate
entries in the `Conf` object before creating your `Context`.  If you
request older versions of OpenGL you will also have to provide shaders
written in the appropriate version of GLSL (which is a bit of a WIP)
and there're no promises that things like `SpriteBatch` and `Canvas`
will work.

<a name="gfx">

# Graphics and GUIs

<a name="gfx_3d">

## Can I do 3D stuff?

Yes; ggez uses `gfx-rs` for its drawing, and you can access the underlying `gfx-rs` drawing functions to draw whatever you want without disrupting ggez's drawing state.  See the `cube` example.

In general, ggez is designed to focus on 2D graphics.  We want it to be possible for you to create a 3D engine using ggez for everything EXCEPT drawing, but we don't really want to make a full 3D drawing engine.  If you want 3D drawing and don't feel like doing it yourself, check out [Amethyst](https://crates.io/crates/amethyst).

<a name="gfx_gui">

## How do I make a GUI?

There's no single optimal way to do it currently, but as of 2021 there's a few
GUI libraries that are able to use `ggez` as a drawing backend.
`raui` seems to offer a `ggez` backend natively, though we have no idea
how well it works, and `iced` used to have one, but it seems to have
vanished with a code rewrite. [`egui`] seems to work well with `ggez`.

There's several other IMGUI-style GUI crates that have pluggable drawing backends,
maybe some of them can either be drawn with `ggez` or are easy to write new backends for.

Contributions are welcome! ;-)

<a name="gfx_resolution">

## Resolution independence (or "Why do things not end up where I want them to be?")

By default ggez uses a coordinate system corresponding to the window size
in physical pixels, but you can change that by calling something like

```rust
graphics::set_screen_coordinates(&mut context, Rect::new(0.0, 0.0, 1.0, 1.0)).unwrap();
```

and scaling your `Image`s with `graphics::DrawParam`.
 
Please note that updating your coordinate system like this may also
be necessary [when drawing onto canvases of custom sizes](https://github.com/ggez/ggez/blob/aed56921fbca8ac8192b492f0a46d92e4a0a95bb/src/graphics/canvas.rs#L44-L48).

<a name="offsets">

## Relative and absolute offsets with `DrawParam::offset`

Offset behavior in ggez has changed a bit in recent times.

In ggez 0.6.1 `DrawParam::offset` used to be interpreted as a _relative_ offset for `Image`, `SpriteBatch`, `MeshBatch`, `Text` and `Canvas` and as an _absolute_ offset for `Mesh`.

Then, we wanted to unify this and switch `Mesh` over to a relative interpretation as well, but we discovered, that [this
relative interpretation can be really problematic for certain `Drawable`s](https://github.com/ggez/ggez/issues/736#issuecomment-945181003), so now the divide is as follows:

+ `Image`, `Canvas` and the sprites inside a `SpriteBatch` use the relative interpretation
+ `Mesh`, `MeshBatch`, `Spritebatch` (and thereby `Text` too) use the absolute interpretation

This is how offsets worked before ggez 0.6 and it's how they work now, for good reasons.

What this means for you is: If you want `DrawParam::offset` to be a relative offset (i.e. [1,1] means "bottom right", [0.5,0.5] means "centered", etc.) for any of the types mentioned as
"absolute interpretations" above, then you'll have to adapt your offset like this:

```rust
// scale up and move the offset according to the dimensions of the Drawable
// the move is necessary as the dimensions (i.e. the bounding box) may not
// necessarily start at [0,0]
let mut new_param = param;
if let Transform::Values { offset, .. } = param.trans {
    if let Some(dim) = drawable.dimensions(ctx) {
        let new_offset = mint::Vector2 {
            x: offset.x * dim.w + dim.x,
            y: offset.y * dim.h + dim.y,
        };
        new_param = param.offset(new_offset);
    }
}
```

If, however, you find yourself desiring to use absolute offsets on any of the types declared
to have "relative interpretation" here instead, you'll have to do almost the opposite:

```rust
// scale down the offset according to the dimensions of the Drawable
let mut new_param = param;
if let Transform::Values { offset, .. } = param.trans {
    if let Some(dim) = drawable.dimensions(ctx) {
        let new_offset = mint::Vector2 {
            x: offset.x / dim.w,
            y: offset.y / dim.h,
        };
        new_param = param.offset(new_offset);
    }
}
```

<a name="input">

# Input

<a name="mouse_coords">

## Returned mouse coordinates are wrong!

This issue tends to come up when your screen coordinate system becomes
different from what it initially was, or when the physical window size
is changed, for example by maximizing the window.

The underlying reason for this is that mouse coordinates are returned
as positions given in physical pixels on the screen, instead of being
given as logical positions inside your current screen coordinate system.

When created, a window starts out with a coordinate system perfectly
corresponding to its physical size in pixels. That's why, initially,
translating mouse coordinates to logical coordinates is not necessary
at all. Both systems are just the same.
 
But once physical and logical coordinates get out of sync problems
start to arise. If you want more info on how to navigate this issue
take a look at the [`input_test`](../examples/input_test.rs) and [`graphics_settings`](../examples/graphics_settings.rs) examples.

<a name="libraries">

# Libraries

<a name="library_ecs">

## Can I use `specs`, `legion` or another entity-component system?

Sure!  ggez doesn't include such a thing itself, since it's more or less
out of scope for this, but it is specifically designed to make it easy
to Lego together with other tools.  The [game
template](https://github.com/ggez/game-template) repo is a little old
but demonstrates how to use ggez with `specs` for ECS, `warmy` for
resource loading, and other nice crates. This template is available with
`legion` in place of `specs` as well
[here](https://github.com/Quetzal2/game-template).

<a name="library_mint">

## What is mint and how do I use `Into<mint::Point2<f32>>` and other `Into<mint::T>` types?

</a>
mint stands for "Math INteroperability Types" which means that it
provides types for other math libraries to convert to and from with.
What you are supposed to do is to add a math library of your choice to
your game such as glam or nalgebra, usually with a "mint" feature.  For
example. You can add
 
 ```rust
 glam = { version = "0.15.2", features = ["mint"] }
 ```
 
 in your Cargo.toml, then when you try to pass
something to, say `DrawParam::new().dest(my_point)`, you will
be able to pass a glam type like
`DrawParam::new().dest(glam::vec2(10.0, 15.0))` to set the
destination to x=10 and y=15.  Going the other way around is a bit more
verbose, you need to do `glam::Vec2::from(my_draw_param.dest)`

Another example, moving a draw param's destination diagonally by 1 down
and 1 right.

```rust
let dest = glam::Vec2::from(my_draw_param.dest);
let new_dest = dest + glam::vec2(1.0, 1.0);
DrawParam::new().dest(new_dest)
```

or simply

```rust
DrawParam::new().dest(glam::Vec2::from(my_draw_param.dest) + glam::vec2(1.0, 1.0))
```

# Performance

<a name="perf_slow1">

## Image/sound loading and font rendering is slow!

Are you running in debug or release mode?  Rust in general is very
slow in debug mode. This causes problems because there is currently no
way to build ggez in debug mode but build all it's dependencies in
release mode. So, things like `image` and `rusttype` end up doing a
lot of very un-optimized number crunching.

It is recommended to set debug mode to build with opt-level=1, which
gets at least marginally acceptable performance.  Just add the
following to your `Cargo.toml`:

```toml
[profile.dev]
opt-level = 1
```

Example benchmarks for a game that did some font rendering each frame:

```
opt-level = 0: 14-15 fps
opt-level = 1: 52 fps
opt-level = 2: 430 fps
opt-level = 3: 450 fps
```

<a name="perf_debug">

## That's lame, can't I just compile my game in debug mode but ggez with optimizations on?

Actually, as of rustc 1.41, you can!  See
<https://doc.rust-lang.org/cargo/reference/profiles.html#overrides> for info
on how to do that.

<a name="perf_drawing">

## Drawing a few hundred images or shapes is slow!

Again, debug mode is slow.  Plus, each single draw call has some overhead.  If building in release mode still isn't fast enough, then look into using `SpriteBatch` to draw a bunch of chunks from a spritesheet (also known as an atlas).  If you're drawing geometry, instead of using `graphics::rectangle()` or `graphics::circle()` and such, which create a new `Mesh` on each call and then throw it away, create and store a `Mesh` and draw it many times, or use a `MeshBuilder` to build a single `Mesh` out of many separate shapes.

<a name="platforms">

# Platform-specific

<a name="platform_build">

## How do I build on platform X?

See the [build
docs](https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md).
If your question is not answered there, open an
[issue](https://github.com/ggez/ggez/issues).

<a name="platform_mac">
## Is Mac/iOS supported?

Apple will be supported when they treat programmers trying to use
their systems as something other than third-class citizens.  See
<https://drewdevault.com/2017/10/26/Fuck-you-nvidia.html> for the
general message, but replace "NVidia" with "Apple" and make it an
ongoing problem of continual exploitation that has made Apple the
richest company in the world off of the work of others.

That said, `ggez` will probably build and run fine on Mac, and pull
requests for Mac-specific bugs will be accepted as long as they don't
break anything else.  In the mean time, consider writing your software
for a company that doesn't treat you like dirt.

<a name="contributing">

# Contributing

<a name="contribute_inclusion">

## If I write X, will you include it in ggez?

Maybe, if it's something that fits in with ggez's goals: a simple and
flexible 2D game framework with a LÖVE-ish API, which provides all the
basics you need in one package without dictating too much about the more
complicated tools.

Examples of things that would be included:

 * Sprite batches -- extension of existing functionality, follows LÖVE's
   example, large performance win
 * Glyph cache -- replaces existing functionality with a more capable
   version, large performance win
 * Sound mixer -- Follows LÖVE's example, fundamental functionality that
   should be provided, not tool-specific

Examples of things that would not be included:

 * Map loader for the Tiled map editor -- No reason we should force a
   user into a particular tool format
 * Sprite animation engine -- Makes assumptions about the sort of game
   the user will create, easily made its own crate
 * GUI library -- A large and complicated problem, and it doesn't need
   to be part of ggez to solve the problem

Part of the goal of this sort of setup is to make it easy for people to
write more sophisticated tools atop ggez!  By all means, write your
Tiled map drawer or your aseprite sprite loader!  Submit a PR to add it
to the `docs/Projects.md` file!  We'd love to have an ecosystem of
awesome tools.

One favor to ask: If you're making a crate to do `foo`, please don't
name it `ggez-foo`.  It makes it harder to search for ggez on crates.io
and get things that are officially supported by the maintainers, such as
`ggez-goodies`.  For an example, search for `gfx` on `crates.io` and see
how messy the results are.

For a fuller discussion of this, see [issue #373](https://github.com/ggez/ggez/issues/373).

<a name="misc">

# Miscellaneous

<a name="misc_conf">

## How do I load my `conf.toml` file?

When you create a `Context` it will automatically look for a
`conf.toml` file in any of the resource directories and, if it finds
one, use that to override all the defaults you give it.

The `files` example should demonstrate this, and more.

<a name="misc_win_console">

## I get a console window when I launch my executable on Windows

You can disable the console entirely by adding the following at
the top of your `main.rs` file:

```rust
#![windows_subsystem = "windows"]
```

If you wish, you can also disable it only in release mode:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

[`egui`]: https://github.com/emilk/egui#integrations

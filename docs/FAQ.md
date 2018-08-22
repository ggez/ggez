# I get `ResourceNotFound("/myfile", ...)` even though it's in the resource dir!

Okay, first, look at [the docs](https://docs.rs/ggez/) for the
`filesystem` module.  That should say exactly where it should look for
files.  Note that paths **must start with leading slash**; relative
paths are not allowed!  Also note that it expects the `resources/`
directory to be beside the *executable*, not in the cargo root dir,
which is annoying because cargo tends to put the executable in
`target/debug/whatever`.  You can add the cargo root dir to the lookup
path by pulling it from the environment variable, see the examples for
how.  Sorry, there's no especially good way of doing it automatically;
we've tried.

If that doesn't help, call `Context::print_resource_stats()`.  That
should print out all the files it can find, and where it is finding
them.

If you want to add a non-standard location to the resources lookup
path, you can use `Filesystem::mount()` or
`ContextBuilder::add_resource_path()`; see the examples for examples.

# Why do I get `WindowError(SdlError("Could not create GL context"))` when I try to run my game?

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
wanted about exactly what your graphics driver supports, and if you
dig enough through it you can find what version of OpenGL it supports.

To request different graphics settings you can change the appropriate
entries in the `Conf` object before creating your `Context`.  If you
request older versions of OpenGL you will also have to provide shaders
written in the appropriate version of GLSL (which is a bit of a WIP)
and there's no promises that things like `SpriteBatch` and `Canvas`
will work.

# Image/sound loading and font rendering is slow!

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

# Text rendering is still slow!

Rendering text to a bitmap is actually pretty computationally expensive.  If you call `Text::new()` every single frame it's going to take a relatively large amount of time, and larger bitmaps and more text will take longer.

Ideally you'd be able to use a glyph cache to render letters to a texture once, and then just create a mesh that uses the bits of that texture to draw text.  There's a couple partial implementations, such as the [gfx_glyph crate](https://crates.io/crates/gfx_glyph).

# Drawing a few hundred images or shapes is slow!

Again, debug mode is slow.  Plus, each single draw call has some overhead.  If building in release mode still isn't fast enough, then look into using `SpriteBatch` to draw a bunch of chunks from a spritesheet (also known as an atlas).  If you're drawing geometry, instead of using `graphics::rectangle()` or `graphics::circle()` and such, which create a new `Mesh` on each call and then throw it away, create and store a `Mesh` and draw it many times, or use a `MeshBuilder` to build a single `Mesh` out of many separate shapes.


# How do I build on platform X?

See the [build docs](https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md).  If your question is not answered there, open an [issue](https://github.com/ggez/ggez/issues).

# Can I do 3D stuff?

Yes; ggez uses `gfx-rs` for its drawing, and you can access the underlying `gfx-rs` drawing functions to draw whatever you want without disrupting ggez's drawing state.  See the `cube` example.

In general, ggez is designed to focus on 2D graphics.  We want it to be possible for you to create a 3D engine using ggez for everything EXCEPT drawing, but we don't really want to make a full 3D drawing engine.  If you want 3D drawing and don't feel like doing it yourself, check out [Amethyst](https://crates.io/crates/amethyst).

# How do I make a GUI?

As of 2017 we know of no good ui options thus far besides "implement
it yourself" or "write a backend for Conrod or something so it can
draw using ggez".

Contributions are welcome! ;-)

# Trying to build something gives me "library not found for -lSDL2"

You don't have the SDL2 development libraries installed.  See [build docs](https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md) for how to install them for your platform.

# How do I load my `conf.toml` file?

When you create a `Context` it will automatically look for a
`conf.toml` file in any of the resource directories and, if it finds
one, use that to override all the defaults you give it.

The `files` example should demonstrate this, and more.

# Resolution independence

By default ggez uses a pixel coordinate system but you can change that
by calling something like

```rust
graphics::set_screen_coordinates(&mut context, Rect::new(0.0, 0.0, 1.0, 1.0)).unwrap();
```

and scaling your `Image`s with `graphics::DrawParam`.

# Can I use `specs` or another entity-component system?

Sure!  ggez doesn't include such a thing itself, since it's more or less out of scope for this, but it is specifically
designed to make it easy to Lego together with other tools.  The [game template](https://github.com/ggez/game-template) repo
demonstrates how to use ggez with `specs` for ECS, `warmy` for resource loading, and other nice crates.

# If I write X, will you include it in ggez?

Maybe, if it's something that fits in with ggez's goals: a simple and flexible 2D game framework with a LÖVE-ish API,
which provides all the basics you need in one package without dictating too much about the more complicated tools.

Examples of things that would be included:

 * Sprite batches -- extension of existing functionality, follows LÖVE's example, large performance win
 * Glyph cache -- replaces existing functionality with a more capable version, large performance win
 * Sound mixer -- Follows LÖVE's example, fundamental functionality that should be provided, not tool-specific

Examples of things that would not be included:

 * Map loader for the Tiled map editor -- No reason we should force a user into a particular tool format
 * Sprite animation engine -- Makes assumptions about the sort of game the user will create, easily made its own crate
 * GUI library -- A large and complicated problem, and it doesn't need to be part of ggez to solve the problem

Part of the goal of this sort of setup is to make it easy for people to write more sophisticated tools atop ggez!  By all
means, write your Tiled map loader or your aseprite sprite loader!  Submit a PR to add it to the `docs/Projects.md` file!
We'd love to have an ecosystem of awesome tools.

One favor to ask: If you're making a crate to do `foo`, please don't name it `ggez-foo`.  It makes it harder to search for
ggez on crates.io and get things that are officially supported by the maintainers, such as `ggez-goodies`.  For an
example, search for `gfx` on `crates.io` and see how messy the results are.

For a fuller discussion of this, see [issue #373](https://github.com/ggez/ggez/issues/373).

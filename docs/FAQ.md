# Help, my game can't find a file it needs to load!

TODO

# Why do I get WindowError(SdlError("Could not create GL context")) when I try to run my game?

TODO

# Image/sound/loading and font rendering is slow in debug mode!

Rust in general is very slow in debug mode. This causes problems because theres currently no way to build ggez in debug mode but build all it's dependencies in release mode. So, the image crate ends up very slow.

I usually set debug mode to build with opt-level=1 in my projects, which gets at least marginally acceptable performance.  Example benchmarks for a game that did some font rendering each frame:

```
opt-level = 0: 14-15 fps
opt-level = 1: 52 fps
opt-level = 2: 430 fps
opt-level = 3: 450 fps
```

# How do I build on platform X?

See the [build docs](https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md).  If your question is not answered there, open an [issue](https://github.com/ggez/ggez/issues).

# Text rendering is still slow!

Rendering text to a bitmap is actually pretty computationally expensive.  If you call `Text::new()` every single frame it's going to take a relatively large amount of time, and larger bitmaps and more text will take longer.

Ideally you'd be able to use a glyph cache to render letters to a texture once, and then just create a mesh that uses the bits of that texture to draw text.  There's a couple partial implementations, such as the [gfx_glyph crate](https://crates.io/crates/gfx_glyph).

# Can I do 3D stuff?

Yes, by drawing with `gfx-rs`; see the `cube` example.  HOWEVER, as of 0.3.3 this is not necessarily... uh, working.  When 0.4 is released there should be a full 3D drawing example.  TODO: Make sure this doc gets updated before 0.4 is released!

In general, ggez is designed to focus on 2D graphics.  We want it to be possible for you to create a 3D engine using ggez for everything EXCEPT drawing, but we don't really want to make a full 3D drawing engine.  If you want that, check out [Amethyst](https://crates.io/crates/amethyst).
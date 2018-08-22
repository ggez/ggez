# 0.4.4

## Added

 * Added functions to get and set mouse cursor visibility.
 * Derived `PartialEq` for `Image` and `SpriteBatch`.

## Changed

Nothing

## Deprecated

Nothing

## Removed

Nothing

## Fixed

 * Myriad small documentation and example typos.
 * Fixed a rounding error in `Font::get_width()`.

# 0.4.3

## Added

 * Added a feature flag to build nalgebra with the `mint` math library inter-operability layer [#344](https://github.com/ggez/ggez/issues/344)
 * Updated `image` to 0.19 which lets us add another feature flag selecting whether or not to use multithreaded libraries when loading images.  [#377](https://github.com/ggez/ggez/issues/377)
 * We got more awesome logos!  Thanks ozkriff and termhn! [#327](https://github.com/ggez/ggez/issues/327)
 * Added hooks to the `log` crate, so we will now output some logging data via it that clients may use.  [#311](https://github.com/ggez/ggez/pull/331)
 * There's now a functional and reasonably ergonomic [game template](https://github.com/ggez/game-template) repo that demonstrates how to use `ggez` with `specs`, `warmy`, `failure`, `log` and other useful tools.
 * Added `Font::new_px()` and `Font::from_bytes_px()` functions to create fonts that are specific pixel sizes  [#268](https://github.com/ggez/ggez/issues/268)
 * Added Ratysz's glyph cache implementation integrating the awesome `gfx_glyph` crate!  This gives us faster text drawing as well as more features; if it works out well it should replace all text rendering in another version or two.  [#132](https://github.com/ggez/ggez/issues/132)

## Changed

 * Made it so that the configuration directories are only created on-demand, not whenever the Context is created: [#356](https://github.com/ggez/ggez/issues/356)
 * Updated rodio to 0.7, which fixes a sample rate bug on Linux: [#359](https://github.com/ggez/ggez/issues/359)
 * Documented which version of rustc we require, and added unit tests for that specific version: it is currently >=1.23.0,
   primarily driven by features required by dependencies.

## Deprecated

## Removed

## Fixed


# 0.4.2

## Added

 * Added a feature to enable or disable bzip2 zip file support
 * Lots of small documentation fixes and improvements thanks to lovely contributors
 * Added termhn's `ggez_snake` to the examples, 'cause it's awesome
 * Added `timer::get_remaining_update_time()` to let you easily do sub-frame timing for interpolation and such.
 * Many small improvements and cleanups

## Changed

 * Version bumped lots of dependencies: zip, rand, rodio, rusttype
 * Switched to the `app_dirs2` crate to avoid a bug in upcoming rustc change

## Deprecated

## Removed

## Fixed

 * Made `Image::from_rgba8` properly check that the array you pass it is the right size
 * Fixed more documentation bugs (https://github.com/ggez/ggez/issues/303).

# 0.4.1

## Added

 * Added `Text::into_inner()` and related methods to get ahold of a `Text` object's underlying `Image`
 * Added `SoundData::new()` and `Source::set_repeat()`/`Source::get_repeat()` (thanks jupart!)
 * Added `Context::process_event()` to smooth out a bump or two in the
   API for writing custom event loops.  This does change the API a little, but the old style should still work.
 * Added functions for taking screenshots and saving `Image`'s (thanks DenialAdams!)

## Changed

 * Version-bumped `lyon` crate

## Deprecated

 * Deprecated `BoundSpriteBatch`, since you can just clone an `Image`
   relatively cheaply.

## Removed

 * Nothing

## Fixed

 * Fixed bug in `mouse::get_position()`, see https://github.com/ggez/ggez/issues/283
 * Lots of small documentation fixes from a variety of awesome sharp-eyed contributors
 * Fixed bug that was making canvas's render upside-down https://github.com/ggez/ggez/issues/252

# 0.4.0

## Added

 * Added `mouse` module with some utility functions
 * Added some utility functions to query window size
 * Sprite batching implemented by termhn!
 * Added mesh builders allowing you to build complex meshes simply.
 * Integrated nalgebra to provide point and vector types.
 * Added MSAA, blend modes, other graphics toys (thanks termhn!)
 * Added graphics_settings example to show hot to play with graphics modes
 * Made the render pipeline just use matrices instead of separate transform elements
 * SHADERS!  Woo, thanks nlordell!
 * Added `Filesystem::mount()` function and made examples use it; they no longer need the `cargo-resource-root` feature
 * Added filesystem and graphics setting examples
 * Added more useful/informative constructors for `Color`
 * Added ability to select OpenGL version
 * Added some useful methods to `Rect`
 * Added a FAQ and some other documentation
 * Added a `ContextBuilder` type that allows finer control over creating a `Context`
 * Added an optional `color` value to `DrawParam`, which overrides the default foreground color.  Life would be simpler removing the foreground color entirely...

## Changed

 * First off, there will be some switches in process: We're going to make the master branch STABLE, tracking the latest release,
   and create a devel branch that new work will be pushed to.  That way people don't check out master and get some WIP stuff.
 * The coordinate system moved from origin-at-center, x-increasing-up to origin-at-top-left, x-increasing-down
 * Updated all dependencies to newer versions
 * Refactored EventHandler interface, again
 * Altered timestep functions to be nicer and made examples use them consistently
 * Updated to Lyon 0.8, which brings some bugfixes
 * Refactored Conf interface a little to separate "things that can be changed at runtime" from "things which must be specified at init time".

## Deprecated

## Removed

 * Removed `get_line_width()` and `set_line_width()` and made line widths parameters where necessary
 * Did the same for `get/set_point_size()`
 * Removed inaccurate `timer::sleep_until_next_frame()`, added `timer::yield_now()`.

## Fixed

 * Fixed some bugs with type visibility and directory paths.
 * Fixed a few smallish filesystem bugs
 * Got the 3D cube example working and shuffled around the gfx-rs interface methods a little, so we could make more of the graphics innards hidden while still exposing the useful bits.

# 0.3.4

 * Backported correction to SRGB color conversions
 * Added std::error::Error implementation for GameError

# 0.3.3

 * Documentation and unit test updates
 * Derive some common traits on types

# 0.3.2

 * Fixed bug in conf.toml reading and writing (thanks chinatsu)
 * Made filesystem.print_all() a little more informative
 * Added graphics::set_mode() function to allow setting window size, etc.
 * Added some functions to allow querying fullscreen modes and such
 * Made gamepad example test all input
 * Added bindings to the `mint` crate (a whole one type conversion)
 * Implemented stop() for audio

# 0.3.1

 * Fixed bug in when CARGO_MANIFEST_DIR is checked (thanks 17cupsofcoffee)
 * Added experimental support for SDL's gamepads (thanks kampffrosch94)
 * Re-improved resource-not-found error messages (thanks 17cupsofcoffee)
 * Fixed minor bug with text rendering alpha, added more useful methods to `Text`
 * Fixed bug with text wrapping (I hope)
 * VERY EXPERIMENTAL functions for exposing the gfx-rs rendering context to a bold user

# 0.3.0

 * Almost everything is now pure rust; the only C dependency is libsdl2.
 * Entirely new rendering engine using `gfx-rs` backed by OpenGL 3.2
 * New (if limited) 2D drawing primitives using `lyon`
 * Font rendering still uses `rusttype` but it's still cool
 * New option to enable/disable vsync
 * New sound system using `rodio`, supporting pure Rust loading of WAV, Vorbis and FLAC files
 * Configuration system now uses `serde` rather than `rustc_serialize`
 * Refactored event loop handling somewhat to make it less magical and more composable.
 * New filesystem indirection code using `app_dirs`, and `cargo-resource-root` feature flag.

# 0.2.2

Added `set_color_mod` and `set_alpha_mod` functions which I'd forgotten

# 0.2.1

IIRC, switched from SDL_ttf to rusttype because of horrible evil API's not playing nice with
lifetimes.

# 0.2.0

Made a fairly fully fleshed out SDL implementation

# 0.1.0

Initial proof of concept

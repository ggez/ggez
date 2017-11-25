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

Made a fairly fully fleshed out SDL implmentation

# 0.1.0

Initial proof of concept

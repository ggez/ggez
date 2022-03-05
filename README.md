# ggez-fork

## What is this?

A fork of ggez with a graphics backend completely rewritten to use WGPU, along with an API redesign.

### Features

- Filesystem abstraction that lets you load resources from folders or zip files
- Hardware-accelerated 2D rendering built on `wgpu`
- Loading and playing .ogg, .wav and .flac files via the `rodio` crate
- TTF font rendering with `wgpu_glyph`
- Interface for handling keyboard and mouse events easily through callbacks
- Config file for defining engine and game settings
- Easy timing and FPS measurement functions.
- Math library integration with `mint`.
- Some more advanced graphics options: shaders, sprite batches and render targets

### Non-Features (i.e. things to add from elsewhere if needed)

- [Physics](https://arewegameyet.rs/ecosystem/physics/)
- Animation (check out [keyframe](https://github.com/HannesMann/keyframe); [it works pretty well with ggez](https://psteinhaus.github.io/ggez/web-examples/) ([source](https://github.com/PSteinhaus/PSteinhaus.github.io/tree/main/ggez/web-examples)))
- [GUI](https://arewegameyet.rs/ecosystem/ui/)
- [Assets manager](https://github.com/a1phyr/assets_manager)
- [AI](https://arewegameyet.rs/ecosystem/ai/)
- [ECS](https://arewegameyet.rs/ecosystem/ecs/)
- [Networking](https://arewegameyet.rs/ecosystem/networking/)

### Supported platforms

The following platforms receive primary support and are known to work:

- Window
- Linux
- MacOS

The following platforms are untested, but the backend is compatible (i.e. no major changes needed to make fully compatible):

- iOS
- Android
- Web

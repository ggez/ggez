# Introduction

Greetings, one and all.  Today we shall explore how to build and
deploy a `ggez` game for every possible platform.  For platforms like
Linux it's pretty darn simple, but sometimes you have to jump through a
couple hoops.  The purpose of this is to document the hoops and give you
a cookbook on the best jumping methods and trajectories.  We will
progress generally from the easiest to hardest jumps.

## Project setup

We will use the `02_hello_world` example program from ggez for all these
examples.  To do the initial setup, assuming you have cargo installed:

```sh
cargo init --bin hello_world
cd hello_world
```

Now copy-paste the contents of
<https://raw.githubusercontent.com/ggez/ggez/master/examples/02_hello_world.rs>
into `hello_world/src/main.rs`, or if you are on Linux, just wget it:

```sh
wget https://raw.githubusercontent.com/ggez/ggez/master/examples/02_hello_world.rs -O src/main.rs
```

You'll need a font to print "Hello world!" with, so we need to fetch one and
put it in a subdirectory called `resources` in your project root:

```sh
mkdir resources
cd resources
wget https://raw.githubusercontent.com/ggez/ggez/master/resources/LiberationMono-Regular.ttf
```

Then edit your `Cargo.toml` with your favorite super duper editor and under `[dependencies]` add:

```
ggez = "0.10.0"
glam = { version = "0.30", features = ["mint"] }
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
apt install libasound2-dev libudev-dev pkg-config
```

Then you should be able to build with `cargo run`

## Redhat

Same libraries as Debian, slightly different names.  On CentOS 7 at
least you can install them with:

```sh
yum install alsa-lib-devel
```

### Fedora

```sh
dnf install systemd-devel alsa-lib-devel pkgconf-pkg-config
```

## Distributing

You should be able to just copy-paste the executable file and the `resources` directory to wherever you want.


# Windows

Should just build.  We recommend using the MSVC toolchain whenever possible, the MinGW one can be pretty jank and difficult to set up.

## Distributing

Just copy-paste the exe and resource directory to the destination computer.

# MacOS

Should just build.

## Distributing

You *can* just directly distribute an exe and resource directory but macos typically has .apps.
You can look into the structure of these(Basically just a folder with a certain layout) or do something
with xcode.

# Android

Not officially supported yet. ;_; See https://github.com/ggez/ggez/issues/70

You might be able to use [`good-web-game`] though to run your `ggez` app on Android.

# Web (wasm32)

ggez targets `wasm32-unknown-unknown` via `wasm-bindgen`, and only supports WebGPU (not WebGL2).
On Linux you may need to enable WebGPU in your browser (e.g. `chrome://flags/#enable-unsafe-webgpu`).

Start your game with `ContextBuilder::run` (or `run_with`); `ContextBuilder::build` blocks, which
doesn't work in a browser.

```sh
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

Then run `wasm-bindgen` on the output to get the JS module + `.wasm`. Use `--target web` (or
`bundler`), and note that the CLI version **must match** `wasm-bindgen` in your `Cargo.lock`:

```sh
cargo install --locked --version <X.Y.Z> wasm-bindgen-cli
wasm-bindgen --target web --out-dir ./out target/wasm32-unknown-unknown/release/your_game.wasm
```

```html
<script type="module">
  import init from './out/your_game.js';
  await init();
</script>
```

The canvas is appended to `<body>`, or to the element set with
`WindowSetup::web_canvas_parent_id("my-element")`.

[`examples/web-minimal/`](../examples/web-minimal/) is a complete page built this way, with no
bundler: a shell script plus one HTML file that also handles resources and audio (see below).
[`examples/web/`](../examples/web/README.md) builds every example and serves them in a gallery with
Vite.

## Asset loading

Browsers can't read files synchronously, so before the wasm module starts, the page should set
`window.__GGEZ_RESOURCES_ZIP__` to a `Uint8Array` containing a zip of your resources. ggez mounts it
as the resources directory, so `Filesystem` and `from_path` APIs work as usual
(see [`examples/web-minimal/index.html`](../examples/web-minimal/index.html)). Alternatively, embed a zip with
`ContextBuilder::add_zipfile_bytes`. `add_resource_path` is ignored on web.

## Known limitations on web

- Browsers block audio until the user interacts with the page. ggez resumes it on the first
  pointer or key event, but sounds played before that are dropped (check `ctx.audio.is_running()`).
- `SpatialSource::stop()` can freeze the tab. Prefer `pause()` and dropping the source.
- `Image::to_pixels` is not supported and returns an error.

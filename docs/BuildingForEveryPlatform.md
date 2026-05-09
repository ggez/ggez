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
ggez = "0.10.0-rc0"
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

ggez targets `wasm32-unknown-unknown` via `wasm-bindgen`. Audio uses the WebAudio backend of
`rodio`, assets via `fetch`, and the a canvas is appended to `<body>` (or a DOM element you choose).

```sh
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

Once built, run `wasm-bindgen` against the output to get an ES module + `.wasm`
pair you can load from a static page. The CLI version **must match** the
`wasm-bindgen` version pinned in `Cargo.lock` (currently 0.2.x):

```sh
cargo install --locked --version <X.Y.Z> wasm-bindgen-cli
wasm-bindgen --target web --out-dir ./out target/wasm32-unknown-unknown/release/your_game.wasm
```

Then load it from any static host:

```html
<script type="module">
  import init from './out/your_game.js';
  await init();
</script>
```

## Trying the bundled examples

A small Vite project is included in `examples/web/` that compiles every example to wasm and
serves them in a gallery. One-time setup:

```sh
rustup target add wasm32-unknown-unknown
cd examples/web
npm install
```

Then:

```sh
npm run dev                   # build everything + serve at http://localhost:5173
npm run build -- 04_snake     # rebuild a single example
npm run build -- --release    # release-mode wasm (much smoother)
npm run serve                 # serve a previously built tree
```

`build.mjs` will `cargo install` a matching `wasm-bindgen-cli` automatically on first run;
pass `--no-bindgen-install` to opt out. See [`examples/web/README.md`](../examples/web/README.md) for the full layout.

## Embedding the canvas inside an existing page

By default ggez appends its `<canvas>` to `<body>`. To attach it to a specific element:

```rust
let cb = ggez::ContextBuilder::new("my_game", "me")
    .window_setup(
        ggez::conf::WindowSetup::default().web_canvas_parent_id("my-element"),
    );
```

This is a no-op on other targets, so you can leave it on a single build configuration.

## Asset loading

Browsers can't do synchronous file I/O, so on web ggez expects the JS host to hand it a single
`Uint8Array` containing a zip of your resources, stashed on `window.__GGEZ_RESOURCES_ZIP__` before
the wasm module starts. ggez mounts that zip as a `ZipFS` so the regular synchronous
`Filesystem::open` / `read` / `read_to_string` calls just work.

The bundled `examples/web/` pipeline shows one way to produce that zip — see
[`examples/web/src/runner.js`](../examples/web/src/runner.js) and
[`examples/web/build.mjs`](../examples/web/build.mjs). `add_resource_path` and the on-disk
`resources.zip` lookups are no-ops on the web target.

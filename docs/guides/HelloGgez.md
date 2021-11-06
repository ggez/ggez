# 👋 Hello `ggez`!

This is a guide to create a program that prints `Hello ggez! dt = 78986ns` every frame.

At the end of this guide you will:
*  Have created a simple program using `ggez`
*  Know the basic scaffolding of a `ggez` program

## ⚠ Prerequisites

### Rust

[Rust](https://www.rust-lang.org) will need to be installed.
Follow Rust's guides for installing the latest stable version.
You did know this was a Rust library right? 😉

## 🛠 Project Setup

### Cargo Crate

Open up your terminal and make sure it is in the location where you want to create your new project folder.

```bash
cargo new --bin hello_ggez
cd hello_ggez
```

Open up `Cargo.toml` with a text editor and add `ggez` as a dependency:

```toml
[package]
name = "hello_ggez"
version = "0.1.0"
# Replace with your name and email
authors = ["Awesome Person awesome@person.com"]

[dependencies]
ggez = "0.7"
```

### ✔ Check Project Setup

Test to make sure everything is correct by running `cargo run`.

Be patient.
This can take a while on the first run.
Subsequent builds will be much faster.
Be sure it says `Hello, world!` at the bottom before continuing.

## ggez

We have successfully compiled `ggez`, but we haven't used it yet!
What we have currently is the canonical Rust hello world.
Let's change that and create a more appropriate hello world for `ggez`.
It is why we're here after all.

First we'll tell Rust we want to use `ggez`.
Add this to the top of your `src/main.rs`:
```rust,skt-definition,no_run
use ggez::*;
```

Now we're cooking with `ggez`.

### Loop

All games have a loop.
Often the loop is referred to as a game loop.

Game loops are responsible for:

1. Handling events such as keyboard, mouse, closing window, etc.
1. Updating state such as player position, health, etc.
1. Drawing shapes, images, etc.

`ggez` provides an [`EventHandler`](https://docs.rs/ggez/0.7.0/ggez/event/trait.EventHandler.html) as the default recommended interface to its internal loop to use in our games. Thanks `ggez`!

You might have noticed [`EventHandler`](https://docs.rs/ggez/0.7.0/ggez/event/trait.EventHandler.html) is a Rust [Trait](https://doc.rust-lang.org/book/ch10-02-traits.html).
This means it is intended to be implemented on a struct.
There are quite a few callbacks defined on the Trait, but only [2 are required: update and draw](https://docs.rs/ggez/0.7.0/ggez/event/trait.EventHandler.html#required-methods).

Let's add `EventHandler` to our `src/main.rs` file:
```rust,skt-definition,no_run
struct State {}

impl ggez::event::EventHandler<GameError> for State {
  fn update(&mut self, ctx: &mut Context) -> GameResult {
      Ok(())
  }
  fn draw(&mut self, ctx: &mut Context) -> GameResult {
      Ok(())
  }
}
```

You'll notice a couple new things here: `State`, `Context`, and `GameResult`.

`State` is all of the data and information required to represent our game's current state.
These could be player positions, scores, cards in your hand, etc..
What is included in your state is very dependent on the game you are making.

[`Context`](https://docs.rs/ggez/0.7.0/ggez/struct.Context.html) is how `ggez` gives you access to hardware such as mouse, keyboard, timers, graphics, sound, etc..

[`GameResult`](https://docs.rs/ggez/0.7.0/ggez/error/type.GameResult.html) is a utility provided by `ggez` to signify if there was an error or not.
Internally it's just a `Result<(), GameError>`, which is why we implement `EventHandler` with `GameError`, so that [`on_error`](https://docs.rs/ggez/0.7.0/ggez/event/trait.EventHandler.html#method.on_error) knows what to expect.
But, we're not going to write any bugs right? 😉

In your main, you will need to create an instance of `State`.
```rust,skt-definition-no-main,no_run
pub fn main() {
    let state = State {};
}
```

### ✔ Check Loop Setup

Test to make sure everything is correct by running `cargo run`.

You should see this:
```
warning: unused variable: `state`
  --> src\main.rs:15:9
   |
15 |     let state = &mut State {};
   |         ^^^^^ help: if this is intentional, prefix it with an underscore: `_state`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `ctx`
 --> src\main.rs:6:26
  |
6 |     fn update(&mut self, ctx: &mut Context) -> GameResult {
  |                          ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`

warning: unused variable: `ctx`
 --> src\main.rs:9:24
  |
9 |     fn draw(&mut self, ctx: &mut Context) -> GameResult {
  |                        ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`

```

And then nothing should happen after it runs.
If you saw the warnings and nothing happened, you can continue.

### Context

Although we have created a loop, we haven't actually started using it yet.
Also, don't mind the loop does nothing right now.
We'll get to that soon.

Now is the time for us to interface with our hardware and do something fun.
To do that, you need to create a [`Context`](https://docs.rs/ggez/0.7.0/ggez/struct.Context.html) courtesy of `ggez`.

Add this to the end of your `main` fn:
```rust,skt-expression,no_run
let c = conf::Conf::new();
let (ctx, event_loop) = ContextBuilder::new("hello_ggez", "awesome_person")
    .default_conf(c)
    .build()
    .unwrap();
```

This will create a `Context` with the `game_id` `hello_ggez` and the author `awesome_person`.
It will also create an [`EventsLoop`](https://docs.rs/ggez/0.7.0/ggez/event/struct.EventsLoop.html).
We'll need it in a minute to call [`run`](https://docs.rs/ggez/0.7.0/ggez/event/fn.run.html).
Feel free to replace the author with yourself.
You are awesome after all.

Now you're ready to kick off the loop!
```rust,skt-expression,no_run
event::run(ctx, event_loop, state);
```

### ✔ Check Context

Once again run `cargo run`

You should get 2 warnings:
```
warning: unused variable: `ctx`
 --> src\main.rs:6:26
  |
6 |     fn update(&mut self, ctx: &mut Context) -> GameResult {
  |                          ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`
  |
  = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `ctx`
 --> src\main.rs:9:24
  |
9 |     fn draw(&mut self, ctx: &mut Context) -> GameResult {
  |                        ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`
```

And a window will pop-up.
Hit `escape` or click the close button to quit.

If you saw the 2 warnings and the window, you're good to continue!

### Use the Loop Luke

Alright! We're ready! Let's use the loop with the context!

For this program, we want to display the duration of each frame in the console along with the text `"Hello ggez!"`.

How should we do that? Well, let's look at the 2 callbacks we have in our loop and our `State` struct.

There is some information we want to track, so we'll modify `State` first.
```rust,skt-definition,no_run
struct State {
    dt: std::time::Duration,
}
```

`dt` is going to represent the time each frame has taken. It stands for "delta time" and is a useful metric for games to handle variable frame rates.

Now in `main`, you need to update the `State` instantiation to include `dt`:

```rust,skt-expression,no_run
let state = State {
    dt: std::time::Duration::new(0, 0),
};
```

So now that we have state to update, let's update it in our `update` callback!
We'll use [`timer::delta`](https://docs.rs/ggez/0.7.0/ggez/timer/fn.delta.html) to get the delta time.

```rust,skt-update,no_run
fn update(&mut self, ctx: &mut Context) -> GameResult {
    self.dt = timer::delta(ctx);
    Ok(())
}
```

To see the changes in `State`, you need to modify the `draw` callback.
```rust,skt-draw,no_run
fn draw(&mut self, ctx: &mut Context) -> GameResult {
    println!("Hello ggez! dt = {}ns", self.dt.as_nanos());
    Ok(())
}
```

Every frame, print out `Hello ggez! dt = {}ns`. This will print once a frame. Which is going to be a lot.

### ✔ Check Program

And yet again, run `cargo run`.
A window should pop up and your console should be spammed with `"Hello ggez!"` and dt's in nanoseconds.

If you see that. Congrats! 🎉
You've successfully bootstrapped a `ggez` program.

# 💪 Challenges

If you are looking to push yourself a bit further on your own, these challenges are for you.

* Display the time in milliseconds or seconds instead of nanoseconds
* Change the size or title of the window that appears
* Print the text to the window instead of the console
* Limit the framerate to 30fps
* Use another callback on `EventHandler` to do something fun
